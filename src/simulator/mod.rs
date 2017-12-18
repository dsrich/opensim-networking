use capabilities::Capabilities;
use circuit::{Circuit, CircuitConfig, MessageHandlerError, SendMessage};
pub use circuit::MessageHandlers;
use data::RegionInfo;
use futures::Future;
use login::LoginResponse;
use logging::Log;
use messages::{MessageInstance, MessageType};
use messages::all::{CompleteAgentMovement, CompleteAgentMovement_AgentData, CompletePingCheck,
                    CompletePingCheck_PingID, UseCircuitCode, UseCircuitCode_CircuitCode};
use systems::agent_update::{AgentState, Modality};
use std::sync::Mutex;
use textures::{GetTexture, TextureService};
use types::{Duration, Ip4Addr, UnitQuaternion, Url, Uuid, Vector3};
use tokio_core::reactor::Handle;

pub mod manager;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct SimLocator {
    pub sim_ip: Ip4Addr,
    pub sim_port: u16,
    //grid: Url,
    //grid_position: (u32, u32),
}

#[derive(Clone, Debug)]
pub struct ConnectInfo {
    pub capabilities_seed: Url,
    pub agent_id: Uuid,
    pub session_id: Uuid,
    pub circuit_code: u32,
    pub sim_ip: Ip4Addr,
    pub sim_port: u16,
}

impl From<LoginResponse> for ConnectInfo {
    fn from(l: LoginResponse) -> Self {
        ConnectInfo {
            capabilities_seed: l.seed_capability,
            agent_id: l.agent_id,
            session_id: l.session_id,
            circuit_code: l.circuit_code,
            sim_ip: l.sim_ip,
            sim_port: l.sim_port,
        }
    }
}

/// This struct manages all connections from the viewer to a (single) simulator
/// instance.
pub struct Simulator {
    caps: Mutex<Capabilities>,
    circuit: Mutex<Circuit>,
    texture_service: Mutex<TextureService>,

    handle: Handle,
    locator: SimLocator,

    // TODO: (future) can this be updated remotely somehow, i.e. by the estate manager?
    // If yes we should register appropriate message handlers which update this data,
    // and maybe also wrap it in a mutex.
    region_info: RegionInfo,
}

#[derive(Debug, ErrorChain)]
#[error_chain(error = "ConnectError")]
#[error_chain(result = "")]
pub enum ConnectErrorKind {
    #[error_chain(foreign)] CapabilitiesError(::capabilities::CapabilitiesError),
    #[error_chain(foreign)] IoError(::std::io::Error),
    #[error_chain(foreign)] MpscError(::std::sync::mpsc::RecvError),
    #[error_chain(foreign)] ReadMessageError(::circuit::ReadMessageError),
    #[error_chain(foreign)] SendMessageError(::circuit::SendMessageError),
    #[error_chain(custom)] Msg(String),
}

impl Simulator {
    pub fn connect(
        connect_info: &ConnectInfo,
        mut handlers: MessageHandlers,
        handle: Handle,
        log: &Log,
    ) -> Result<Simulator, ConnectError> {
        // Setup default handlers (TODO move to right place and make more transparent
        // to user?)
        handlers.insert(
            MessageType::StartPingCheck,
            Box::new(|msg, circuit| {
                let start_ping_check = match msg {
                    MessageInstance::StartPingCheck(m) => Ok(m),
                    _ => Err(MessageHandlerError::WrongHandler),
                }?;
                let response = CompletePingCheck {
                    ping_id: CompletePingCheck_PingID {
                        ping_id: start_ping_check.ping_id.ping_id,
                    },
                };
                circuit.send(response, false);
                Ok(())
            }),
        );

        let capabilities = Self::setup_capabilities(connect_info)?;
        info!(
            log.slog_logger(),
            "received capabilities from sim: {:?}",
            capabilities
        );
        let (circuit, region_info) = Self::setup_circuit(connect_info, handlers, log)?;
        let texture_service = Self::setup_texture_service(&capabilities, log.clone());
        let locator = SimLocator {
            sim_ip: connect_info.sim_ip.clone(),
            sim_port: connect_info.sim_port.clone(),
        };

        Ok(Simulator {
            caps: Mutex::new(capabilities),
            circuit: Mutex::new(circuit),
            region_info: region_info,
            texture_service: Mutex::new(texture_service),
            handle: handle,
            locator: locator,
        })
    }

    pub fn locator(&self) -> SimLocator {
        self.locator.clone()
    }

    pub fn region_info(&self) -> &RegionInfo {
        &self.region_info
    }

    pub fn send_message<M: Into<MessageInstance>>(
        &self,
        message: M,
        reliable: bool,
    ) -> SendMessage {
        self.circuit.lock().unwrap().send(message, reliable)
    }

    /// To call this method you need to use `EventLoop::run_with_handle`.
    pub fn get_texture(&self, id: &Uuid, handle: &Handle) -> GetTexture {
        self.texture_service.lock().unwrap().get_texture(id, handle)
    }

    fn setup_circuit(
        connect_info: &ConnectInfo,
        handlers: MessageHandlers,
        log: &Log,
    ) -> Result<(Circuit, RegionInfo), ConnectError> {
        let config = CircuitConfig {
            send_timeout: Duration::from_millis(5000),
            send_attempts: 5,
        };
        let agent_id = connect_info.agent_id.clone();
        let session_id = connect_info.session_id.clone();
        let circuit_code = connect_info.circuit_code.clone();

        let circuit = Circuit::initiate(connect_info, config, handlers, log.clone())?;

        let message = UseCircuitCode {
            circuit_code: UseCircuitCode_CircuitCode {
                code: circuit_code,
                session_id: session_id,
                id: agent_id,
            },
        };
        circuit.send(message, true).wait()?;

        // Now wait for the RegionHandshake message.
        let timeout = Duration::from_millis(15_000);
        let region_info = match circuit.read(Some(timeout))? {
            MessageInstance::RegionHandshake(handshake) => {
                Ok(RegionInfo::extract_message(handshake))
            }
            _ => Err(ConnectError::from("Did not receive RegionHandshake")),
        }?;
        info!(
            log.slog_logger(),
            "Connected to simulator successfully, received region_info: {:?}",
            region_info
        );

        let message = CompleteAgentMovement {
            agent_data: CompleteAgentMovement_AgentData {
                agent_id: agent_id.clone(),
                session_id: session_id.clone(),
                circuit_code: circuit_code,
            },
        };
        circuit.send(message, true).wait()?;

        // let region_x = 256000.;
        // let region_y = 256000.;
        let local_x = 10.;
        let local_y = 10.;

        let z_axis = Vector3::z_axis();
        let agent_state = AgentState {
            position: Vector3::new(local_x, local_y, 0.),
            move_direction: None,
            modality: Modality::Walking,
            body_rotation: UnitQuaternion::from_axis_angle(&z_axis, 0.),
            head_rotation: UnitQuaternion::from_axis_angle(&z_axis, 0.),
        };
        let message = agent_state.to_update_message(agent_id, session_id);
        circuit.send(message, true).wait()?;

        Ok((circuit, region_info))
    }

    fn setup_capabilities(info: &ConnectInfo) -> Result<Capabilities, ConnectError> {
        Ok(Capabilities::setup_capabilities(
            info.capabilities_seed.clone(),
        )?)
    }

    fn setup_texture_service(caps: &Capabilities, log: Log) -> TextureService {
        TextureService::new(caps, log)
    }
}

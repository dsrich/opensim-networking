//! Contains some code to finish the initiation for a avatar to enter a new
//! region.
// TODO: include stuff like session id somehow with the Circuit struct so it
// doesn't have to be passed around all the time.
use Uuid;
use circuit::{Circuit, SendMessageError};
use messages::{UseCircuitCode, UseCircuitCode_CircuitCode};
use messages::{CompleteAgentMovement, CompleteAgentMovement_AgentData};
use logging::Logger;

use futures::Future;

pub fn initiate<L: Logger>(
    circuit: &Circuit,
    circuit_code: u32,
    agent_id: Uuid,
    session_id: Uuid,
    logger: L,
) -> Result<(), SendMessageError> {
    logger.debug(format!("using circuit code: {}", circuit_code));
    logger.debug(format!("session_id: {}", session_id));
    logger.debug(format!("agent id: {}", agent_id));

    let msg1 = UseCircuitCode {
        circuit_code: UseCircuitCode_CircuitCode {
            code: circuit_code,
            session_id: session_id,
            id: agent_id,
        },
    };

    let msg2 = CompleteAgentMovement {
        agent_data: CompleteAgentMovement_AgentData {
            agent_id: agent_id,
            session_id: session_id,
            circuit_code: circuit_code,
        },
    };

    logger.info("sending UseCircuitCode and waiting for ack");
    circuit.send(msg1, true).wait()?;
    logger.info("sending CompleteAgentMovement and waiting for ack");
    circuit.send(msg2, true).wait()?;
    Ok(())
}

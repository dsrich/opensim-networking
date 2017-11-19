// TODO: This should probably moved to its own crate at a later time. (Maybe some decoding
// facilities could be combined together conveniently.)

mod idct;
mod bitsreader;
mod extractor;

use nalgebra::DMatrix;

use messages::all::LayerData;
pub use self::extractor::{ExtractSurfaceError, ExtractSurfaceErrorKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LayerType {
    Land,
    Wind,
    Cloud,
    Water,
    VarLand,
    VarWind,
    VarCloud,
    VarWater,
}

impl LayerType {
    fn from_code(c: u8) -> Result<Self, ExtractSurfaceError> {
        match c {
            b'L' => Ok(LayerType::Land),
            b'7' => Ok(LayerType::Wind),
            b'8' => Ok(LayerType::Cloud),
            b'W' => Ok(LayerType::Water),
            b'M' => Ok(LayerType::VarLand),
            b'X' => Ok(LayerType::VarWind),
            b'9' => Ok(LayerType::VarCloud),
            b':' => Ok(LayerType::VarWater),
            code => return Err(ExtractSurfaceErrorKind::UnknownLayerType(code).into()),
        }
    }
}

impl LayerType {
    fn is_large_patch(&self) -> bool {
        match *self {
            LayerType::Land => false,
            _ => unimplemented!(), // TODO
        }
    }
}

/// One patch of a region's heightmap.
///
/// A region's heightmap is split into many square shaped patches.
#[derive(Debug)]
pub struct Patch {
    /// Side length of the square shape patch.
    size: u32,

    /// (x,y) index of patch in grid.
    patch_pos: (u32, u32),

    /// Decoded height map, square matrix of size `size`x`size`.
    /// TODO: (x,y)<->(i,j) ?
    data: DMatrix<f32>,
}

impl Patch {
    /// Side length of the square shape patch.
    ///
    /// This is both the number of values per direction, and the side length in meters of the
    /// patch, as there is one elevation value per meter.
    pub fn side_length(&self) -> u32 {
        self.size
    }

    /// Patch position (index, not meters) in the region.
    pub fn patch_position(&self) -> (u32, u32) {
        self.patch_pos.clone()
    }

    pub fn data(&self) -> &DMatrix<f32> {
        &self.data
    }
}

pub fn extract_land_patch(msg: &LayerData) -> Result<Vec<Patch>, ExtractSurfaceError> {
    let layer_type = LayerType::from_code(msg.layer_id.type_)?;
    extractor::extract_land_patches(&msg.layer_data.data[..], layer_type)
}

//! Spatial configuration validation

use crate::SpatialError;
use max2sc_core::Position3D;

pub fn validate_speaker_array(speakers: &[Position3D]) -> Result<(), SpatialError> {
    todo!("Implement speaker array validation")
}

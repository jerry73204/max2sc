//! Common types for max2sc

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SphericalCoord {
    pub azimuth: f32,
    pub elevation: f32,
    pub distance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioFormat {
    Mono,
    Stereo,
    Multichannel(u32),
    Ambisonic { order: u32, dimension: u32 },
}

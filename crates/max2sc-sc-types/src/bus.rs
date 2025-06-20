//! Bus and routing definitions

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BusConfig {
    pub audio_buses: Vec<AudioBus>,
    pub control_buses: Vec<ControlBus>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioBus {
    pub index: u32,
    pub num_channels: u32,
    pub name: Option<String>,
    pub private: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ControlBus {
    pub index: u32,
    pub name: Option<String>,
    pub default_value: f32,
}

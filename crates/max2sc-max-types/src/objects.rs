//! Max object type definitions

use crate::BoxContent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Spat5Object {
    #[serde(flatten)]
    pub base: BoxContent,
    pub spatial_params: SpatialParams,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpatialParams {
    pub sources: Option<u32>,
    pub speakers: Option<u32>,
    pub dimensions: Option<u32>,
    pub order: Option<u32>,
}

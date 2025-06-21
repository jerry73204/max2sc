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

/// Extract spatial parameters from a Max object's attributes
pub fn extract_spatial_params(content: &BoxContent) -> Option<SpatialParams> {
    // Look for common SPAT5 attributes
    let sources = content
        .attributes
        .get("sources")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);

    let speakers = content
        .attributes
        .get("speakers")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);

    let dimensions = content
        .attributes
        .get("dimensions")
        .or_else(|| content.attributes.get("dim"))
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);

    let order = content
        .attributes
        .get("order")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);

    if sources.is_some() || speakers.is_some() || dimensions.is_some() || order.is_some() {
        Some(SpatialParams {
            sources,
            speakers,
            dimensions,
            order,
        })
    } else {
        None
    }
}

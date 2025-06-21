//! Spatial audio object converters (pan~, etc.)

use max2sc_core::{Result, SCObject, SCValue};
use max2sc_max_types::BoxContent;

/// Converter for spatial audio objects
pub struct SpatialConverter;

impl SpatialConverter {
    /// Convert a spatial Max object to SuperCollider
    pub fn convert(content: &BoxContent) -> Result<SCObject> {
        let text = content.text.as_deref().unwrap_or("");

        // Parse object name and arguments
        let parts: Vec<&str> = text.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(SCObject::new("Unknown"));
        }

        let obj_name = parts[0];
        let args = &parts[1..];

        match obj_name {
            "pan~" => Self::convert_pan(args),
            "pan2~" => Self::convert_pan2(args),
            "pan4~" => Self::convert_pan4(args),
            "pan8~" => Self::convert_pan8(args),
            "stereo~" => Self::convert_stereo(),
            "matrix~" => Self::convert_matrix(args, content),
            _ if obj_name.starts_with("spat5") => Self::convert_spat5_basic(obj_name, args),
            _ => Ok(SCObject::new("UnknownSpatial").arg(obj_name)),
        }
    }

    /// Convert pan~ to Pan2.ar
    fn convert_pan(args: &[&str]) -> Result<SCObject> {
        // pan~ is stereo panning with position 0-1 (left to right)
        let default_pos = args
            .first()
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(0.5);

        // Convert Max pan position (0-1) to SC position (-1 to 1)
        let sc_pos = (default_pos * 2.0) - 1.0;

        Ok(SCObject::new("Pan2")
            .with_method("ar")
            .arg(SCValue::Symbol("input".to_string()))
            .arg(sc_pos)
            .arg(1.0) // level
            .prop("comment", "pan~"))
    }

    /// Convert pan2~ to Pan2.ar (alias for pan~)
    fn convert_pan2(args: &[&str]) -> Result<SCObject> {
        Self::convert_pan(args)
    }

    /// Convert pan4~ to Pan4.ar
    fn convert_pan4(args: &[&str]) -> Result<SCObject> {
        // pan4~ is quad panning with X/Y positions
        let x_pos = args
            .get(0)
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(0.0);
        let y_pos = args
            .get(1)
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(0.0);

        Ok(SCObject::new("Pan4")
            .with_method("ar")
            .arg(SCValue::Symbol("input".to_string()))
            .arg(x_pos)
            .arg(y_pos)
            .arg(1.0) // gain
            .prop("comment", "pan4~"))
    }

    /// Convert pan8~ to PanAz.ar with 8 channels
    fn convert_pan8(args: &[&str]) -> Result<SCObject> {
        // pan8~ is 8-channel circular panning
        let pos = args
            .first()
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(0.0);

        // Convert position to azimuth (0-2)
        let azimuth = pos * 2.0;

        Ok(SCObject::new("PanAz")
            .with_method("ar")
            .arg(8) // numChans
            .arg(SCValue::Symbol("input".to_string()))
            .arg(azimuth)
            .arg(1.0) // level
            .arg(2.0) // width
            .arg(0) // orientation
            .prop("comment", "pan8~"))
    }

    /// Convert stereo~ object
    fn convert_stereo() -> Result<SCObject> {
        // stereo~ usually just passes stereo signals through
        Ok(SCObject::new("Array")
            .arg(SCValue::Symbol("inputL".to_string()))
            .arg(SCValue::Symbol("inputR".to_string()))
            .prop("comment", "stereo~"))
    }

    /// Convert matrix~ for multichannel routing
    fn convert_matrix(args: &[&str], content: &BoxContent) -> Result<SCObject> {
        // matrix~ is a routing matrix
        let num_ins = args
            .get(0)
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(content.numinlets as i32);
        let num_outs = args
            .get(1)
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(content.numoutlets as i32);

        Ok(SCObject::new("Matrix")
            .with_method("ar")
            .arg(num_ins)
            .arg(num_outs)
            .arg(SCValue::Symbol("input".to_string()))
            .prop("comment", format!("matrix~ {}x{}", num_ins, num_outs)))
    }

    /// Basic SPAT5 object conversion (placeholder for now)
    fn convert_spat5_basic(obj_name: &str, _args: &[&str]) -> Result<SCObject> {
        match obj_name {
            "spat5.pan~" => {
                // SPAT5 panning - use VBAP for now
                Ok(SCObject::new("VBAP")
                    .with_method("ar")
                    .arg(8) // numChans (default)
                    .arg(SCValue::Symbol("input".to_string()))
                    .arg(SCValue::Symbol("azimuth".to_string()))
                    .arg(SCValue::Symbol("elevation".to_string()))
                    .arg(SCValue::Symbol("spread".to_string()))
                    .prop("comment", "spat5.pan~"))
            }
            "spat5.stereo~" => {
                // SPAT5 stereo processing
                Ok(SCObject::new("Splay")
                    .with_method("ar")
                    .arg(SCValue::Symbol("input".to_string()))
                    .arg(1.0) // spread
                    .arg(1.0) // level
                    .arg(0.0) // center
                    .prop("comment", "spat5.stereo~"))
            }
            _ => {
                // Generic SPAT5 placeholder
                Ok(SCObject::new("SPAT5_Placeholder")
                    .arg(obj_name)
                    .prop("comment", format!("{} - needs implementation", obj_name)))
            }
        }
    }
}

/// Helper to convert Max pan position to SC pan position
fn max_to_sc_pan(max_pos: f32) -> f32 {
    // Max: 0 = left, 1 = right
    // SC: -1 = left, 1 = right
    (max_pos * 2.0) - 1.0
}

/// Helper to convert degrees to radians
fn deg_to_rad(deg: f32) -> f32 {
    deg * std::f32::consts::PI / 180.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pan_conversion() {
        let content = BoxContent {
            id: "obj-1".to_string(),
            maxclass: "newobj".to_string(),
            text: Some("pan~ 0.7".to_string()),
            numinlets: 2,
            numoutlets: 2,
            patching_rect: None,
            outlettype: None,
            parameter_enable: None,
            attributes: Default::default(),
        };

        let result = SpatialConverter::convert(&content);
        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.class_name, "Pan2");
        assert_eq!(obj.method, Some("ar".to_string()));
    }

    #[test]
    fn test_pan4_conversion() {
        let content = BoxContent {
            id: "obj-1".to_string(),
            maxclass: "newobj".to_string(),
            text: Some("pan4~ 0.5 0.5".to_string()),
            numinlets: 3,
            numoutlets: 4,
            patching_rect: None,
            outlettype: None,
            parameter_enable: None,
            attributes: Default::default(),
        };

        let result = SpatialConverter::convert(&content);
        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.class_name, "Pan4");
    }

    #[test]
    fn test_spat5_pan_conversion() {
        let content = BoxContent {
            id: "obj-1".to_string(),
            maxclass: "newobj".to_string(),
            text: Some("spat5.pan~".to_string()),
            numinlets: 1,
            numoutlets: 8,
            patching_rect: None,
            outlettype: None,
            parameter_enable: None,
            attributes: Default::default(),
        };

        let result = SpatialConverter::convert(&content);
        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.class_name, "VBAP");
    }
}

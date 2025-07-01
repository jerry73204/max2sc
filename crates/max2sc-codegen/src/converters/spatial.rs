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
            .first()
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
            .first()
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
            .prop("comment", format!("matrix~ {num_ins}x{num_outs}")))
    }

    /// SPAT5 object conversion with detailed implementations
    fn convert_spat5_basic(obj_name: &str, args: &[&str]) -> Result<SCObject> {
        match obj_name {
            "spat5.panoramix~" => Self::convert_spat5_panoramix(args),
            "spat5.pan~" => Self::convert_spat5_pan(args),
            "spat5.stereo~" => Self::convert_spat5_stereo(args),
            "spat5.hoa.encoder~" => Self::convert_spat5_hoa_encoder(args),
            "spat5.hoa.decoder~" => Self::convert_spat5_hoa_decoder(args),
            "spat5.hoa.rotate~" => Self::convert_spat5_hoa_rotate(args),
            "spat5.vbap~" => Self::convert_spat5_vbap(args),
            "spat5.reverb~" => Self::convert_spat5_reverb(args),
            "spat5.early~" => Self::convert_spat5_early_reflections(args),
            _ => {
                // Generic SPAT5 placeholder for unimplemented objects
                Ok(SCObject::new("SPAT5_Placeholder")
                    .arg(obj_name)
                    .prop("comment", format!("{obj_name} - needs implementation")))
            }
        }
    }

    /// Convert spat5.panoramix~ - the main SPAT5 spatialization engine
    fn convert_spat5_panoramix(args: &[&str]) -> Result<SCObject> {
        // Parse arguments: numInputs, numOutputs, room model, etc.
        let num_inputs = args
            .first()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(1);
        let num_outputs = args.get(1).and_then(|s| s.parse::<i32>().ok()).unwrap_or(8);

        // spat5.panoramix~ is a complex multichannel spatializer
        // We'll implement it as a modular system with multiple components
        Ok(SCObject::new("SpatPanoramix")
            .with_method("ar")
            .arg(SCValue::Symbol("input".to_string()))
            .arg(num_inputs)
            .arg(num_outputs)
            .arg(SCValue::Array(vec![
                SCValue::Symbol("azimuth".to_string()),
                SCValue::Symbol("elevation".to_string()),
                SCValue::Symbol("distance".to_string()),
            ]))
            .prop("format", "VBAP") // Default format, can be changed via OSC
            .prop("room_model", "basic")
            .prop("reverb_enable", true)
            .prop("early_reflections", true)
            .prop("comment", "spat5.panoramix~ - main spatialization engine"))
    }

    /// Convert spat5.pan~ - flexible panning object
    fn convert_spat5_pan(args: &[&str]) -> Result<SCObject> {
        let num_outputs = args
            .first()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(8);

        Ok(SCObject::new("VBAP")
            .with_method("ar")
            .arg(num_outputs)
            .arg(SCValue::Symbol("input".to_string()))
            .arg(SCValue::Symbol("azimuth".to_string()))
            .arg(SCValue::Symbol("elevation".to_string()))
            .arg(SCValue::Symbol("spread".to_string()))
            .prop("comment", "spat5.pan~"))
    }

    /// Convert spat5.stereo~ - stereo processing
    fn convert_spat5_stereo(_args: &[&str]) -> Result<SCObject> {
        Ok(SCObject::new("Splay")
            .with_method("ar")
            .arg(SCValue::Symbol("input".to_string()))
            .arg(1.0) // spread
            .arg(1.0) // level
            .arg(0.0) // center
            .prop("comment", "spat5.stereo~"))
    }

    /// Convert spat5.hoa.encoder~ - Higher Order Ambisonics encoder
    fn convert_spat5_hoa_encoder(args: &[&str]) -> Result<SCObject> {
        let order = args
            .first()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(1); // Default to first order

        match order {
            1 => {
                // First Order Ambisonics - use ATK
                Ok(SCObject::new("FoaEncode")
                    .with_method("ar")
                    .arg(SCValue::Symbol("input".to_string()))
                    .arg(SCValue::Symbol("azimuth".to_string()))
                    .arg(SCValue::Symbol("elevation".to_string()))
                    .prop("encoder_type", "omni")
                    .prop("comment", "spat5.hoa.encoder~ (FOA)"))
            }
            _ => {
                // Higher order - use HoaLib if available
                Ok(SCObject::new("HoaEncodeMatrix")
                    .with_method("ar")
                    .arg(order)
                    .arg(SCValue::Symbol("input".to_string()))
                    .arg(SCValue::Symbol("azimuth".to_string()))
                    .arg(SCValue::Symbol("elevation".to_string()))
                    .prop("comment", format!("spat5.hoa.encoder~ (order {order})")))
            }
        }
    }

    /// Convert spat5.hoa.decoder~ - HOA decoder
    fn convert_spat5_hoa_decoder(args: &[&str]) -> Result<SCObject> {
        let order = args
            .first()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(1);
        let num_speakers = args.get(1).and_then(|s| s.parse::<i32>().ok()).unwrap_or(8);

        match order {
            1 => {
                // First Order Ambisonics
                Ok(SCObject::new("FoaDecode")
                    .with_method("ar")
                    .arg(SCValue::Symbol("encoded_input".to_string()))
                    .arg(SCValue::Symbol("decoder_matrix".to_string()))
                    .prop("num_speakers", num_speakers)
                    .prop("comment", "spat5.hoa.decoder~ (FOA)"))
            }
            _ => {
                // Higher order
                Ok(SCObject::new("HoaDecodeMatrix")
                    .with_method("ar")
                    .arg(order)
                    .arg(num_speakers)
                    .arg(SCValue::Symbol("encoded_input".to_string()))
                    .prop("comment", format!("spat5.hoa.decoder~ (order {order})")))
            }
        }
    }

    /// Convert spat5.hoa.rotate~ - HOA rotation
    fn convert_spat5_hoa_rotate(args: &[&str]) -> Result<SCObject> {
        let order = args
            .first()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(1);

        match order {
            1 => Ok(SCObject::new("FoaRotate")
                .with_method("ar")
                .arg(SCValue::Symbol("encoded_input".to_string()))
                .arg(SCValue::Symbol("azimuth".to_string()))
                .arg(SCValue::Symbol("elevation".to_string()))
                .arg(SCValue::Symbol("roll".to_string()))
                .prop("comment", "spat5.hoa.rotate~ (FOA)")),
            _ => Ok(SCObject::new("HoaRotate")
                .with_method("ar")
                .arg(order)
                .arg(SCValue::Symbol("encoded_input".to_string()))
                .arg(SCValue::Symbol("azimuth".to_string()))
                .arg(SCValue::Symbol("elevation".to_string()))
                .prop("comment", format!("spat5.hoa.rotate~ (order {order})"))),
        }
    }

    /// Convert spat5.vbap~ - Vector Based Amplitude Panning
    fn convert_spat5_vbap(args: &[&str]) -> Result<SCObject> {
        let num_speakers = args
            .first()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(8);

        Ok(SCObject::new("VBAP")
            .with_method("ar")
            .arg(num_speakers)
            .arg(SCValue::Symbol("input".to_string()))
            .arg(SCValue::Symbol("azimuth".to_string()))
            .arg(SCValue::Symbol("elevation".to_string()))
            .arg(SCValue::Symbol("spread".to_string()))
            .prop("speaker_setup", "ring")
            .prop("comment", "spat5.vbap~"))
    }

    /// Convert spat5.reverb~ - spatial reverb
    fn convert_spat5_reverb(args: &[&str]) -> Result<SCObject> {
        let num_outputs = args
            .first()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(2);

        Ok(SCObject::new("JPverb")
            .with_method("ar")
            .arg(SCValue::Symbol("input".to_string()))
            .arg(SCValue::Symbol("rt60".to_string()))
            .arg(SCValue::Symbol("damping".to_string()))
            .arg(SCValue::Symbol("size".to_string()))
            .arg(SCValue::Symbol("early_diff".to_string()))
            .arg(SCValue::Symbol("mod_depth".to_string()))
            .arg(SCValue::Symbol("mod_freq".to_string()))
            .arg(SCValue::Symbol("low".to_string()))
            .arg(SCValue::Symbol("mid".to_string()))
            .arg(SCValue::Symbol("high".to_string()))
            .arg(SCValue::Symbol("hf_damping".to_string()))
            .prop("num_outputs", num_outputs)
            .prop("comment", "spat5.reverb~"))
    }

    /// Convert spat5.early~ - early reflections
    fn convert_spat5_early_reflections(args: &[&str]) -> Result<SCObject> {
        let num_taps = args
            .first()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(8);

        Ok(SCObject::new("EarlyReflections")
            .with_method("ar")
            .arg(SCValue::Symbol("input".to_string()))
            .arg(num_taps)
            .arg(SCValue::Symbol("room_size".to_string()))
            .arg(SCValue::Symbol("damping".to_string()))
            .arg(SCValue::Array(vec![
                SCValue::Symbol("delay_times".to_string()),
                SCValue::Symbol("gains".to_string()),
                SCValue::Symbol("pan_positions".to_string()),
            ]))
            .prop("comment", "spat5.early~ - early reflections"))
    }
}

/// Helper to convert Max pan position to SC pan position
fn _max_to_sc_pan(max_pos: f32) -> f32 {
    // Max: 0 = left, 1 = right
    // SC: -1 = left, 1 = right
    (max_pos * 2.0) - 1.0
}

/// Helper to convert degrees to radians
fn _deg_to_rad(deg: f32) -> f32 {
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

    #[test]
    fn test_spat5_panoramix_conversion() {
        let content = BoxContent {
            id: "obj-1".to_string(),
            maxclass: "newobj".to_string(),
            text: Some("spat5.panoramix~ 4 16".to_string()),
            numinlets: 4,
            numoutlets: 16,
            patching_rect: None,
            outlettype: None,
            parameter_enable: None,
            attributes: Default::default(),
        };

        let result = SpatialConverter::convert(&content);
        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.class_name, "SpatPanoramix");
        assert_eq!(obj.method, Some("ar".to_string()));
        assert!(obj.args.len() >= 4); // input, num_inputs, num_outputs, positions
    }

    #[test]
    fn test_spat5_hoa_encoder_conversion() {
        let content = BoxContent {
            id: "obj-1".to_string(),
            maxclass: "newobj".to_string(),
            text: Some("spat5.hoa.encoder~ 1".to_string()),
            numinlets: 1,
            numoutlets: 4,
            patching_rect: None,
            outlettype: None,
            parameter_enable: None,
            attributes: Default::default(),
        };

        let result = SpatialConverter::convert(&content);
        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.class_name, "FoaEncode"); // First order uses FOA
    }

    #[test]
    fn test_spat5_hoa_decoder_conversion() {
        let content = BoxContent {
            id: "obj-1".to_string(),
            maxclass: "newobj".to_string(),
            text: Some("spat5.hoa.decoder~ 1 8".to_string()),
            numinlets: 4,
            numoutlets: 8,
            patching_rect: None,
            outlettype: None,
            parameter_enable: None,
            attributes: Default::default(),
        };

        let result = SpatialConverter::convert(&content);
        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.class_name, "FoaDecode"); // First order uses FOA
    }

    #[test]
    fn test_spat5_vbap_conversion() {
        let content = BoxContent {
            id: "obj-1".to_string(),
            maxclass: "newobj".to_string(),
            text: Some("spat5.vbap~ 8".to_string()),
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
        assert_eq!(obj.method, Some("ar".to_string()));
    }
}

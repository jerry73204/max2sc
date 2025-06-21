//! Audio I/O object converters (dac~, adc~, etc.)

use max2sc_core::{Result, SCObject, SCValue};
use max2sc_max_types::BoxContent;

/// Converter for audio I/O objects
pub struct AudioIOConverter;

impl AudioIOConverter {
    /// Convert an audio I/O Max object to SuperCollider
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
            "dac~" => Self::convert_dac(args),
            "adc~" => Self::convert_adc(args),
            "ezdac~" => Self::convert_ezdac(),
            "ezadc~" => Self::convert_ezadc(),
            "out~" => Self::convert_out(args),
            "in~" => Self::convert_in(args),
            _ => Ok(SCObject::new("UnknownIO").arg(obj_name)),
        }
    }

    /// Convert dac~ to Out.ar
    fn convert_dac(args: &[&str]) -> Result<SCObject> {
        // dac~ outputs to specified channels
        let mut channels = Vec::new();

        for arg in args {
            if let Ok(ch) = arg.parse::<i32>() {
                channels.push(ch - 1); // Max uses 1-based, SC uses 0-based
            }
        }

        // Default to stereo output if no channels specified
        if channels.is_empty() {
            channels = vec![0, 1];
        }

        if channels.len() == 1 {
            // Single channel output
            Ok(SCObject::new("Out")
                .with_method("ar")
                .arg(channels[0])
                .arg(SCValue::Symbol("input".to_string())))
        } else if channels.len() == 2 && channels[0] == 0 && channels[1] == 1 {
            // Stereo output (most common case)
            Ok(SCObject::new("Out")
                .with_method("ar")
                .arg(0)
                .arg(SCValue::Array(vec![
                    SCValue::Symbol("inputL".to_string()),
                    SCValue::Symbol("inputR".to_string()),
                ])))
        } else {
            // Multiple arbitrary channels
            let channel_array =
                SCValue::Array(channels.iter().map(|&ch| SCValue::Int(ch)).collect());

            Ok(SCObject::new("Out")
                .with_method("ar")
                .arg(channel_array)
                .arg(SCValue::Symbol("input".to_string())))
        }
    }

    /// Convert adc~ to In.ar/SoundIn.ar
    fn convert_adc(args: &[&str]) -> Result<SCObject> {
        // adc~ inputs from specified channels
        let mut channels = Vec::new();

        for arg in args {
            if let Ok(ch) = arg.parse::<i32>() {
                channels.push(ch - 1); // Max uses 1-based, SC uses 0-based
            }
        }

        // Default to stereo input if no channels specified
        if channels.is_empty() {
            channels = vec![0, 1];
        }

        if channels.len() == 1 {
            // Single channel input
            Ok(SCObject::new("SoundIn").with_method("ar").arg(channels[0]))
        } else if channels.len() == 2 && channels[0] == 0 && channels[1] == 1 {
            // Stereo input (most common case)
            Ok(SCObject::new("SoundIn")
                .with_method("ar")
                .arg(SCValue::Array(vec![SCValue::Int(0), SCValue::Int(1)])))
        } else {
            // Multiple arbitrary channels
            let channel_array =
                SCValue::Array(channels.iter().map(|&ch| SCValue::Int(ch)).collect());

            Ok(SCObject::new("SoundIn")
                .with_method("ar")
                .arg(channel_array))
        }
    }

    /// Convert ezdac~ to simple stereo output
    fn convert_ezdac() -> Result<SCObject> {
        Ok(SCObject::new("Out")
            .with_method("ar")
            .arg(0)
            .arg(SCValue::Array(vec![
                SCValue::Symbol("inputL".to_string()),
                SCValue::Symbol("inputR".to_string()),
            ]))
            .prop("comment", "ezdac~ - simple stereo output"))
    }

    /// Convert ezadc~ to simple stereo input
    fn convert_ezadc() -> Result<SCObject> {
        Ok(SCObject::new("SoundIn")
            .with_method("ar")
            .arg(SCValue::Array(vec![SCValue::Int(0), SCValue::Int(1)]))
            .prop("comment", "ezadc~ - simple stereo input"))
    }

    /// Convert out~ (outlet) - used in subpatchers
    fn convert_out(args: &[&str]) -> Result<SCObject> {
        let outlet_num = args
            .first()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(1);

        Ok(SCObject::new("Out")
            .arg(outlet_num - 1)
            .arg(SCValue::Symbol("signal".to_string()))
            .prop("comment", format!("out~ {}", outlet_num)))
    }

    /// Convert in~ (inlet) - used in subpatchers
    fn convert_in(args: &[&str]) -> Result<SCObject> {
        let inlet_num = args
            .first()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(1);

        Ok(SCObject::new("In")
            .arg(inlet_num - 1)
            .prop("comment", format!("in~ {}", inlet_num)))
    }
}

/// Helper function to parse channel lists from Max format
fn parse_channel_list(text: &str) -> Vec<i32> {
    let mut channels = Vec::new();
    let parts: Vec<&str> = text.split_whitespace().collect();

    // Skip the object name
    for part in parts.iter().skip(1) {
        if let Ok(ch) = part.parse::<i32>() {
            channels.push(ch - 1); // Convert to 0-based
        }
    }

    channels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dac_stereo_conversion() {
        let content = BoxContent {
            id: "obj-1".to_string(),
            maxclass: "newobj".to_string(),
            text: Some("dac~".to_string()),
            numinlets: 2,
            numoutlets: 0,
            patching_rect: None,
            outlettype: None,
            parameter_enable: None,
            attributes: Default::default(),
        };

        let result = AudioIOConverter::convert(&content);
        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.class_name, "Out");
        assert_eq!(obj.method, Some("ar".to_string()));
    }

    #[test]
    fn test_adc_multichannel_conversion() {
        let content = BoxContent {
            id: "obj-1".to_string(),
            maxclass: "newobj".to_string(),
            text: Some("adc~ 1 2 3 4".to_string()),
            numinlets: 0,
            numoutlets: 4,
            patching_rect: None,
            outlettype: None,
            parameter_enable: None,
            attributes: Default::default(),
        };

        let result = AudioIOConverter::convert(&content);
        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.class_name, "SoundIn");
        assert_eq!(obj.method, Some("ar".to_string()));
    }
}

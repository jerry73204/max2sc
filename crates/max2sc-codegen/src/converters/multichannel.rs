//! Multichannel object converters (mc.*)

use max2sc_core::{Result, SCObject, SCValue};
use max2sc_max_types::BoxContent;

/// Converter for multichannel objects
pub struct MultichannelConverter;

impl MultichannelConverter {
    /// Convert a multichannel Max object to SuperCollider
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
            "mc.pack~" => Self::convert_mc_pack(args),
            "mc.unpack~" => Self::convert_mc_unpack(args),
            "mc.dac~" => Self::convert_mc_dac(args, content),
            "mc.adc~" => Self::convert_mc_adc(args, content),
            "mc.live.gain~" => Self::convert_mc_live_gain(args),
            _ => {
                // Generic multichannel object handling
                Ok(SCObject::new("UnknownMC").arg(obj_name))
            }
        }
    }

    /// Convert mc.pack~ to SC array construction
    fn convert_mc_pack(args: &[&str]) -> Result<SCObject> {
        // mc.pack~ combines multiple signals into a multichannel signal
        // In SC, this is just array construction
        let num_channels = args
            .first()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(2);

        Ok(SCObject::new("Array").prop("comment", format!("mc.pack~ {num_channels} channels")))
    }

    /// Convert mc.unpack~ to SC array indexing
    fn convert_mc_unpack(args: &[&str]) -> Result<SCObject> {
        // mc.unpack~ extracts channels from multichannel signal
        // In SC, this is array indexing
        let num_channels = args
            .first()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(2);

        Ok(SCObject::new("ArrayIndex")
            .prop("comment", format!("mc.unpack~ {num_channels} channels")))
    }

    /// Convert mc.dac~ to multichannel output
    fn convert_mc_dac(args: &[&str], content: &BoxContent) -> Result<SCObject> {
        // mc.dac~ outputs to multiple channels
        // Get channel numbers from arguments or use defaults
        let mut channels = Vec::new();

        for arg in args {
            if let Ok(ch) = arg.parse::<i32>() {
                channels.push(ch - 1); // Max uses 1-based, SC uses 0-based
            }
        }

        // If no channels specified, use number of inlets
        if channels.is_empty() {
            let num_channels = content.numinlets as i32;
            channels = (0..num_channels).collect();
        }

        let channel_array = SCValue::Array(channels.iter().map(|&ch| SCValue::Int(ch)).collect());

        Ok(SCObject::new("Out")
            .with_method("ar")
            .arg(channel_array)
            .arg(SCValue::Symbol("input".to_string())))
    }

    /// Convert mc.adc~ to multichannel input
    fn convert_mc_adc(args: &[&str], content: &BoxContent) -> Result<SCObject> {
        // mc.adc~ inputs from multiple channels
        let mut channels = Vec::new();

        for arg in args {
            if let Ok(ch) = arg.parse::<i32>() {
                channels.push(ch - 1); // Max uses 1-based, SC uses 0-based
            }
        }

        // If no channels specified, use number of outlets
        if channels.is_empty() {
            let num_channels = content.numoutlets as i32;
            channels = (0..num_channels).collect();
        }

        let channel_array = SCValue::Array(channels.iter().map(|&ch| SCValue::Int(ch)).collect());

        Ok(SCObject::new("In").with_method("ar").arg(channel_array))
    }

    /// Convert mc.live.gain~ to multichannel gain control
    fn convert_mc_live_gain(args: &[&str]) -> Result<SCObject> {
        // mc.live.gain~ is a multichannel gain control
        // In SC, we multiply the signal array by a gain value
        let default_gain = args
            .first()
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(0.0); // dB

        // Convert dB to linear
        let linear_gain = 10.0_f32.powf(default_gain / 20.0);

        Ok(SCObject::new("*")
            .arg(SCValue::Symbol("input".to_string()))
            .arg(linear_gain)
            .prop("lag", 0.1) // Smooth gain changes
            .prop("comment", "mc.live.gain~"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mc_pack_conversion() {
        let content = BoxContent {
            id: "obj-1".to_string(),
            maxclass: "newobj".to_string(),
            text: Some("mc.pack~ 4".to_string()),
            numinlets: 4,
            numoutlets: 1,
            patching_rect: None,
            outlettype: None,
            parameter_enable: None,
            attributes: Default::default(),
        };

        let result = MultichannelConverter::convert(&content);
        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.class_name, "Array");
    }

    #[test]
    fn test_mc_dac_conversion() {
        let content = BoxContent {
            id: "obj-1".to_string(),
            maxclass: "newobj".to_string(),
            text: Some("mc.dac~ 1 2 3 4".to_string()),
            numinlets: 4,
            numoutlets: 0,
            patching_rect: None,
            outlettype: None,
            parameter_enable: None,
            attributes: Default::default(),
        };

        let result = MultichannelConverter::convert(&content);
        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.class_name, "Out");
        assert_eq!(obj.method, Some("ar".to_string()));
    }
}

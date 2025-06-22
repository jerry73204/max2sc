//! Higher Order Ambisonics (HOA) converter using ATK integration

use max2sc_analyzer::spatial_analysis::SpeakerArray;
use max2sc_core::{ConversionError, Result, SCObject, SCValue};

/// HOA converter for SuperCollider using ATK (Ambisonic Toolkit)
pub struct HoaConverter;

impl HoaConverter {
    /// Generate HOA encoder for given order and dimension
    pub fn generate_hoa_encoder(order: u32, dimension: u32) -> Result<SCObject> {
        match (order, dimension) {
            (1, 2) => Self::generate_foa_2d_encoder(),
            (1, 3) => Self::generate_foa_3d_encoder(),
            (order, 3) => Self::generate_hoa_3d_encoder(order),
            _ => Err(ConversionError::UnsupportedObject(format!(
                "Unsupported HOA configuration: order {}, dimension {}",
                order, dimension
            ))),
        }
    }

    /// Generate First Order Ambisonics (FOA) 2D encoder
    fn generate_foa_2d_encoder() -> Result<SCObject> {
        Ok(SCObject::new("FoaEncode")
            .with_method("ar")
            .arg(SCValue::Symbol("input".to_string()))
            .arg(SCValue::Symbol("azimuth".to_string()))
            .prop("encoder_type", "omni")
            .prop("dimension", "2D")
            .prop("order", 1)
            .prop("channels", 3) // B-format: W, X, Y
            .prop("comment", "FOA 2D encoder"))
    }

    /// Generate First Order Ambisonics (FOA) 3D encoder
    fn generate_foa_3d_encoder() -> Result<SCObject> {
        Ok(SCObject::new("FoaEncode")
            .with_method("ar")
            .arg(SCValue::Symbol("input".to_string()))
            .arg(SCValue::Symbol("azimuth".to_string()))
            .arg(SCValue::Symbol("elevation".to_string()))
            .prop("encoder_type", "omni")
            .prop("dimension", "3D")
            .prop("order", 1)
            .prop("channels", 4) // B-format: W, X, Y, Z
            .prop("comment", "FOA 3D encoder"))
    }

    /// Generate Higher Order Ambisonics encoder
    fn generate_hoa_3d_encoder(order: u32) -> Result<SCObject> {
        let num_channels = (order + 1).pow(2); // (N+1)² channels for 3D HOA

        Ok(SCObject::new("HoaEncode")
            .with_method("ar")
            .arg(order)
            .arg(SCValue::Symbol("input".to_string()))
            .arg(SCValue::Symbol("azimuth".to_string()))
            .arg(SCValue::Symbol("elevation".to_string()))
            .prop("dimension", "3D")
            .prop("order", order)
            .prop("channels", num_channels)
            .prop(
                "comment",
                format!("HOA 3D encoder, order {}, {} channels", order, num_channels),
            ))
    }

    /// Generate HOA decoder for speaker array
    pub fn generate_hoa_decoder(
        order: u32,
        array: &SpeakerArray,
        decoder_type: HoaDecoderType,
    ) -> Result<SCObject> {
        let num_speakers = array.speakers.len() as u32;

        match order {
            1 => Self::generate_foa_decoder(array, decoder_type),
            _ => Self::generate_hoa_decoder_matrix(order, num_speakers, decoder_type),
        }
    }

    /// Generate First Order Ambisonics decoder
    fn generate_foa_decoder(
        array: &SpeakerArray,
        decoder_type: HoaDecoderType,
    ) -> Result<SCObject> {
        let num_speakers = array.speakers.len() as i32;
        let decoder_method = match decoder_type {
            HoaDecoderType::Basic => "basic",
            HoaDecoderType::MaxRe => "maxRe",
            HoaDecoderType::InPhase => "inPhase",
            HoaDecoderType::Controlled => "controlled",
            HoaDecoderType::Binaural => "binaural",
        };

        // Generate speaker position matrix
        let speaker_positions = Self::generate_speaker_position_matrix(array);

        Ok(SCObject::new("FoaDecode")
            .with_method("ar")
            .arg(SCValue::Symbol("encoded_input".to_string()))
            .arg(SCValue::Symbol("decoder_matrix".to_string()))
            .prop("decoder_type", decoder_method)
            .prop("num_speakers", num_speakers)
            .prop("speaker_positions", speaker_positions)
            .prop("order", 1)
            .prop(
                "comment",
                format!(
                    "FOA decoder: {} method, {} speakers",
                    decoder_method, num_speakers
                ),
            ))
    }

    /// Generate Higher Order Ambisonics decoder
    fn generate_hoa_decoder_matrix(
        order: u32,
        num_speakers: u32,
        decoder_type: HoaDecoderType,
    ) -> Result<SCObject> {
        let num_channels = (order + 1).pow(2);
        let decoder_method = match decoder_type {
            HoaDecoderType::Basic => "basic",
            HoaDecoderType::MaxRe => "maxRe",
            HoaDecoderType::InPhase => "inPhase",
            HoaDecoderType::Controlled => "controlled",
            HoaDecoderType::Binaural => "binaural",
        };

        Ok(SCObject::new("HoaDecode")
            .with_method("ar")
            .arg(order)
            .arg(num_speakers)
            .arg(SCValue::Symbol("encoded_input".to_string()))
            .arg(SCValue::Symbol("decoder_matrix".to_string()))
            .prop("decoder_type", decoder_method)
            .prop("channels", num_channels)
            .prop(
                "comment",
                format!(
                    "HOA decoder: order {}, {} method, {} speakers",
                    order, decoder_method, num_speakers
                ),
            ))
    }

    /// Generate HOA rotation transform
    pub fn generate_hoa_rotation(order: u32) -> Result<SCObject> {
        match order {
            1 => Ok(SCObject::new("FoaRotate")
                .with_method("ar")
                .arg(SCValue::Symbol("encoded_input".to_string()))
                .arg(SCValue::Symbol("azimuth".to_string()))
                .arg(SCValue::Symbol("elevation".to_string()))
                .arg(SCValue::Symbol("roll".to_string()))
                .prop("order", 1)
                .prop("comment", "FOA rotation transform")),
            _ => Ok(SCObject::new("HoaRotate")
                .with_method("ar")
                .arg(order)
                .arg(SCValue::Symbol("encoded_input".to_string()))
                .arg(SCValue::Symbol("azimuth".to_string()))
                .arg(SCValue::Symbol("elevation".to_string()))
                .arg(SCValue::Symbol("roll".to_string()))
                .prop("order", order)
                .prop(
                    "comment",
                    format!("HOA rotation transform, order {}", order),
                )),
        }
    }

    /// Generate HOA mirror/reflection transform
    pub fn generate_hoa_mirror(order: u32, mirror_type: HoaMirrorType) -> Result<SCObject> {
        let mirror_axis = match mirror_type {
            HoaMirrorType::X => "x",
            HoaMirrorType::Y => "y",
            HoaMirrorType::Z => "z",
            HoaMirrorType::XY => "xy",
            HoaMirrorType::YZ => "yz",
            HoaMirrorType::XZ => "xz",
        };

        match order {
            1 => Ok(SCObject::new("FoaMirror")
                .with_method("ar")
                .arg(SCValue::Symbol("encoded_input".to_string()))
                .arg(mirror_axis)
                .prop("order", 1)
                .prop(
                    "comment",
                    format!("FOA mirror transform: {} axis", mirror_axis),
                )),
            _ => Ok(SCObject::new("HoaMirror")
                .with_method("ar")
                .arg(order)
                .arg(SCValue::Symbol("encoded_input".to_string()))
                .arg(mirror_axis)
                .prop("order", order)
                .prop(
                    "comment",
                    format!(
                        "HOA mirror transform: {} axis, order {}",
                        mirror_axis, order
                    ),
                )),
        }
    }

    /// Generate HOA focus/beamforming transform
    pub fn generate_hoa_focus(order: u32, focus_type: HoaFocusType) -> Result<SCObject> {
        let focus_method = match focus_type {
            HoaFocusType::Push => "push",
            HoaFocusType::Press => "press",
            HoaFocusType::Zoom => "zoom",
        };

        match order {
            1 => Ok(SCObject::new("FoaFocus")
                .with_method("ar")
                .arg(SCValue::Symbol("encoded_input".to_string()))
                .arg(SCValue::Symbol("azimuth".to_string()))
                .arg(SCValue::Symbol("elevation".to_string()))
                .arg(SCValue::Symbol("focus_amount".to_string()))
                .prop("focus_type", focus_method)
                .prop("order", 1)
                .prop("comment", format!("FOA focus transform: {}", focus_method))),
            _ => Ok(SCObject::new("HoaFocus")
                .with_method("ar")
                .arg(order)
                .arg(SCValue::Symbol("encoded_input".to_string()))
                .arg(SCValue::Symbol("azimuth".to_string()))
                .arg(SCValue::Symbol("elevation".to_string()))
                .arg(SCValue::Symbol("focus_amount".to_string()))
                .prop("focus_type", focus_method)
                .prop("order", order)
                .prop(
                    "comment",
                    format!("HOA focus transform: {}, order {}", focus_method, order),
                )),
        }
    }

    /// Generate HOA distance simulation
    pub fn generate_hoa_distance(order: u32) -> Result<SCObject> {
        match order {
            1 => Ok(SCObject::new("FoaNFC")
                .with_method("ar")
                .arg(SCValue::Symbol("encoded_input".to_string()))
                .arg(SCValue::Symbol("distance".to_string()))
                .prop("order", 1)
                .prop("comment", "FOA near-field compensation")),
            _ => Ok(SCObject::new("HoaNFC")
                .with_method("ar")
                .arg(order)
                .arg(SCValue::Symbol("encoded_input".to_string()))
                .arg(SCValue::Symbol("distance".to_string()))
                .prop("order", order)
                .prop(
                    "comment",
                    format!("HOA near-field compensation, order {}", order),
                )),
        }
    }

    /// Generate HOA format converter (e.g., FOA to HOA)
    pub fn generate_hoa_converter(from_order: u32, to_order: u32) -> Result<SCObject> {
        if from_order == to_order {
            return Ok(SCObject::new("Through")
                .with_method("ar")
                .arg(SCValue::Symbol("input".to_string()))
                .prop("comment", "Pass-through (same order)"));
        }

        Ok(SCObject::new("HoaConvert")
            .with_method("ar")
            .arg(from_order)
            .arg(to_order)
            .arg(SCValue::Symbol("encoded_input".to_string()))
            .prop("from_order", from_order)
            .prop("to_order", to_order)
            .prop(
                "comment",
                format!("HOA format converter: order {} to {}", from_order, to_order),
            ))
    }

    /// Generate binaural decoder for headphone output
    pub fn generate_binaural_decoder(order: u32, hrtf_type: HrtfType) -> Result<SCObject> {
        let hrtf_method = match hrtf_type {
            HrtfType::Diffuse => "diffuse",
            HrtfType::Spherical => "spherical",
            HrtfType::CIPIC => "cipic",
            HrtfType::Listen => "listen",
        };

        match order {
            1 => Ok(SCObject::new("FoaDecode")
                .with_method("ar")
                .arg(SCValue::Symbol("encoded_input".to_string()))
                .arg("binaural")
                .prop("hrtf_type", hrtf_method)
                .prop("order", 1)
                .prop("channels", 2)
                .prop("comment", format!("FOA binaural decoder: {}", hrtf_method))),
            _ => Ok(SCObject::new("HoaBinaural")
                .with_method("ar")
                .arg(order)
                .arg(SCValue::Symbol("encoded_input".to_string()))
                .prop("hrtf_type", hrtf_method)
                .prop("order", order)
                .prop("channels", 2)
                .prop(
                    "comment",
                    format!("HOA binaural decoder: {}, order {}", hrtf_method, order),
                )),
        }
    }

    /// Generate speaker position matrix for decoder
    fn generate_speaker_position_matrix(array: &SpeakerArray) -> SCValue {
        let positions: Vec<SCValue> = array
            .speakers
            .iter()
            .map(|speaker| {
                SCValue::Array(vec![
                    SCValue::Float(speaker.position.azimuth),
                    SCValue::Float(speaker.position.elevation),
                    SCValue::Float(speaker.position.distance),
                ])
            })
            .collect();

        SCValue::Array(positions)
    }

    /// Calculate optimal HOA order for given speaker array
    pub fn calculate_optimal_order(array: &SpeakerArray) -> u32 {
        let num_speakers = array.speakers.len() as u32;

        // Rough heuristic: order should be approximately sqrt(num_speakers/4)
        // For proper decoding, we need roughly (N+1)² speakers for order N
        let optimal_order = ((num_speakers as f32 / 4.0).sqrt() as u32).max(1);

        // Cap at reasonable maximum
        optimal_order.min(7)
    }

    /// Validate HOA configuration
    pub fn validate_hoa_config(order: u32, num_speakers: u32) -> HoaValidationResult {
        let min_speakers = (order + 1).pow(2);
        let recommended_speakers = min_speakers * 2;

        let mut result = HoaValidationResult {
            is_valid: num_speakers >= min_speakers,
            warnings: Vec::new(),
            errors: Vec::new(),
            recommended_order: Self::calculate_recommended_order(num_speakers),
        };

        if num_speakers < min_speakers {
            result.errors.push(format!(
                "Order {} requires at least {} speakers, but only {} available",
                order, min_speakers, num_speakers
            ));
        } else if num_speakers < recommended_speakers {
            result.warnings.push(format!(
                "Order {} works best with {} speakers, only {} available",
                order, recommended_speakers, num_speakers
            ));
        }

        result
    }

    fn calculate_recommended_order(num_speakers: u32) -> u32 {
        // Conservative estimate for good decoding quality
        ((num_speakers as f32 / 8.0).sqrt() as u32).max(1).min(5)
    }
}

/// HOA decoder types
#[derive(Debug, Clone, Copy)]
pub enum HoaDecoderType {
    Basic,
    MaxRe,      // Maximum rE decoding
    InPhase,    // In-phase decoding
    Controlled, // Controlled opposites
    Binaural,   // Binaural HRTF
}

/// HOA mirror/reflection types
#[derive(Debug, Clone, Copy)]
pub enum HoaMirrorType {
    X,
    Y,
    Z,
    XY,
    YZ,
    XZ,
}

/// HOA focus types
#[derive(Debug, Clone, Copy)]
pub enum HoaFocusType {
    Push,  // Push transform
    Press, // Press transform
    Zoom,  // Zoom transform
}

/// HRTF types for binaural decoding
#[derive(Debug, Clone, Copy)]
pub enum HrtfType {
    Diffuse,
    Spherical,
    CIPIC,
    Listen,
}

/// HOA validation result
#[derive(Debug)]
pub struct HoaValidationResult {
    pub is_valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub recommended_order: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use max2sc_analyzer::spatial_analysis::{Speaker, SpeakerArrayType};
    use max2sc_core::SphericalCoord;

    fn create_test_speaker_array() -> SpeakerArray {
        let speakers = (0..16)
            .map(|i| {
                Speaker {
                    id: i,
                    position: SphericalCoord {
                        azimuth: (i as f32 * 22.5) % 360.0,        // Every 22.5 degrees
                        elevation: if i < 8 { 0.0 } else { 30.0 }, // Two rings
                        distance: 3.0,
                    },
                    delay: 0.0,
                    gain: 1.0,
                }
            })
            .collect();

        SpeakerArray {
            id: "hoa_test".to_string(),
            array_type: SpeakerArrayType::Ring { radius: 3.0 },
            speakers,
            wfs_config: None,
        }
    }

    #[test]
    fn test_foa_encoder_generation() {
        let result = HoaConverter::generate_hoa_encoder(1, 3);

        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.class_name, "FoaEncode");
        assert_eq!(obj.method, Some("ar".to_string()));
    }

    #[test]
    fn test_hoa_encoder_generation() {
        let result = HoaConverter::generate_hoa_encoder(3, 3);

        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.class_name, "HoaEncode");
        assert_eq!(obj.method, Some("ar".to_string()));
    }

    #[test]
    fn test_foa_decoder_generation() {
        let array = create_test_speaker_array();
        let result = HoaConverter::generate_hoa_decoder(1, &array, HoaDecoderType::MaxRe);

        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.class_name, "FoaDecode");
        assert_eq!(obj.method, Some("ar".to_string()));
    }

    #[test]
    fn test_hoa_rotation_generation() {
        let result = HoaConverter::generate_hoa_rotation(2);

        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.class_name, "HoaRotate");
        assert_eq!(obj.method, Some("ar".to_string()));
    }

    #[test]
    fn test_hoa_focus_generation() {
        let result = HoaConverter::generate_hoa_focus(1, HoaFocusType::Push);

        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.class_name, "FoaFocus");
        assert_eq!(obj.method, Some("ar".to_string()));
    }

    #[test]
    fn test_binaural_decoder_generation() {
        let result = HoaConverter::generate_binaural_decoder(1, HrtfType::Diffuse);

        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.class_name, "FoaDecode");
        assert_eq!(obj.method, Some("ar".to_string()));
    }

    #[test]
    fn test_optimal_order_calculation() {
        let array = create_test_speaker_array();
        let optimal_order = HoaConverter::calculate_optimal_order(&array);

        assert!(optimal_order >= 1);
        assert!(optimal_order <= 7);
    }

    #[test]
    fn test_hoa_validation() {
        let validation = HoaConverter::validate_hoa_config(2, 16);

        assert!(validation.is_valid); // 16 speakers should be enough for order 2
        assert!(validation.recommended_order > 0);
    }

    #[test]
    fn test_hoa_validation_insufficient_speakers() {
        let validation = HoaConverter::validate_hoa_config(3, 8);

        assert!(!validation.is_valid); // 8 speakers not enough for order 3 (needs 16)
        assert!(!validation.errors.is_empty());
    }
}

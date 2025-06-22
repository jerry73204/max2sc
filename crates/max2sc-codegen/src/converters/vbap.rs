//! Vector Based Amplitude Panning (VBAP) converter

use max2sc_analyzer::spatial_analysis::{SpeakerArray, SpeakerArrayType};
use max2sc_core::{Result, SCObject, SCValue, SphericalCoord};

/// VBAP converter for SuperCollider
pub struct VbapConverter;

impl VbapConverter {
    /// Generate VBAP implementation for a speaker array
    pub fn generate_vbap_setup(array: &SpeakerArray) -> Result<SCObject> {
        match &array.array_type {
            SpeakerArrayType::Ring { radius } => Self::generate_ring_vbap_setup(array, *radius),
            SpeakerArrayType::Wfs { .. } => {
                // WFS arrays can also be used for VBAP
                Self::generate_linear_vbap_setup(array)
            }
            SpeakerArrayType::Irregular => Self::generate_irregular_vbap_setup(array),
        }
    }

    /// Generate VBAP setup for circular/ring speaker arrangement
    fn generate_ring_vbap_setup(array: &SpeakerArray, radius: f32) -> Result<SCObject> {
        let num_speakers = array.speakers.len() as i32;

        // Extract speaker angles for ring setup
        let mut speaker_angles: Vec<f32> =
            array.speakers.iter().map(|s| s.position.azimuth).collect();
        speaker_angles.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let angles: Vec<SCValue> = speaker_angles
            .into_iter()
            .map(|angle| SCValue::Float(angle))
            .collect();

        Ok(SCObject::new("VBAPSpeakerSetup")
            .with_method("new")
            .arg(num_speakers)
            .arg("ring")
            .arg(radius)
            .arg(SCValue::Array(angles))
            .prop("dimension", "2D")
            .prop("buffer_size", 512)
            .prop(
                "comment",
                format!(
                    "VBAP ring setup: {} speakers, {:.2}m radius",
                    num_speakers, radius
                ),
            ))
    }

    /// Generate VBAP setup for linear speaker arrangement
    fn generate_linear_vbap_setup(array: &SpeakerArray) -> Result<SCObject> {
        let num_speakers = array.speakers.len() as i32;

        let speaker_positions: Vec<SCValue> = array
            .speakers
            .iter()
            .map(|s| {
                SCValue::Array(vec![
                    SCValue::Float(s.position.azimuth),
                    SCValue::Float(s.position.elevation),
                    SCValue::Float(s.position.distance),
                ])
            })
            .collect();

        Ok(SCObject::new("VBAPSpeakerSetup")
            .with_method("new")
            .arg(num_speakers)
            .arg("linear")
            .arg(SCValue::Array(speaker_positions))
            .prop("dimension", "3D")
            .prop(
                "comment",
                format!("VBAP linear setup: {} speakers", num_speakers),
            ))
    }

    /// Generate VBAP setup for irregular speaker arrangement
    fn generate_irregular_vbap_setup(array: &SpeakerArray) -> Result<SCObject> {
        let num_speakers = array.speakers.len() as i32;

        // Create position matrix for irregular setup
        let speaker_positions: Vec<SCValue> = array
            .speakers
            .iter()
            .map(|s| {
                // Convert spherical to cartesian coordinates for VBAP
                let (x, y, z) = spherical_to_cartesian(&s.position);
                SCValue::Array(vec![
                    SCValue::Float(x),
                    SCValue::Float(y),
                    SCValue::Float(z),
                ])
            })
            .collect();

        Ok(SCObject::new("VBAPSpeakerSetup")
            .with_method("new")
            .arg(num_speakers)
            .arg("irregular")
            .arg(SCValue::Array(speaker_positions))
            .prop("dimension", "3D")
            .prop("triangulation", "auto")
            .prop(
                "comment",
                format!("VBAP irregular setup: {} speakers", num_speakers),
            ))
    }

    /// Generate VBAP panning UGen
    pub fn generate_vbap_panner(
        num_channels: i32,
        speaker_setup: &str,
        use_3d: bool,
    ) -> Result<SCObject> {
        let ugen_name = if use_3d { "VBAP3D" } else { "VBAP" };

        Ok(SCObject::new(ugen_name)
            .with_method("ar")
            .arg(num_channels)
            .arg(SCValue::Symbol("input".to_string()))
            .arg(SCValue::Symbol("azimuth".to_string()))
            .arg(SCValue::Symbol("elevation".to_string()))
            .arg(SCValue::Symbol("spread".to_string()))
            .arg(SCValue::Symbol("gain".to_string()))
            .prop("speaker_setup", speaker_setup)
            .prop(
                "comment",
                format!(
                    "VBAP panner: {} channels, {}",
                    num_channels,
                    if use_3d { "3D" } else { "2D" }
                ),
            ))
    }

    /// Generate distance-based VBAP with compensation
    pub fn generate_distance_vbap(
        num_channels: i32,
        compensation_type: DistanceCompensation,
    ) -> Result<SCObject> {
        let compensation_factor = match compensation_type {
            DistanceCompensation::None => 0.0,
            DistanceCompensation::Linear => 1.0,
            DistanceCompensation::InverseSquare => 2.0,
        };

        Ok(SCObject::new("VBAPDistance")
            .with_method("ar")
            .arg(num_channels)
            .arg(SCValue::Symbol("input".to_string()))
            .arg(SCValue::Symbol("azimuth".to_string()))
            .arg(SCValue::Symbol("elevation".to_string()))
            .arg(SCValue::Symbol("distance".to_string()))
            .arg(SCValue::Symbol("spread".to_string()))
            .arg(compensation_factor)
            .prop("reference_distance", 2.0)
            .prop(
                "comment",
                format!(
                    "Distance VBAP: {} channels, {:?} compensation",
                    num_channels, compensation_type
                ),
            ))
    }

    /// Generate VBAP with spread control
    pub fn generate_spread_vbap(num_channels: i32, spread_type: SpreadType) -> Result<SCObject> {
        let spread_method = match spread_type {
            SpreadType::Linear => "linear",
            SpreadType::Gaussian => "gaussian",
            SpreadType::Uniform => "uniform",
        };

        Ok(SCObject::new("VBAPSpread")
            .with_method("ar")
            .arg(num_channels)
            .arg(SCValue::Symbol("input".to_string()))
            .arg(SCValue::Symbol("azimuth".to_string()))
            .arg(SCValue::Symbol("elevation".to_string()))
            .arg(SCValue::Symbol("spread_amount".to_string()))
            .arg(spread_method)
            .prop(
                "comment",
                format!(
                    "VBAP with {} spread: {} channels",
                    spread_method, num_channels
                ),
            ))
    }

    /// Generate VBAP speaker configuration validation
    pub fn generate_speaker_validation(array: &SpeakerArray) -> Result<SCObject> {
        let num_speakers = array.speakers.len() as i32;
        let is_3d = array
            .speakers
            .iter()
            .any(|s| s.position.elevation.abs() > 1.0);

        Ok(SCObject::new("VBAPValidation")
            .with_method("validate")
            .arg(num_speakers)
            .arg(is_3d)
            .arg(SCValue::Symbol("speaker_positions".to_string()))
            .prop("min_angle_threshold", 10.0) // Minimum angle between speakers
            .prop("max_distance_ratio", 3.0) // Maximum distance variation
            .prop("comment", "VBAP speaker setup validation"))
    }

    /// Generate optimal triplet calculation for 3D VBAP
    pub fn generate_triplet_calculation(array: &SpeakerArray) -> Result<SCObject> {
        let num_speakers = array.speakers.len() as i32;

        Ok(SCObject::new("VBAPTriplets")
            .with_method("calculate")
            .arg(num_speakers)
            .arg(SCValue::Symbol("speaker_positions".to_string()))
            .prop("hull_method", "convex")
            .prop("angle_threshold", 10.0)
            .prop("comment", "VBAP triplet calculation for 3D panning"))
    }
}

/// Distance compensation types for VBAP
#[derive(Debug, Clone, Copy)]
pub enum DistanceCompensation {
    None,
    Linear,
    InverseSquare,
}

/// Spread types for VBAP
#[derive(Debug, Clone, Copy)]
pub enum SpreadType {
    Linear,
    Gaussian,
    Uniform,
}

/// Helper functions for VBAP calculations
impl VbapConverter {
    /// Convert spherical coordinates to cartesian for VBAP calculations
    pub fn spherical_to_cartesian_vbap(pos: &SphericalCoord) -> (f32, f32, f32) {
        spherical_to_cartesian(pos)
    }

    /// Calculate optimal speaker triangle for a given direction
    pub fn find_optimal_triangle(
        azimuth: f32,
        elevation: f32,
        speakers: &[max2sc_analyzer::spatial_analysis::Speaker],
    ) -> Option<(usize, usize, usize)> {
        // Simplified triangle finding - in practice this would be more complex
        if speakers.len() < 3 {
            return None;
        }

        // For now, return first three speakers as placeholder
        Some((0, 1, 2))
    }

    /// Calculate VBAP gain factors for a speaker triangle
    pub fn calculate_vbap_gains(
        azimuth: f32,
        elevation: f32,
        speaker_triangle: (usize, usize, usize),
        speakers: &[max2sc_analyzer::spatial_analysis::Speaker],
    ) -> (f32, f32, f32) {
        // Simplified VBAP gain calculation
        // In practice, this would solve the VBAP matrix equation
        let base_gain = 1.0 / 3.0;
        (base_gain, base_gain, base_gain)
    }

    /// Validate speaker setup for VBAP compatibility
    pub fn validate_speaker_setup(array: &SpeakerArray) -> VbapValidationResult {
        let mut result = VbapValidationResult {
            is_valid: true,
            warnings: Vec::new(),
            errors: Vec::new(),
            optimal_spread: 0.0,
        };

        // Check minimum number of speakers
        if array.speakers.len() < 3 {
            result.is_valid = false;
            result
                .errors
                .push("VBAP requires at least 3 speakers".to_string());
            return result;
        }

        // Check for speakers too close together
        for (i, speaker1) in array.speakers.iter().enumerate() {
            for (j, speaker2) in array.speakers.iter().enumerate() {
                if i >= j {
                    continue;
                }

                let angle_diff = (speaker1.position.azimuth - speaker2.position.azimuth).abs();
                if angle_diff < 10.0 {
                    result.warnings.push(format!(
                        "Speakers {} and {} are very close ({:.1}°)",
                        i, j, angle_diff
                    ));
                }
            }
        }

        // Calculate optimal spread
        result.optimal_spread = calculate_optimal_spread(array);

        result
    }
}

/// Helper function for coordinate conversion
fn spherical_to_cartesian(pos: &SphericalCoord) -> (f32, f32, f32) {
    let azimuth_rad = pos.azimuth.to_radians();
    let elevation_rad = pos.elevation.to_radians();

    let x = pos.distance * elevation_rad.cos() * azimuth_rad.cos();
    let y = pos.distance * elevation_rad.cos() * azimuth_rad.sin();
    let z = pos.distance * elevation_rad.sin();

    (x, y, z)
}

/// Calculate optimal spread for speaker array
fn calculate_optimal_spread(array: &SpeakerArray) -> f32 {
    if array.speakers.len() < 2 {
        return 0.0;
    }

    // Calculate average angular spacing
    let mut angles: Vec<f32> = array.speakers.iter().map(|s| s.position.azimuth).collect();
    angles.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let mut total_spacing = 0.0;
    for i in 1..angles.len() {
        total_spacing += angles[i] - angles[i - 1];
    }

    // Add wrap-around spacing
    total_spacing += (360.0 + angles[0] - angles[angles.len() - 1]) % 360.0;

    total_spacing / array.speakers.len() as f32
}

/// VBAP validation result
#[derive(Debug)]
pub struct VbapValidationResult {
    pub is_valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub optimal_spread: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use max2sc_analyzer::spatial_analysis::{Speaker, SpeakerArrayType};
    use max2sc_core::SphericalCoord;

    fn create_test_ring_array() -> SpeakerArray {
        let speakers = (0..8)
            .map(|i| {
                Speaker {
                    id: i,
                    position: SphericalCoord {
                        azimuth: i as f32 * 45.0, // 0°, 45°, 90°, etc.
                        elevation: 0.0,
                        distance: 2.5,
                    },
                    delay: 0.0,
                    gain: 1.0,
                }
            })
            .collect();

        SpeakerArray {
            id: "ring_vbap".to_string(),
            array_type: SpeakerArrayType::Ring { radius: 2.5 },
            speakers,
            wfs_config: None,
        }
    }

    fn create_test_3d_array() -> SpeakerArray {
        let mut speakers = Vec::new();

        // Create cube-like arrangement
        for i in 0..8 {
            let azimuth = (i % 4) as f32 * 90.0;
            let elevation = if i < 4 { -30.0 } else { 30.0 };

            speakers.push(Speaker {
                id: i,
                position: SphericalCoord {
                    azimuth,
                    elevation,
                    distance: 3.0,
                },
                delay: 0.0,
                gain: 1.0,
            });
        }

        SpeakerArray {
            id: "cube_vbap".to_string(),
            array_type: SpeakerArrayType::Irregular,
            speakers,
            wfs_config: None,
        }
    }

    #[test]
    fn test_ring_vbap_setup() {
        let array = create_test_ring_array();
        let result = VbapConverter::generate_vbap_setup(&array);

        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.class_name, "VBAPSpeakerSetup");
        assert_eq!(obj.method, Some("new".to_string()));
    }

    #[test]
    fn test_3d_vbap_setup() {
        let array = create_test_3d_array();
        let result = VbapConverter::generate_vbap_setup(&array);

        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.class_name, "VBAPSpeakerSetup");
        assert_eq!(obj.method, Some("new".to_string()));
    }

    #[test]
    fn test_vbap_panner_generation() {
        let result = VbapConverter::generate_vbap_panner(8, "ring", false);

        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.class_name, "VBAP");
        assert_eq!(obj.method, Some("ar".to_string()));
    }

    #[test]
    fn test_3d_vbap_panner_generation() {
        let result = VbapConverter::generate_vbap_panner(8, "cube", true);

        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.class_name, "VBAP3D");
        assert_eq!(obj.method, Some("ar".to_string()));
    }

    #[test]
    fn test_distance_vbap_generation() {
        let result = VbapConverter::generate_distance_vbap(8, DistanceCompensation::InverseSquare);

        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.class_name, "VBAPDistance");
        assert_eq!(obj.method, Some("ar".to_string()));
    }

    #[test]
    fn test_speaker_validation() {
        let array = create_test_ring_array();
        let validation = VbapConverter::validate_speaker_setup(&array);

        assert!(validation.is_valid);
        assert!(validation.optimal_spread > 0.0);
    }

    #[test]
    fn test_spherical_to_cartesian() {
        let pos = SphericalCoord {
            azimuth: 0.0,
            elevation: 0.0,
            distance: 1.0,
        };
        let (x, y, z) = spherical_to_cartesian(&pos);

        assert!((x - 1.0).abs() < 0.001);
        assert!(y.abs() < 0.001);
        assert!(z.abs() < 0.001);
    }

    #[test]
    fn test_optimal_spread_calculation() {
        let array = create_test_ring_array();
        let spread = calculate_optimal_spread(&array);

        assert!((spread - 45.0).abs() < 1.0); // Should be close to 45° for 8 speakers
    }
}

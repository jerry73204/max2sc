//! Wave Field Synthesis (WFS) converter

use max2sc_analyzer::spatial_analysis::{SpeakerArray, SpeakerArrayType, WfsConfig};
use max2sc_core::{Result, SCObject, SCValue, SphericalCoord};

/// WFS array converter for SuperCollider
pub struct WfsConverter;

impl WfsConverter {
    /// Generate WFS implementation for a speaker array
    pub fn generate_wfs_array(array: &SpeakerArray) -> Result<SCObject> {
        match &array.array_type {
            SpeakerArrayType::Wfs { length, spacing } => {
                Self::generate_linear_wfs_array(array, *length, *spacing)
            }
            SpeakerArrayType::Ring { radius } => Self::generate_circular_wfs_array(array, *radius),
            SpeakerArrayType::Irregular => Self::generate_irregular_wfs_array(array),
        }
    }

    /// Generate linear WFS array implementation
    fn generate_linear_wfs_array(
        array: &SpeakerArray,
        length: f32,
        spacing: f32,
    ) -> Result<SCObject> {
        let num_speakers = array.speakers.len() as i32;
        let default_config = WfsConfig::default();
        let wfs_config = array.wfs_config.as_ref().unwrap_or(&default_config);

        // Generate speaker delays for WFS
        let delays = Self::calculate_wfs_delays(&array.speakers);
        let gains = Self::calculate_wfs_gains(&array.speakers);

        Ok(SCObject::new("WFSArrayLinear")
            .with_method("ar")
            .arg(SCValue::Symbol("input".to_string()))
            .arg(SCValue::Symbol("source_azimuth".to_string()))
            .arg(SCValue::Symbol("source_distance".to_string()))
            .arg(num_speakers)
            .arg(length)
            .arg(spacing)
            .arg(SCValue::Array(delays))
            .arg(SCValue::Array(gains))
            .prop("prefilter_cutoff", wfs_config.prefilter_cutoff)
            .prop("distance_compensation", wfs_config.distance_compensation)
            .prop("amplitude_correction", wfs_config.amplitude_correction)
            .prop("aliasing_frequency", wfs_config.aliasing_frequency)
            .prop(
                "comment",
                format!("WFS linear array: {num_speakers} speakers, {length:.2}m length"),
            ))
    }

    /// Generate circular WFS array implementation
    fn generate_circular_wfs_array(array: &SpeakerArray, radius: f32) -> Result<SCObject> {
        let num_speakers = array.speakers.len() as i32;
        let default_config = WfsConfig::default();
        let wfs_config = array.wfs_config.as_ref().unwrap_or(&default_config);

        let delays = Self::calculate_wfs_delays(&array.speakers);
        let gains = Self::calculate_wfs_gains(&array.speakers);

        Ok(SCObject::new("WFSArrayCircular")
            .with_method("ar")
            .arg(SCValue::Symbol("input".to_string()))
            .arg(SCValue::Symbol("source_azimuth".to_string()))
            .arg(SCValue::Symbol("source_distance".to_string()))
            .arg(num_speakers)
            .arg(radius)
            .arg(SCValue::Array(delays))
            .arg(SCValue::Array(gains))
            .prop("prefilter_cutoff", wfs_config.prefilter_cutoff)
            .prop("distance_compensation", wfs_config.distance_compensation)
            .prop("amplitude_correction", wfs_config.amplitude_correction)
            .prop(
                "comment",
                format!("WFS circular array: {num_speakers} speakers, {radius:.2}m radius"),
            ))
    }

    /// Generate irregular WFS array implementation
    fn generate_irregular_wfs_array(array: &SpeakerArray) -> Result<SCObject> {
        let num_speakers = array.speakers.len() as i32;
        let default_config = WfsConfig::default();
        let wfs_config = array.wfs_config.as_ref().unwrap_or(&default_config);

        // Extract speaker positions as arrays
        let azimuths: Vec<SCValue> = array
            .speakers
            .iter()
            .map(|s| SCValue::Float(s.position.azimuth))
            .collect();
        let distances: Vec<SCValue> = array
            .speakers
            .iter()
            .map(|s| SCValue::Float(s.position.distance))
            .collect();
        let elevations: Vec<SCValue> = array
            .speakers
            .iter()
            .map(|s| SCValue::Float(s.position.elevation))
            .collect();

        let delays = Self::calculate_wfs_delays(&array.speakers);
        let gains = Self::calculate_wfs_gains(&array.speakers);

        Ok(SCObject::new("WFSArrayIrregular")
            .with_method("ar")
            .arg(SCValue::Symbol("input".to_string()))
            .arg(SCValue::Symbol("source_azimuth".to_string()))
            .arg(SCValue::Symbol("source_distance".to_string()))
            .arg(num_speakers)
            .arg(SCValue::Array(azimuths))
            .arg(SCValue::Array(elevations))
            .arg(SCValue::Array(distances))
            .arg(SCValue::Array(delays))
            .arg(SCValue::Array(gains))
            .prop("prefilter_cutoff", wfs_config.prefilter_cutoff)
            .prop("distance_compensation", wfs_config.distance_compensation)
            .prop(
                "comment",
                format!("WFS irregular array: {num_speakers} speakers"),
            ))
    }

    /// Calculate WFS delays for each speaker
    fn calculate_wfs_delays(
        speakers: &[max2sc_analyzer::spatial_analysis::Speaker],
    ) -> Vec<SCValue> {
        speakers
            .iter()
            .map(|speaker| SCValue::Float(speaker.delay))
            .collect()
    }

    /// Calculate WFS gains for each speaker
    fn calculate_wfs_gains(
        speakers: &[max2sc_analyzer::spatial_analysis::Speaker],
    ) -> Vec<SCValue> {
        speakers
            .iter()
            .map(|speaker| SCValue::Float(speaker.gain))
            .collect()
    }

    /// Generate WFS prefilter implementation
    pub fn generate_wfs_prefilter(cutoff: f32) -> Result<SCObject> {
        Ok(SCObject::new("WFSPrefilter")
            .with_method("ar")
            .arg(SCValue::Symbol("input".to_string()))
            .arg(cutoff)
            .prop("comment", "WFS prefilter for spatial aliasing reduction"))
    }

    /// Generate focused source implementation for WFS
    pub fn generate_wfs_focused_source(
        source_azimuth: f32,
        source_distance: f32,
        focus_distance: f32,
    ) -> Result<SCObject> {
        Ok(SCObject::new("WFSFocusedSource")
            .with_method("ar")
            .arg(SCValue::Symbol("input".to_string()))
            .arg(source_azimuth)
            .arg(source_distance)
            .arg(focus_distance)
            .prop("comment", "WFS focused source with virtual distance"))
    }

    /// Generate WFS plane wave implementation
    pub fn generate_wfs_plane_wave(azimuth: f32) -> Result<SCObject> {
        Ok(SCObject::new("WFSPlaneWave")
            .with_method("ar")
            .arg(SCValue::Symbol("input".to_string()))
            .arg(azimuth)
            .prop("comment", "WFS plane wave synthesis"))
    }

    /// Generate distance-based amplitude compensation
    pub fn generate_distance_compensation(reference_distance: f32) -> Result<SCObject> {
        Ok(SCObject::new("WFSDistanceCompensation")
            .with_method("ar")
            .arg(SCValue::Symbol("input".to_string()))
            .arg(SCValue::Symbol("source_distance".to_string()))
            .arg(reference_distance)
            .prop("comment", "WFS distance-based amplitude compensation"))
    }
}

/// Helper functions for WFS calculations
impl WfsConverter {
    /// Calculate the speed of sound (m/s) at given temperature
    pub fn speed_of_sound(temperature_celsius: f32) -> f32 {
        343.0 + (0.6 * temperature_celsius)
    }

    /// Calculate delay time for a speaker relative to a virtual source
    pub fn calculate_speaker_delay(
        speaker_pos: &SphericalCoord,
        source_azimuth: f32,
        source_distance: f32,
        sound_speed: f32,
    ) -> f32 {
        // Convert spherical to cartesian for easier calculation
        let speaker_x = speaker_pos.distance * speaker_pos.azimuth.to_radians().cos();
        let speaker_y = speaker_pos.distance * speaker_pos.azimuth.to_radians().sin();

        let source_x = source_distance * source_azimuth.to_radians().cos();
        let source_y = source_distance * source_azimuth.to_radians().sin();

        // Calculate distance from virtual source to speaker
        let distance = ((speaker_x - source_x).powi(2) + (speaker_y - source_y).powi(2)).sqrt();

        // Convert to delay time
        distance / sound_speed
    }

    /// Calculate amplitude factor for WFS based on distance
    pub fn calculate_wfs_amplitude(
        speaker_pos: &SphericalCoord,
        source_distance: f32,
        reference_distance: f32,
    ) -> f32 {
        if source_distance <= 0.0 {
            return 1.0; // Plane wave case
        }

        // Distance-based amplitude scaling
        let distance_factor = (reference_distance / source_distance).sqrt();

        // Additional scaling based on speaker distance
        let speaker_factor = (reference_distance / speaker_pos.distance).sqrt();

        distance_factor * speaker_factor
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use max2sc_analyzer::spatial_analysis::{Speaker, SpeakerArrayType};
    use max2sc_core::SphericalCoord;

    fn create_test_linear_array() -> SpeakerArray {
        let speakers = (0..16)
            .map(|i| {
                Speaker {
                    id: i,
                    position: SphericalCoord {
                        azimuth: -45.0 + (i as f32 * 90.0 / 15.0), // -45° to +45°
                        elevation: 0.0,
                        distance: 3.0,
                    },
                    delay: 0.0,
                    gain: 1.0,
                }
            })
            .collect();

        SpeakerArray {
            id: "linear_wfs".to_string(),
            array_type: SpeakerArrayType::Wfs {
                length: 8.0,
                spacing: 0.5,
            },
            speakers,
            wfs_config: Some(WfsConfig::default()),
        }
    }

    fn create_test_circular_array() -> SpeakerArray {
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
            id: "circular_wfs".to_string(),
            array_type: SpeakerArrayType::Ring { radius: 2.5 },
            speakers,
            wfs_config: Some(WfsConfig::default()),
        }
    }

    #[test]
    fn test_linear_wfs_array_generation() {
        let array = create_test_linear_array();
        let result = WfsConverter::generate_wfs_array(&array);

        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.class_name, "WFSArrayLinear");
        assert_eq!(obj.method, Some("ar".to_string()));
        assert!(obj.args.len() >= 8); // input, azimuth, distance, num_speakers, length, spacing, delays, gains
    }

    #[test]
    fn test_circular_wfs_array_generation() {
        let array = create_test_circular_array();
        let result = WfsConverter::generate_wfs_array(&array);

        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.class_name, "WFSArrayCircular");
        assert_eq!(obj.method, Some("ar".to_string()));
    }

    #[test]
    fn test_wfs_prefilter_generation() {
        let result = WfsConverter::generate_wfs_prefilter(1000.0);

        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.class_name, "WFSPrefilter");
        assert_eq!(obj.method, Some("ar".to_string()));
    }

    #[test]
    fn test_wfs_delay_calculation() {
        let speaker_pos = SphericalCoord {
            azimuth: 0.0,
            elevation: 0.0,
            distance: 3.0,
        };
        let delay = WfsConverter::calculate_speaker_delay(&speaker_pos, 30.0, 2.0, 343.0);
        assert!(delay >= 0.0);
        assert!(delay < 0.1); // Should be reasonable delay time
    }

    #[test]
    fn test_wfs_amplitude_calculation() {
        let speaker_pos = SphericalCoord {
            azimuth: 0.0,
            elevation: 0.0,
            distance: 3.0,
        };
        let amplitude = WfsConverter::calculate_wfs_amplitude(&speaker_pos, 2.0, 3.0);
        assert!(amplitude > 0.0);
        assert!(amplitude <= 2.0); // Should be reasonable amplitude factor
    }
}

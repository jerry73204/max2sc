//! Spatial configuration analysis

use crate::AnalysisError;
use max2sc_core::{AudioFormat, SphericalCoord};
use max2sc_max_types::{MaxPatch, OSCConfig};

/// Analyze spatial configuration from Max patch and OSC setup
pub fn analyze_spatial_config(
    patch: &MaxPatch,
    osc_config: Option<&OSCConfig>,
) -> Result<SpatialConfig, AnalysisError> {
    // Analyze Max patch for spatial objects
    let spatial_objects = analyze_spatial_objects(patch)?;

    // Analyze speaker configuration from OSC config
    let speaker_arrays = if let Some(osc) = osc_config {
        analyze_speaker_arrays(osc)?
    } else {
        Vec::new()
    };

    let mut config = SpatialConfig {
        spatial_objects,
        speaker_arrays,
        ..Default::default()
    };

    // Determine optimal spatial processing method
    config.processing_method = determine_processing_method(&config);

    Ok(config)
}

/// Analyze spatial objects in the patch
fn analyze_spatial_objects(patch: &MaxPatch) -> Result<Vec<SpatialObject>, AnalysisError> {
    let mut objects = Vec::new();

    for obj in &patch.patcher.boxes {
        if let Some(text) = &obj.content.text {
            let obj_name = text.split_whitespace().next().unwrap_or("");

            match obj_name {
                "spat5.panoramix~" => {
                    objects.push(SpatialObject {
                        id: obj.content.id.clone(),
                        object_type: SpatialObjectType::Panoramix,
                        inputs: obj.content.numinlets,
                        outputs: obj.content.numoutlets,
                        format: AudioFormat::Multichannel(obj.content.numoutlets),
                        parameters: parse_panoramix_params(text),
                    });
                }
                "spat5.hoa.encoder~" => {
                    let order = extract_hoa_order(text).unwrap_or(1);
                    objects.push(SpatialObject {
                        id: obj.content.id.clone(),
                        object_type: SpatialObjectType::HoaEncoder { order },
                        inputs: obj.content.numinlets,
                        outputs: obj.content.numoutlets,
                        format: AudioFormat::Ambisonic {
                            order,
                            dimension: 3,
                        },
                        parameters: Vec::new(),
                    });
                }
                "spat5.hoa.decoder~" => {
                    let order = extract_hoa_order(text).unwrap_or(1);
                    objects.push(SpatialObject {
                        id: obj.content.id.clone(),
                        object_type: SpatialObjectType::HoaDecoder { order },
                        inputs: obj.content.numinlets,
                        outputs: obj.content.numoutlets,
                        format: AudioFormat::Multichannel(obj.content.numoutlets),
                        parameters: Vec::new(),
                    });
                }
                "spat5.vbap~" => {
                    let num_speakers = extract_speaker_count(text).unwrap_or(8);
                    objects.push(SpatialObject {
                        id: obj.content.id.clone(),
                        object_type: SpatialObjectType::Vbap { num_speakers },
                        inputs: obj.content.numinlets,
                        outputs: obj.content.numoutlets,
                        format: AudioFormat::Multichannel(num_speakers),
                        parameters: Vec::new(),
                    });
                }
                _ if obj_name.starts_with("spat5") => {
                    objects.push(SpatialObject {
                        id: obj.content.id.clone(),
                        object_type: SpatialObjectType::Generic(obj_name.to_string()),
                        inputs: obj.content.numinlets,
                        outputs: obj.content.numoutlets,
                        format: AudioFormat::Multichannel(obj.content.numoutlets),
                        parameters: Vec::new(),
                    });
                }
                _ => {}
            }
        }
    }

    Ok(objects)
}

/// Analyze speaker arrays from OSC configuration
fn analyze_speaker_arrays(osc_config: &OSCConfig) -> Result<Vec<SpeakerArray>, AnalysisError> {
    let mut arrays = Vec::new();

    for array in &osc_config.speaker_arrays {
        let mut speakers = Vec::new();

        // Convert OSC speakers to our speaker format
        for osc_speaker in &array.speakers {
            let coords = SphericalCoord {
                azimuth: osc_speaker.position.azimuth,
                elevation: osc_speaker.position.elevation,
                distance: osc_speaker.position.distance,
            };

            speakers.push(Speaker {
                id: osc_speaker.id,
                position: coords,
                delay: osc_speaker.delay,
                gain: osc_speaker.gain,
            });
        }

        // Determine array type based on speaker positions
        let array_type = determine_array_type(&speakers);

        arrays.push(SpeakerArray {
            id: array.name.clone(),
            array_type: array_type.clone(),
            speakers,
            wfs_config: if matches!(array_type, SpeakerArrayType::Wfs { .. }) {
                Some(WfsConfig::default())
            } else {
                None
            },
        });
    }

    Ok(arrays)
}

/// Determine the optimal spatial processing method
fn determine_processing_method(config: &SpatialConfig) -> SpatialProcessingMethod {
    // Check if we have WFS-suitable arrays
    let has_wfs_array = config
        .speaker_arrays
        .iter()
        .any(|array| matches!(array.array_type, SpeakerArrayType::Wfs { .. }));

    // Check if we have HOA objects
    let has_hoa = config.spatial_objects.iter().any(|obj| {
        matches!(
            obj.object_type,
            SpatialObjectType::HoaEncoder { .. } | SpatialObjectType::HoaDecoder { .. }
        )
    });

    // Check speaker count for VBAP
    let max_speakers = config
        .speaker_arrays
        .iter()
        .map(|array| array.speakers.len())
        .max()
        .unwrap_or(0);

    match (has_wfs_array, has_hoa, max_speakers) {
        (true, _, _) => SpatialProcessingMethod::Wfs,
        (false, true, _) => SpatialProcessingMethod::Hoa,
        (false, false, n) if n >= 4 => SpatialProcessingMethod::Vbap,
        _ => SpatialProcessingMethod::Stereo,
    }
}

// Helper functions

fn parse_panoramix_params(_text: &str) -> Vec<SpatialParameter> {
    // Parse spat5.panoramix~ parameters
    Vec::new() // TODO: Implement parameter parsing
}

fn extract_hoa_order(text: &str) -> Option<u32> {
    text.split_whitespace().nth(1).and_then(|s| s.parse().ok())
}

fn extract_speaker_count(text: &str) -> Option<u32> {
    text.split_whitespace().nth(1).and_then(|s| s.parse().ok())
}

fn determine_array_type(speakers: &[Speaker]) -> SpeakerArrayType {
    if speakers.len() >= 16 && is_linear_array(speakers) {
        SpeakerArrayType::Wfs {
            length: calculate_array_length(speakers),
            spacing: calculate_speaker_spacing(speakers),
        }
    } else if speakers.len() >= 4 && is_circular_array(speakers) {
        SpeakerArrayType::Ring {
            radius: calculate_average_distance(speakers),
        }
    } else {
        SpeakerArrayType::Irregular
    }
}

fn is_linear_array(speakers: &[Speaker]) -> bool {
    // Check if speakers form a linear arrangement
    if speakers.len() < 3 {
        return false;
    }

    // Simple heuristic: check if elevation is similar and azimuth changes linearly
    let avg_elevation =
        speakers.iter().map(|s| s.position.elevation).sum::<f32>() / speakers.len() as f32;

    speakers.iter().all(|s| {
        (s.position.elevation - avg_elevation).abs() < 10.0 // 10 degree tolerance
    })
}

fn is_circular_array(speakers: &[Speaker]) -> bool {
    // Check if speakers form a circular arrangement
    let avg_distance = calculate_average_distance(speakers);
    speakers.iter().all(|s| {
        (s.position.distance - avg_distance).abs() < 0.5 // 0.5m tolerance
    })
}

fn calculate_array_length(speakers: &[Speaker]) -> f32 {
    // Calculate the physical length of a linear array
    let angles: Vec<f32> = speakers.iter().map(|s| s.position.azimuth).collect();
    let min_angle = angles.iter().fold(f32::INFINITY, |a, &b| a.min(b));
    let max_angle = angles.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
    let avg_distance = calculate_average_distance(speakers);

    // Convert angular span to linear distance
    avg_distance * (max_angle - min_angle).to_radians()
}

fn calculate_speaker_spacing(speakers: &[Speaker]) -> f32 {
    if speakers.len() < 2 {
        return 0.0;
    }

    let mut angles: Vec<f32> = speakers.iter().map(|s| s.position.azimuth).collect();
    angles.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let total_span = angles.last().unwrap() - angles.first().unwrap();
    let avg_distance = calculate_average_distance(speakers);

    // Average angular spacing converted to linear distance
    avg_distance * (total_span / (speakers.len() - 1) as f32).to_radians()
}

fn calculate_average_distance(speakers: &[Speaker]) -> f32 {
    speakers.iter().map(|s| s.position.distance).sum::<f32>() / speakers.len() as f32
}

#[derive(Debug, Default)]
pub struct SpatialConfig {
    pub spatial_objects: Vec<SpatialObject>,
    pub speaker_arrays: Vec<SpeakerArray>,
    pub processing_method: SpatialProcessingMethod,
}

#[derive(Debug)]
pub struct SpatialObject {
    pub id: String,
    pub object_type: SpatialObjectType,
    pub inputs: u32,
    pub outputs: u32,
    pub format: AudioFormat,
    pub parameters: Vec<SpatialParameter>,
}

#[derive(Debug)]
pub enum SpatialObjectType {
    Panoramix,
    HoaEncoder { order: u32 },
    HoaDecoder { order: u32 },
    Vbap { num_speakers: u32 },
    Generic(String),
}

#[derive(Debug)]
pub struct SpatialParameter {
    pub name: String,
    pub value: f32,
    pub range: (f32, f32),
}

#[derive(Debug)]
pub struct SpeakerArray {
    pub id: String,
    pub array_type: SpeakerArrayType,
    pub speakers: Vec<Speaker>,
    pub wfs_config: Option<WfsConfig>,
}

#[derive(Debug, Clone)]
pub enum SpeakerArrayType {
    Ring { radius: f32 },
    Wfs { length: f32, spacing: f32 },
    Irregular,
}

#[derive(Debug)]
pub struct Speaker {
    pub id: u32,
    pub position: SphericalCoord,
    pub delay: f32,
    pub gain: f32,
}

#[derive(Debug, Default)]
pub struct WfsConfig {
    pub prefilter_cutoff: f32,
    pub distance_compensation: bool,
    pub amplitude_correction: bool,
    pub aliasing_frequency: f32,
}

#[derive(Debug, Default)]
pub enum SpatialProcessingMethod {
    #[default]
    Stereo,
    Vbap,
    Hoa,
    Wfs,
}

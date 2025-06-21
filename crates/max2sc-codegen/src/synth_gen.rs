//! SynthDef generation

use crate::CodegenError;
use max2sc_max_types::MaxPatch;
use max2sc_sc_types::{Parameter, Rate, SynthDef, UGen, UGenInput, UGenOutput};

/// Generate SynthDefs from a Max patch
pub fn generate_synth_defs(patch: &MaxPatch) -> Result<Vec<SynthDef>, CodegenError> {
    let mut synth_defs = Vec::new();

    // Generate a basic synth def based on patch contents
    let basic_synth = generate_basic_synthdef(patch)?;
    synth_defs.push(basic_synth);

    Ok(synth_defs)
}

/// Generate a basic SynthDef from patch objects
fn generate_basic_synthdef(patch: &MaxPatch) -> Result<SynthDef, CodegenError> {
    let mut ugens = Vec::new();
    let mut params = Vec::new();

    // Add default parameters
    params.push(Parameter {
        name: "freq".to_string(),
        default_value: 440.0,
        rate: Rate::Control,
    });

    params.push(Parameter {
        name: "amp".to_string(),
        default_value: 0.5,
        rate: Rate::Control,
    });

    params.push(Parameter {
        name: "pan".to_string(),
        default_value: 0.0,
        rate: Rate::Control,
    });

    // Check what types of objects are in the patch
    let mut has_audio = false;
    for box_container in &patch.patcher.boxes {
        let text = box_container.content.text.as_deref().unwrap_or("");
        if text.contains('~') || text.starts_with("spat5") {
            has_audio = true;
            break;
        }
    }

    // If audio objects found, create appropriate synth
    if has_audio {
        // Simple sine oscillator
        ugens.push(UGen {
            name: "SinOsc".to_string(),
            rate: Rate::Audio,
            inputs: vec![
                UGenInput::Parameter("freq".to_string()),
                UGenInput::Constant(0.0), // phase
            ],
            outputs: vec![UGenOutput { rate: Rate::Audio }],
            special_index: None,
        });

        // Pan2 for stereo output
        ugens.push(UGen {
            name: "Pan2".to_string(),
            rate: Rate::Audio,
            inputs: vec![
                UGenInput::UGen {
                    ugen_index: 0,
                    output_index: 0,
                }, // input from SinOsc
                UGenInput::Parameter("pan".to_string()),
            ],
            outputs: vec![
                UGenOutput { rate: Rate::Audio },
                UGenOutput { rate: Rate::Audio },
            ],
            special_index: None,
        });

        // Multiply by amplitude
        ugens.push(UGen {
            name: "*".to_string(),
            rate: Rate::Audio,
            inputs: vec![
                UGenInput::UGen {
                    ugen_index: 1,
                    output_index: 0,
                }, // left channel
                UGenInput::Parameter("amp".to_string()),
            ],
            outputs: vec![UGenOutput { rate: Rate::Audio }],
            special_index: None,
        });

        ugens.push(UGen {
            name: "*".to_string(),
            rate: Rate::Audio,
            inputs: vec![
                UGenInput::UGen {
                    ugen_index: 1,
                    output_index: 1,
                }, // right channel
                UGenInput::Parameter("amp".to_string()),
            ],
            outputs: vec![UGenOutput { rate: Rate::Audio }],
            special_index: None,
        });

        // Output
        ugens.push(UGen {
            name: "Out".to_string(),
            rate: Rate::Audio,
            inputs: vec![
                UGenInput::Constant(0.0), // bus index
                UGenInput::UGen {
                    ugen_index: 2,
                    output_index: 0,
                }, // left
            ],
            outputs: vec![],
            special_index: None,
        });

        ugens.push(UGen {
            name: "Out".to_string(),
            rate: Rate::Audio,
            inputs: vec![
                UGenInput::Constant(1.0), // bus index
                UGenInput::UGen {
                    ugen_index: 3,
                    output_index: 0,
                }, // right
            ],
            outputs: vec![],
            special_index: None,
        });
    }

    Ok(SynthDef {
        name: "mainSynth".to_string(),
        params,
        ugens,
        variants: None,
    })
}

// Keep the old function name for compatibility
pub fn generate_synth_def(_patch: &MaxPatch) -> Result<Vec<SynthDef>, CodegenError> {
    Ok(vec![])
}

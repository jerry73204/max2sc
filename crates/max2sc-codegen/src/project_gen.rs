//! Project structure generation

use crate::CodegenError;
use max2sc_max_types::{MaxPatch, OSCConfig};
use max2sc_sc_types::{AudioBus, BusConfig, ControlBus, SCProject, SynthDef};
use std::fs;
use std::path::Path;

/// Project generator with configurable options
pub struct ProjectGenerator {
    skip_spatial: bool,
    skip_multichannel: bool,
    skip_osc: bool,
    simplified_mappings: bool,
}

impl ProjectGenerator {
    /// Create a new project generator with default settings
    pub fn new() -> Self {
        Self {
            skip_spatial: false,
            skip_multichannel: false,
            skip_osc: false,
            simplified_mappings: false,
        }
    }

    /// Skip spatial audio objects during conversion
    pub fn skip_spatial_objects(&mut self) {
        self.skip_spatial = true;
    }

    /// Skip multichannel objects during conversion
    pub fn skip_multichannel_objects(&mut self) {
        self.skip_multichannel = true;
    }

    /// Skip OSC responder generation
    pub fn skip_osc_generation(&mut self) {
        self.skip_osc = true;
    }

    /// Use simplified object mappings
    pub fn use_simplified_mappings(&mut self) {
        self.simplified_mappings = true;
    }

    /// Generate a complete project from a Max patch
    pub fn generate_project(
        &self,
        patch: &MaxPatch,
        output_dir: &Path,
    ) -> Result<SCProject, CodegenError> {
        generate_project_with_options(patch, output_dir, self)
    }

    /// Generate speaker setup from OSC configuration
    pub fn generate_speaker_setup(
        &self,
        config: &OSCConfig,
        output_dir: &Path,
    ) -> Result<(), CodegenError> {
        generate_speaker_config(config, output_dir)
    }
}

impl Default for ProjectGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate project with specific options
fn generate_project_with_options(
    patch: &MaxPatch,
    output_dir: &Path,
    _options: &ProjectGenerator,
) -> Result<SCProject, CodegenError> {
    // Create output directory structure
    create_project_directories(output_dir)?;

    // Generate basic project structure
    let project = SCProject {
        main_file: "main.scd".to_string(),
        synth_defs: generate_basic_synthdefs(patch)?,
        patterns: Vec::new(), // Will be populated later
        bus_config: generate_basic_bus_config(),
        osc_responders: Vec::new(), // Will be populated later
        init_code: generate_init_code(patch),
        cleanup_code: Some(generate_cleanup_code()),
    };

    // Write project files
    write_project_files(&project, output_dir)?;

    Ok(project)
}

/// Generate speaker configuration from OSC config
fn generate_speaker_config(config: &OSCConfig, output_dir: &Path) -> Result<(), CodegenError> {
    let speaker_file = output_dir.join("config").join("speakers.scd");

    let mut content = String::from("// Speaker configuration generated from OSC file\n(\n");

    // Process all speaker arrays
    for (array_idx, speaker_array) in config.speaker_arrays.iter().enumerate() {
        content.push_str(&format!(
            "// Speaker array {}: {}\n",
            array_idx, speaker_array.name
        ));
        for (i, speaker) in speaker_array.speakers.iter().enumerate() {
            content.push_str(&format!(
                "~speakers[{}] = (azimuth: {}, elevation: {}, distance: {}, array: \"{}\");\n",
                i,
                speaker.position.azimuth,
                speaker.position.elevation,
                speaker.position.distance,
                speaker_array.name
            ));
        }
    }

    content.push_str(")\n");

    fs::write(&speaker_file, content).map_err(|e| {
        CodegenError::GenerationFailed(format!("Failed to write speaker config: {e}"))
    })?;

    Ok(())
}

pub fn generate_project(patch: &MaxPatch, output_dir: &Path) -> Result<SCProject, CodegenError> {
    // Create output directory structure
    create_project_directories(output_dir)?;

    // Generate basic project structure
    let project = SCProject {
        main_file: "main.scd".to_string(),
        synth_defs: generate_basic_synthdefs(patch)?,
        patterns: Vec::new(), // Will be populated later
        bus_config: generate_basic_bus_config(),
        osc_responders: Vec::new(), // Will be populated later
        init_code: generate_init_code(patch),
        cleanup_code: Some(generate_cleanup_code()),
    };

    // Write project files
    write_project_files(&project, output_dir)?;

    Ok(project)
}

fn create_project_directories(output_dir: &Path) -> Result<(), CodegenError> {
    fs::create_dir_all(output_dir).map_err(|e| {
        CodegenError::GenerationFailed(format!("Failed to create output directory: {e}"))
    })?;

    fs::create_dir_all(output_dir.join("config")).map_err(|e| {
        CodegenError::GenerationFailed(format!("Failed to create config directory: {e}"))
    })?;

    fs::create_dir_all(output_dir.join("lib")).map_err(|e| {
        CodegenError::GenerationFailed(format!("Failed to create lib directory: {e}"))
    })?;

    fs::create_dir_all(output_dir.join("assets")).map_err(|e| {
        CodegenError::GenerationFailed(format!("Failed to create assets directory: {e}"))
    })?;

    Ok(())
}

fn generate_basic_synthdefs(patch: &MaxPatch) -> Result<Vec<SynthDef>, CodegenError> {
    let mut synth_defs = Vec::new();

    // Count unique object types to determine what synths we need
    let mut dac_found = false;
    let mut cycle_found = false;
    let mut spat5_found = false;

    for box_container in &patch.patcher.boxes {
        let maxclass = &box_container.content.maxclass;
        let text = box_container.content.text.as_deref().unwrap_or("");

        match maxclass.as_str() {
            "newobj" => {
                if text.starts_with("dac~") {
                    dac_found = true;
                } else if text.starts_with("cycle~") {
                    cycle_found = true;
                } else if text.starts_with("spat5") {
                    spat5_found = true;
                }
            }
            "flonum" | "slider" => {
                // Control objects - don't need synthdefs
            }
            _ => {}
        }
    }

    // Generate basic test synth
    if cycle_found || dac_found {
        synth_defs.push(create_test_synth());
    }

    // Generate spatial synth if needed
    if spat5_found {
        synth_defs.push(create_spatial_synth());
    }

    Ok(synth_defs)
}

fn create_test_synth() -> SynthDef {
    use max2sc_sc_types::{Parameter, Rate, UGen, UGenInput, UGenOutput};

    SynthDef {
        name: "testSynth".to_string(),
        params: vec![
            Parameter {
                name: "freq".to_string(),
                default_value: 440.0,
                rate: Rate::Control,
            },
            Parameter {
                name: "amp".to_string(),
                default_value: 0.1,
                rate: Rate::Control,
            },
            Parameter {
                name: "out".to_string(),
                default_value: 0.0,
                rate: Rate::Scalar,
            },
        ],
        ugens: vec![
            UGen {
                name: "SinOsc".to_string(),
                rate: Rate::Audio,
                inputs: vec![
                    UGenInput::Parameter("freq".to_string()),
                    UGenInput::Constant(0.0),
                ],
                outputs: vec![UGenOutput { rate: Rate::Audio }],
                special_index: None,
            },
            UGen {
                name: "Out".to_string(),
                rate: Rate::Audio,
                inputs: vec![
                    UGenInput::Parameter("out".to_string()),
                    UGenInput::UGen {
                        ugen_index: 0,
                        output_index: 0,
                    },
                ],
                outputs: vec![],
                special_index: None,
            },
        ],
        variants: None,
    }
}

fn create_spatial_synth() -> SynthDef {
    use max2sc_sc_types::{Parameter, Rate, UGen, UGenInput, UGenOutput};

    SynthDef {
        name: "spatialSynth".to_string(),
        params: vec![
            Parameter {
                name: "freq".to_string(),
                default_value: 440.0,
                rate: Rate::Control,
            },
            Parameter {
                name: "azimuth".to_string(),
                default_value: 0.0,
                rate: Rate::Control,
            },
            Parameter {
                name: "elevation".to_string(),
                default_value: 0.0,
                rate: Rate::Control,
            },
            Parameter {
                name: "distance".to_string(),
                default_value: 1.0,
                rate: Rate::Control,
            },
            Parameter {
                name: "out".to_string(),
                default_value: 0.0,
                rate: Rate::Scalar,
            },
        ],
        ugens: vec![
            UGen {
                name: "SinOsc".to_string(),
                rate: Rate::Audio,
                inputs: vec![
                    UGenInput::Parameter("freq".to_string()),
                    UGenInput::Constant(0.0),
                ],
                outputs: vec![UGenOutput { rate: Rate::Audio }],
                special_index: None,
            },
            UGen {
                name: "Pan2".to_string(),
                rate: Rate::Audio,
                inputs: vec![
                    UGenInput::UGen {
                        ugen_index: 0,
                        output_index: 0,
                    },
                    UGenInput::Parameter("azimuth".to_string()),
                ],
                outputs: vec![
                    UGenOutput { rate: Rate::Audio },
                    UGenOutput { rate: Rate::Audio },
                ],
                special_index: None,
            },
            UGen {
                name: "Out".to_string(),
                rate: Rate::Audio,
                inputs: vec![
                    UGenInput::Parameter("out".to_string()),
                    UGenInput::UGen {
                        ugen_index: 1,
                        output_index: 0,
                    },
                ],
                outputs: vec![],
                special_index: None,
            },
        ],
        variants: None,
    }
}

fn generate_basic_bus_config() -> BusConfig {
    BusConfig {
        audio_buses: vec![
            AudioBus {
                index: 0,
                num_channels: 2,
                name: Some("Main Output".to_string()),
                private: false,
            },
            AudioBus {
                index: 2,
                num_channels: 16,
                name: Some("WFS Array 1".to_string()),
                private: false,
            },
            AudioBus {
                index: 18,
                num_channels: 32,
                name: Some("WFS Array 2".to_string()),
                private: false,
            },
        ],
        control_buses: vec![
            ControlBus {
                index: 0,
                name: Some("Master Volume".to_string()),
                default_value: 0.8,
            },
            ControlBus {
                index: 1,
                name: Some("Spatial Control".to_string()),
                default_value: 0.0,
            },
        ],
    }
}

fn generate_init_code(patch: &MaxPatch) -> String {
    let num_boxes = patch.patcher.boxes.len();
    let num_lines = patch.patcher.lines.len();

    format!(
        r#"// Auto-generated SuperCollider project
// Converted from Max MSP 8 patch
// Original patch had {num_boxes} objects and {num_lines} connections

(
    var server = Server.default;
    var loadProject;
    
    loadProject = {{
        "Loading project...".postln;
        
        // Load configuration
        var config = ();
        if(File.exists("config/buses.yaml".resolveRelative), {{
            // TODO: Load YAML configuration
            "Loading bus configuration...".postln;
        }});
        
        // Load SynthDefs
        "lib/SynthDefs.scd".resolveRelative.load;
        
        // Set up buses
        ~mainOut = Bus.audio(server, 2);
        ~controlBuses = ();
        
        // Load OSC responders
        if(File.exists("lib/OSCRouters.scd".resolveRelative), {{
            "lib/OSCRouters.scd".resolveRelative.load;
        }});
        
        // Initialize patterns
        if(File.exists("lib/Patterns.scd".resolveRelative), {{
            "lib/Patterns.scd".resolveRelative.load;
        }});
        
        "Project ready!".postln;
        "Main output on bus: %".format(~mainOut.index).postln;
    }};
    
    // Boot server if needed
    if(server.serverRunning.not, {{
        server.waitForBoot(loadProject);
    }}, {{
        loadProject.value;
    }});
)
"#
    )
}

fn generate_cleanup_code() -> String {
    r#"// Cleanup code
(
    "Cleaning up project...".postln;
    
    // Stop all synths
    Server.default.freeAll;
    
    // Reset buses
    Server.default.newBusAllocators;
    
    "Cleanup complete.".postln;
)"#
    .to_string()
}

fn write_project_files(project: &SCProject, output_dir: &Path) -> Result<(), CodegenError> {
    // Write main file
    let main_path = output_dir.join(&project.main_file);
    fs::write(&main_path, &project.init_code)
        .map_err(|e| CodegenError::GenerationFailed(format!("Failed to write main file: {e}")))?;

    // Write cleanup file
    if let Some(cleanup) = &project.cleanup_code {
        let cleanup_path = output_dir.join("cleanup.scd");
        fs::write(&cleanup_path, cleanup).map_err(|e| {
            CodegenError::GenerationFailed(format!("Failed to write cleanup file: {e}"))
        })?;
    }

    // Write SynthDefs file
    let synthdefs_content = generate_synthdefs_file(&project.synth_defs);
    let synthdefs_path = output_dir.join("lib").join("SynthDefs.scd");
    fs::write(&synthdefs_path, synthdefs_content).map_err(|e| {
        CodegenError::GenerationFailed(format!("Failed to write SynthDefs file: {e}"))
    })?;

    // Write bus configuration
    let bus_config_content = generate_bus_config_file(&project.bus_config);
    let bus_config_path = output_dir.join("config").join("buses.yaml");
    fs::write(&bus_config_path, bus_config_content).map_err(|e| {
        CodegenError::GenerationFailed(format!("Failed to write bus config file: {e}"))
    })?;

    // Write README
    let readme_content = generate_readme();
    let readme_path = output_dir.join("README.md");
    fs::write(&readme_path, readme_content)
        .map_err(|e| CodegenError::GenerationFailed(format!("Failed to write README file: {e}")))?;

    Ok(())
}

fn generate_synthdefs_file(synth_defs: &[SynthDef]) -> String {
    let mut content = String::from("// Auto-generated SynthDefs\n\n");

    for synth_def in synth_defs {
        content.push_str(&format!("// SynthDef: {}\n", synth_def.name));
        content.push_str(&format!("SynthDef(\\{}, {{\n", synth_def.name));

        // Add parameters
        content.push_str("    arg ");
        for (i, param) in synth_def.params.iter().enumerate() {
            if i > 0 {
                content.push_str(", ");
            }
            content.push_str(&format!("{}={}", param.name, param.default_value));
        }
        content.push_str(";\n");

        // Add simple signal chain (placeholder)
        content.push_str("    var sig = SinOsc.ar(freq, 0, amp);\n");
        content.push_str("    Out.ar(out, sig);\n");

        content.push_str("}).add;\n\n");
    }

    content
}

fn generate_bus_config_file(bus_config: &BusConfig) -> String {
    let mut content = String::from("# Auto-generated bus configuration\n\n");

    content.push_str("audio_buses:\n");
    for bus in &bus_config.audio_buses {
        content.push_str(&format!("  - index: {}\n", bus.index));
        content.push_str(&format!("    channels: {}\n", bus.num_channels));
        if let Some(name) = &bus.name {
            content.push_str(&format!("    name: \"{name}\"\n"));
        }
        content.push_str(&format!("    private: {}\n", bus.private));
    }

    content.push_str("\ncontrol_buses:\n");
    for bus in &bus_config.control_buses {
        content.push_str(&format!("  - index: {}\n", bus.index));
        content.push_str(&format!("    default: {}\n", bus.default_value));
        if let Some(name) = &bus.name {
            content.push_str(&format!("    name: \"{name}\"\n"));
        }
    }

    content
}

fn generate_readme() -> String {
    r#"# Converted SuperCollider Project

This project was automatically generated from a Max MSP 8 patch using max2sc.

## Project Structure

- `main.scd` - Main project file, run this to start
- `cleanup.scd` - Cleanup code to stop everything
- `lib/` - SuperCollider libraries and SynthDefs
  - `SynthDefs.scd` - All converted SynthDefs
- `config/` - Configuration files
  - `buses.yaml` - Audio and control bus configuration
- `assets/` - Audio files and other assets

## Usage

1. Open SuperCollider
2. Load and run `main.scd`
3. Use `cleanup.scd` to stop and clean up

## Notes

This is an automated conversion. Some manual adjustment may be required for complex patches.
The spatial audio routing has been simplified but preserves the basic structure.
"#
    .to_string()
}

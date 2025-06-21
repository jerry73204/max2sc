//! Configuration file generation

use crate::CodegenError;
use max2sc_max_types::{MaxPatch, OSCConfig};
use std::fs;
use std::path::Path;

/// Generate configuration files for the SC project
pub fn generate_config_files(
    patch: &MaxPatch,
    osc_config: Option<&OSCConfig>,
    output_dir: &Path,
) -> Result<(), CodegenError> {
    let config_dir = output_dir.join("config");

    // Generate bus configuration
    generate_bus_config(&config_dir)?;

    // Generate speaker configuration if available
    if let Some(osc) = osc_config {
        generate_speaker_config(osc, &config_dir)?;
    }

    // Generate server options
    generate_server_options(patch, &config_dir)?;

    Ok(())
}

/// Generate bus configuration file
fn generate_bus_config(config_dir: &Path) -> Result<(), CodegenError> {
    let bus_config = r#"# Bus Configuration
# Auto-generated from Max patch

audio_buses:
  - index: 0
    num_channels: 2
    name: "Main Output"
    private: false
  
  - index: 2
    num_channels: 16
    name: "WFS Array 1"
    private: false
  
  - index: 18
    num_channels: 32
    name: "WFS Array 2"
    private: false
  
  - index: 50
    num_channels: 8
    name: "HOA Bus"
    private: false

control_buses:
  - index: 0
    name: "Master Volume"
    default_value: 0.8
  
  - index: 1
    name: "Spatial X"
    default_value: 0.0
  
  - index: 2
    name: "Spatial Y"
    default_value: 0.0
  
  - index: 3
    name: "Spatial Z"
    default_value: 0.0
"#;

    let bus_path = config_dir.join("buses.yaml");
    fs::write(bus_path, bus_config).map_err(|e| {
        CodegenError::GenerationFailed(format!("Failed to write bus config: {}", e))
    })?;

    Ok(())
}

/// Generate speaker configuration from OSC data
fn generate_speaker_config(osc_config: &OSCConfig, config_dir: &Path) -> Result<(), CodegenError> {
    let mut yaml_content =
        String::from("# Speaker Configuration\n# Auto-generated from OSC configuration\n\n");

    yaml_content.push_str("speaker_arrays:\n");

    for array in &osc_config.speaker_arrays {
        yaml_content.push_str(&format!("  - bus_id: {}\n", array.bus_id));
        yaml_content.push_str(&format!("    format: \"{}\"\n", array.format));
        yaml_content.push_str(&format!("    name: \"{}\"\n", array.name));
        yaml_content.push_str(&format!("    speakers:\n"));

        for speaker in &array.speakers {
            yaml_content.push_str(&format!("      - id: {}\n", speaker.id));
            yaml_content.push_str(&format!("        position:\n"));
            yaml_content.push_str(&format!(
                "          azimuth: {}\n",
                speaker.position.azimuth
            ));
            yaml_content.push_str(&format!(
                "          elevation: {}\n",
                speaker.position.elevation
            ));
            yaml_content.push_str(&format!(
                "          distance: {}\n",
                speaker.position.distance
            ));
            yaml_content.push_str(&format!("        delay: {}\n", speaker.delay));
            yaml_content.push_str(&format!("        gain: {}\n", speaker.gain));
        }
        yaml_content.push_str("\n");
    }

    let speaker_path = config_dir.join("speakers.yaml");
    fs::write(speaker_path, yaml_content).map_err(|e| {
        CodegenError::GenerationFailed(format!("Failed to write speaker config: {}", e))
    })?;

    Ok(())
}

/// Generate server options file
fn generate_server_options(patch: &MaxPatch, config_dir: &Path) -> Result<(), CodegenError> {
    // Analyze patch to determine server requirements
    let mut max_channels = 2; // Default stereo
    let mut uses_spatial = false;

    for box_container in &patch.patcher.boxes {
        let text = box_container.content.text.as_deref().unwrap_or("");

        if text.starts_with("mc.dac~") || text.starts_with("mc.adc~") {
            // Extract channel count
            let parts: Vec<&str> = text.split_whitespace().collect();
            for part in parts.iter().skip(1) {
                if let Ok(ch) = part.parse::<u32>() {
                    max_channels = max_channels.max(ch);
                }
            }
        }

        if text.starts_with("spat5") || text.starts_with("pan") {
            uses_spatial = true;
        }
    }

    // Determine appropriate settings
    let num_outputs = if uses_spatial {
        64
    } else {
        max_channels.max(8)
    };
    let num_inputs = max_channels.max(8);
    let memory_size = if uses_spatial { 262144 } else { 65536 };

    let server_options = format!(
        r#"// Server Options
// Auto-generated based on patch analysis

(
    s = Server.default;
    
    s.options.numOutputBusChannels = {};
    s.options.numInputBusChannels = {};
    s.options.numAudioBusChannels = 1024;
    s.options.numControlBusChannels = 4096;
    s.options.numBuffers = 1024;
    s.options.numWireBufs = 256;
    s.options.memSize = {};
    s.options.maxNodes = 2048;
    s.options.maxSynthDefs = 512;
    
    // Device settings (platform-specific)
    // s.options.device = "Your Audio Device";
    
    "Server options configured.".postln;
)
"#,
        num_outputs, num_inputs, memory_size
    );

    let server_path = config_dir.join("server_options.scd");
    fs::write(server_path, server_options).map_err(|e| {
        CodegenError::GenerationFailed(format!("Failed to write server options: {}", e))
    })?;

    Ok(())
}

/// Generate SuperCollider configuration loader
pub fn generate_config_loader() -> String {
    r#"// Configuration Loader
// Loads YAML configuration files into SC environment

(
    var loadYAML, loadBusConfig, loadSpeakerConfig;
    
    // Simple YAML parser (basic implementation)
    loadYAML = { |path|
        var lines, result = ();
        
        if(File.exists(path), {
            lines = File.readAllString(path).split($\n);
            // TODO: Implement proper YAML parsing
            "Loaded configuration from: %".format(path).postln;
        }, {
            "Configuration file not found: %".format(path).warn;
        });
        
        result;
    };
    
    // Load bus configuration
    loadBusConfig = {
        var config = loadYAML.("config/buses.yaml".resolveRelative);
        
        ~audioBuses = ();
        ~controlBuses = ();
        
        // TODO: Parse and create buses from config
        
        "Bus configuration loaded.".postln;
    };
    
    // Load speaker configuration
    loadSpeakerConfig = {
        var config = loadYAML.("config/speakers.yaml".resolveRelative);
        
        ~speakerArrays = ();
        
        // TODO: Parse speaker arrays from config
        
        "Speaker configuration loaded.".postln;
    };
    
    // Export functions
    ~loadBusConfig = loadBusConfig;
    ~loadSpeakerConfig = loadSpeakerConfig;
    
    "Configuration loader initialized.".postln;
)
"#
    .to_string()
}

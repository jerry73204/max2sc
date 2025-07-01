//! Main conversion logic and runner

use crate::cli::CliArgs;
use crate::config::ConfigGenerator;
use crate::validation::Validator;
use eyre::{eyre, Result, WrapErr};
use max2sc_analyzer::graph::{SignalFlowGraph, SignalFlowGraphExt};
use max2sc_codegen::project_gen::ProjectGenerator;
use max2sc_max_types::parse_osc_text;
use max2sc_parser::json_parser::parse_patch_file;
use std::fs;
use std::time::Instant;
use tracing::{debug, info, warn};

/// Main conversion runner
pub struct ConversionRunner {
    args: CliArgs,
}

impl ConversionRunner {
    /// Create a new conversion runner
    pub fn new(args: CliArgs) -> Self {
        Self { args }
    }

    /// Run conversion and return the SCProject without writing to disk
    pub fn _run_conversion(&self) -> Result<max2sc_sc_types::SCProject> {
        // Load and parse input files
        let (patch, _speaker_config) = self.load_input_files()?;

        // Analyze the patch
        let _analysis = self.analyze_patch(&patch)?;

        // Generate SuperCollider project
        let generator = ProjectGenerator::new();
        Ok(generator.generate_project(&patch, &self.args.output)?)
    }

    /// Run the complete conversion process
    pub fn run(self) -> Result<()> {
        let start_time = Instant::now();

        info!("🚀 Starting max2sc conversion");
        info!("Input: {}", self.args.input.display());
        info!("Output: {}", self.args.output.display());

        // Handle special commands first
        if self.args.generate_config {
            return self.generate_config_files();
        }

        // Validate all inputs
        self.validate_inputs()?;

        if self.args.dry_run {
            info!("✓ Dry run completed successfully - no output generated");
            return Ok(());
        }

        // Load and parse input files
        let (patch, speaker_config) = self.load_input_files()?;

        // Analyze the patch
        let analysis = self.analyze_patch(&patch)?;

        // Generate SuperCollider project
        self.generate_project(&patch, &analysis, speaker_config.as_ref())?;

        let duration = start_time.elapsed();
        info!(
            "✅ Conversion completed successfully in {:.2}s",
            duration.as_secs_f64()
        );
        info!("Output project: {}", self.args.output.display());

        Ok(())
    }

    /// Generate example configuration files
    fn generate_config_files(&self) -> Result<()> {
        info!("Generating example configuration files...");

        let output_dir = if self.args.output.to_string_lossy() == "." {
            std::env::current_dir()?.join("max2sc_examples")
        } else {
            self.args.output.clone()
        };

        ConfigGenerator::generate_examples(&output_dir)?;

        info!(
            "✓ Example configuration files generated in: {}",
            output_dir.display()
        );
        info!("  - example_speakers.txt");
        info!("  - max2sc_config.yaml");
        info!("  - project_template/");

        Ok(())
    }

    /// Validate all CLI inputs and files
    fn validate_inputs(&self) -> Result<()> {
        // Don't validate input file if we're just generating config
        if !self.args.generate_config {
            // Check that input file was provided
            if self.args.input.to_string_lossy() == "dummy.maxpat" {
                return Err(eyre!(
                    "Input Max patch file is required. Use --help for usage information."
                ));
            }

            Validator::validate_all(
                &self.args.input,
                &self.args.output,
                self.args.speaker_config.as_deref(),
                self.args.force,
            )?;
        }

        // Additional validation based on options
        if self.args.options.skip_spatial && self.args.speaker_config.is_some() {
            warn!("Speaker config provided but spatial conversion is disabled");
        }

        if self.args.options.simplified && !self.args.options.skip_spatial {
            warn!("Simplified mode with spatial audio may produce unexpected results");
        }

        Ok(())
    }

    /// Load and parse input files
    fn load_input_files(
        &self,
    ) -> Result<(
        max2sc_max_types::MaxPatch,
        Option<max2sc_max_types::OSCConfig>,
    )> {
        info!("📖 Loading input files...");

        // Parse Max patch
        let patch = parse_patch_file(&self.args.input).wrap_err_with(|| {
            format!("Failed to parse Max patch: {}", self.args.input.display())
        })?;

        info!(
            "✓ Loaded Max patch with {} objects",
            patch.patcher.boxes.len()
        );

        if self.args.debug {
            debug!("Patch details:");
            debug!("  - Objects: {}", patch.patcher.boxes.len());
            debug!("  - Connections: {}", patch.patcher.lines.len());
        }

        // Parse speaker configuration if provided
        let speaker_config = if let Some(speaker_path) = &self.args.speaker_config {
            info!("📖 Loading speaker configuration...");
            let content = std::fs::read_to_string(speaker_path).wrap_err_with(|| {
                format!("Failed to read speaker config: {}", speaker_path.display())
            })?;
            let config = parse_osc_text(&content).map_err(|e| {
                eyre!(
                    "Failed to parse speaker config {}: {}",
                    speaker_path.display(),
                    e
                )
            })?;
            info!(
                "✓ Loaded speaker config with {} speaker arrays",
                config.speaker_arrays.len()
            );
            Some(config)
        } else {
            None
        };

        Ok((patch, speaker_config))
    }

    /// Analyze the patch structure and signal flow
    fn analyze_patch(&self, patch: &max2sc_max_types::MaxPatch) -> Result<AnalysisResults> {
        info!("🔍 Analyzing patch structure...");

        // Build signal flow graph
        let mut graph = SignalFlowGraph::new_graph();
        graph.build_from_patch(patch)?;

        info!(
            "✓ Built signal flow graph with {} nodes",
            graph.node_count()
        );

        if self.args.debug {
            debug!("Graph analysis:");
            debug!("  - Audio sources: {}", graph.count_audio_sources());
            debug!("  - Audio sinks: {}", graph.count_audio_sinks());
            debug!("  - Control objects: {}", graph.count_control_objects());
            debug!("  - Spatial objects: {}", graph.count_spatial_objects());
        }

        // Detect object types and complexity
        let object_stats = self.analyze_object_types(patch);

        if self.args.verbose {
            info!("Object analysis:");
            info!("  - Total objects: {}", object_stats.total_objects);
            info!("  - Audio objects: {}", object_stats.audio_objects);
            info!("  - Spatial objects: {}", object_stats.spatial_objects);
            info!(
                "  - Multichannel objects: {}",
                object_stats.multichannel_objects
            );
            info!("  - Unknown objects: {}", object_stats.unknown_objects);
        }

        // Warn about potential issues
        if object_stats.unknown_objects > 0 {
            warn!(
                "Found {} unknown objects that may not convert properly",
                object_stats.unknown_objects
            );
        }

        if object_stats.spatial_objects > 0 && self.args.options.skip_spatial {
            warn!(
                "Found {} spatial objects but spatial conversion is disabled",
                object_stats.spatial_objects
            );
        }

        Ok(AnalysisResults {
            graph,
            object_stats,
        })
    }

    /// Analyze object types in the patch
    fn analyze_object_types(&self, patch: &max2sc_max_types::MaxPatch) -> ObjectStats {
        let mut stats = ObjectStats {
            total_objects: patch.patcher.boxes.len(),
            ..Default::default()
        };

        for obj in &patch.patcher.boxes {
            if let Some(text) = &obj.content.text {
                let obj_name = text.split_whitespace().next().unwrap_or("");

                if obj_name.ends_with('~') || obj_name.starts_with("spat5") {
                    stats.audio_objects += 1;
                }

                if obj_name.starts_with("spat5") || obj_name.starts_with("pan") {
                    stats.spatial_objects += 1;
                }

                if obj_name.starts_with("mc.") {
                    stats.multichannel_objects += 1;
                }

                // Check against known objects
                if !is_known_object(obj_name) {
                    stats.unknown_objects += 1;
                    if self.args.debug {
                        debug!("Unknown object: {}", obj_name);
                    }
                }
            }
        }

        stats
    }

    /// Generate the SuperCollider project
    fn generate_project(
        &self,
        patch: &max2sc_max_types::MaxPatch,
        analysis: &AnalysisResults,
        speaker_config: Option<&max2sc_max_types::OSCConfig>,
    ) -> Result<()> {
        info!("🏗️  Generating SuperCollider project...");

        // Create output directory
        if self.args.output.exists() && self.args.force {
            warn!("Removing existing output directory");
            fs::remove_dir_all(&self.args.output)?;
        }

        fs::create_dir_all(&self.args.output)?;

        // Create project generator with options
        let mut generator = ProjectGenerator::new();

        // Apply conversion options
        if self.args.options.skip_spatial {
            generator.skip_spatial_objects();
        }

        if self.args.options.skip_multichannel {
            generator.skip_multichannel_objects();
        }

        if !self.args.options.generate_osc {
            generator.skip_osc_generation();
        }

        if self.args.options.simplified {
            generator.use_simplified_mappings();
        }

        // Generate the project
        generator.generate_project(patch, &self.args.output)?;

        // Generate speaker configuration if provided
        if let Some(config) = speaker_config {
            generator.generate_speaker_setup(config, &self.args.output)?;
        }

        // Generate additional files based on analysis
        self.generate_analysis_report(analysis)?;

        info!("✓ SuperCollider project generated");

        // List generated files
        self.list_generated_files()?;

        Ok(())
    }

    /// Generate analysis report
    fn generate_analysis_report(&self, analysis: &AnalysisResults) -> Result<()> {
        if !self.args.verbose {
            return Ok(());
        }

        let report_path = self.args.output.join("conversion_report.md");

        let report = format!(
            r#"# Conversion Report
            
Generated by max2sc on {}

## Input Analysis

- **Total objects**: {}
- **Audio objects**: {}
- **Spatial objects**: {}
- **Multichannel objects**: {}
- **Unknown objects**: {}

## Signal Flow Graph

- **Total nodes**: {}
- **Audio sources**: {}
- **Audio sinks**: {}
- **Control objects**: {}

## Conversion Options

- **Skip spatial**: {}
- **Skip multichannel**: {}
- **Generate OSC**: {}
- **Simplified mappings**: {}
- **Target SC version**: {}

## Notes

{}
"#,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            analysis.object_stats.total_objects,
            analysis.object_stats.audio_objects,
            analysis.object_stats.spatial_objects,
            analysis.object_stats.multichannel_objects,
            analysis.object_stats.unknown_objects,
            analysis.graph.node_count(),
            analysis.graph.count_audio_sources(),
            analysis.graph.count_audio_sinks(),
            analysis.graph.count_control_objects(),
            self.args.options.skip_spatial,
            self.args.options.skip_multichannel,
            self.args.options.generate_osc,
            self.args.options.simplified,
            self.args.options.sc_version,
            if analysis.object_stats.unknown_objects > 0 {
                format!(
                    "⚠️  {} unknown objects may require manual conversion",
                    analysis.object_stats.unknown_objects
                )
            } else {
                "✅ All objects have known conversions".to_string()
            }
        );

        fs::write(&report_path, report)?;
        info!("✓ Generated conversion report: {}", report_path.display());

        Ok(())
    }

    /// List all generated files
    fn list_generated_files(&self) -> Result<()> {
        if !self.args.verbose {
            return Ok(());
        }

        info!("Generated files:");

        fn list_dir_recursive(dir: &std::path::Path, prefix: &str) -> Result<()> {
            let entries = fs::read_dir(dir)?;
            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                let name = path.file_name().unwrap().to_string_lossy();

                if path.is_dir() {
                    info!("{}📁 {}/", prefix, name);
                    list_dir_recursive(&path, &format!("{prefix}  "))?;
                } else {
                    info!("{}📄 {}", prefix, name);
                }
            }
            Ok(())
        }

        list_dir_recursive(&self.args.output, "  ")?;

        Ok(())
    }
}

/// Results from patch analysis
#[derive(Debug)]
struct AnalysisResults {
    graph: SignalFlowGraph,
    object_stats: ObjectStats,
}

/// Statistics about object types in the patch
#[derive(Debug, Default)]
struct ObjectStats {
    total_objects: usize,
    audio_objects: usize,
    spatial_objects: usize,
    multichannel_objects: usize,
    unknown_objects: usize,
}

/// Check if an object name is known to the converter
fn is_known_object(name: &str) -> bool {
    matches!(
        name,
        // Audio I/O
        "dac~" | "adc~" | "ezdac~" | "ezadc~" | "out~" | "in~" |
        // Multichannel
        "mc.pack~" | "mc.unpack~" | "mc.dac~" | "mc.adc~" | "mc.live.gain~" |
        // Spatial
        "pan~" | "pan2~" | "pan4~" | "pan8~" | "stereo~" | "matrix~" |
        // SPAT5 (basic support)
        "spat5.panoramix~" | "spat5.pan~" | "spat5.osc.route" |
        // Basic objects
        "flonum" | "slider" | "dial" | "number" | "toggle" | "button" |
        "gain~" | "live.gain~" | "live.dial" | "live.slider" |
        // Math/Logic
        "+" | "-" | "*" | "/" | "%" | "==" | "!=" | "<" | ">" | 
        "&&" | "||" | "!" | "abs" | "sqrt" | "pow" | "sin" | "cos" |
        // Control
        "metro" | "counter" | "select" | "route" | "unpack" | "pack" |
        "loadmess" | "loadbang" | "trigger" | "delay" | "pipe" |
        // Audio processing
        "cycle~" | "noise~" | "saw~" | "tri~" | "rect~" | "phasor~" |
        "filtergraph~" | "biquad~" | "onepole~" | "lores~" | "hires~" |
        "delay~" | "delwrite~" | "delread~" | "tapin~" | "tapout~" |
        "reverb~" | "freeverb~" | "gverb~" | "*~" | "+~" | "-~" | "/~"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{CliArgs, ConversionOptions};
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_args(input: PathBuf, output: PathBuf) -> CliArgs {
        CliArgs {
            input,
            output,
            speaker_config: None,
            verbose: false,
            debug: false,
            force: true,
            dry_run: false,
            generate_config: false,
            options: ConversionOptions::default(),
        }
    }

    #[test]
    fn test_generate_config_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path().to_path_buf();

        let args = CliArgs {
            generate_config: true,
            output: output_path.clone(),
            ..create_test_args(PathBuf::from("dummy.maxpat"), output_path)
        };

        let runner = ConversionRunner::new(args);
        runner.generate_config_files()?;

        // Check that files were created
        assert!(temp_dir.path().join("example_speakers.txt").exists());
        assert!(temp_dir.path().join("max2sc_config.yaml").exists());

        Ok(())
    }

    #[test]
    fn test_is_known_object() {
        assert!(is_known_object("dac~"));
        assert!(is_known_object("mc.pack~"));
        assert!(is_known_object("spat5.panoramix~"));
        assert!(is_known_object("cycle~"));
        assert!(!is_known_object("unknown_object"));
        assert!(!is_known_object("custom.object~"));
    }
}

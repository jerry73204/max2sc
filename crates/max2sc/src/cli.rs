//! CLI argument parsing and configuration

use clap::{Arg, ArgAction, Command, ValueHint};
use eyre::{eyre, Result};
use std::path::PathBuf;

/// CLI arguments for max2sc converter
#[derive(Debug, Clone)]
pub struct CliArgs {
    /// Input Max patch file (.maxpat)
    pub input: PathBuf,
    /// Output directory for SuperCollider project
    pub output: PathBuf,
    /// Optional speaker configuration file (OSC format)
    pub speaker_config: Option<PathBuf>,
    /// Enable verbose logging
    pub verbose: bool,
    /// Enable debug mode with extra validation
    pub debug: bool,
    /// Force overwrite existing output directory
    pub force: bool,
    /// Dry run - validate input without generating output
    pub dry_run: bool,
    /// Generate example configuration files
    pub generate_config: bool,
    /// Conversion options
    pub options: ConversionOptions,
}

/// Options for controlling the conversion process
#[derive(Debug, Clone)]
pub struct ConversionOptions {
    /// Skip spatial audio conversion
    pub skip_spatial: bool,
    /// Skip multichannel conversion
    pub skip_multichannel: bool,
    /// Generate OSC responders
    pub generate_osc: bool,
    /// Use simplified object mappings
    pub simplified: bool,
    /// Target SuperCollider version (3.12, 3.13, etc.)
    pub sc_version: String,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            skip_spatial: false,
            skip_multichannel: false,
            generate_osc: true,
            simplified: false,
            sc_version: "3.13".to_string(),
        }
    }
}

impl CliArgs {
    /// Parse command line arguments
    pub fn parse() -> Result<Self> {
        let matches = Command::new("max2sc")
            .version(env!("CARGO_PKG_VERSION"))
            .author("Max to SuperCollider Converter")
            .about("Convert Max MSP 8 projects to SuperCollider with spatial audio support")
            .long_about(
                "max2sc converts Max MSP 8 patches to SuperCollider projects, with special \
                support for spatial audio objects like SPAT5, multichannel routing, and \
                complex signal flow graphs.",
            )
            .arg(
                Arg::new("input")
                    .help("Input Max patch file (.maxpat)")
                    .required(false)
                    .value_hint(ValueHint::FilePath)
                    .value_parser(clap::value_parser!(PathBuf)),
            )
            .arg(
                Arg::new("output")
                    .short('o')
                    .long("output")
                    .help("Output directory for SuperCollider project")
                    .value_hint(ValueHint::DirPath)
                    .value_parser(clap::value_parser!(PathBuf))
                    .required(true),
            )
            .arg(
                Arg::new("speaker-config")
                    .short('s')
                    .long("speaker-config")
                    .help("Speaker configuration file (OSC format)")
                    .value_hint(ValueHint::FilePath)
                    .value_parser(clap::value_parser!(PathBuf)),
            )
            .arg(
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .help("Enable verbose logging")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("debug")
                    .short('d')
                    .long("debug")
                    .help("Enable debug mode with extra validation")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("force")
                    .short('f')
                    .long("force")
                    .help("Force overwrite existing output directory")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("dry-run")
                    .long("dry-run")
                    .help("Validate input without generating output")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("generate-config")
                    .long("generate-config")
                    .help("Generate example configuration files")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("skip-spatial")
                    .long("skip-spatial")
                    .help("Skip spatial audio conversion")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("skip-multichannel")
                    .long("skip-multichannel")
                    .help("Skip multichannel conversion")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("no-osc")
                    .long("no-osc")
                    .help("Don't generate OSC responders")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("simplified")
                    .long("simplified")
                    .help("Use simplified object mappings")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("sc-version")
                    .long("sc-version")
                    .help("Target SuperCollider version")
                    .value_parser(["3.12", "3.13", "3.14"])
                    .default_value("3.13"),
            )
            .get_matches();

        let input = matches
            .get_one::<PathBuf>("input")
            .cloned()
            .unwrap_or_else(|| PathBuf::from("dummy.maxpat"));
        let output = matches.get_one::<PathBuf>("output").unwrap().clone();
        let speaker_config = matches.get_one::<PathBuf>("speaker-config").cloned();

        let verbose = matches.get_flag("verbose");
        let debug = matches.get_flag("debug");
        let force = matches.get_flag("force");
        let dry_run = matches.get_flag("dry-run");
        let generate_config = matches.get_flag("generate-config");

        let options = ConversionOptions {
            skip_spatial: matches.get_flag("skip-spatial"),
            skip_multichannel: matches.get_flag("skip-multichannel"),
            generate_osc: !matches.get_flag("no-osc"),
            simplified: matches.get_flag("simplified"),
            sc_version: matches.get_one::<String>("sc-version").unwrap().clone(),
        };

        Ok(Self {
            input,
            output,
            speaker_config,
            verbose,
            debug,
            force,
            dry_run,
            generate_config,
            options,
        })
    }

    /// Get help text for the CLI
    pub fn help() -> String {
        Command::new("max2sc")
            .version(env!("CARGO_PKG_VERSION"))
            .author("Max to SuperCollider Converter")
            .about("Convert Max MSP 8 projects to SuperCollider with spatial audio support")
            .long_about(
                "max2sc converts Max MSP 8 patches to SuperCollider projects, with special \
                support for spatial audio objects like SPAT5, multichannel routing, and \
                complex signal flow graphs.\n\n\
                EXAMPLES:\n  \
                max2sc input.maxpat -o output_project/\n  \
                max2sc patch.maxpat -o sc_project/ -s speakers.txt -v\n  \
                max2sc complex.maxpat -o out/ --skip-spatial --simplified\n  \
                max2sc --generate-config\n  \
                max2sc input.maxpat -o test/ --dry-run",
            )
            .render_long_help()
            .to_string()
    }
}

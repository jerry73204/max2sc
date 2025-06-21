use clap::{Arg, ArgAction, Command};
use eyre::{eyre, Result, WrapErr};
use max2sc_analyzer::graph::SignalFlowGraph;
use max2sc_codegen::project_gen::ProjectGenerator;
use max2sc_parser::json_parser::parse_patch_file;
use std::path::{Path, PathBuf};
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod cli;
mod config;
mod conversion;
mod validation;

use cli::CliArgs;
use conversion::ConversionRunner;

fn main() -> Result<()> {
    color_eyre::install()?;

    // Initialize logging
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Parse command line arguments
    let args = CliArgs::parse()?;

    // Set up logging level based on verbosity
    if args.verbose {
        tracing::subscriber::set_global_default(
            tracing_subscriber::registry()
                .with(tracing_subscriber::EnvFilter::new("debug"))
                .with(tracing_subscriber::fmt::layer()),
        )?;
    }

    // Run the conversion
    let runner = ConversionRunner::new(args);
    runner.run()
}

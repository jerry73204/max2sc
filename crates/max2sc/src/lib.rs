//! max2sc - Convert Max MSP 8 projects to SuperCollider

pub mod cli;
pub mod config;
pub mod conversion;
pub mod validation;

pub use conversion::ConversionRunner;

/// Convenience alias for the convert function
pub mod convert {
    use crate::conversion::ConversionRunner;
    use eyre::Result;
    use max2sc_sc_types::SCProject;
    use std::path::Path;

    /// Convert a Max patch file to a SuperCollider project
    pub fn convert_patch(input_path: &Path, output_dir: &Path) -> Result<SCProject> {
        // Create temporary args for conversion
        let args = crate::cli::CliArgs {
            input: input_path.to_path_buf(),
            output: output_dir.to_path_buf(),
            speaker_config: None,
            verbose: false,
            debug: false,
            force: false,
            dry_run: false,
            generate_config: false,
            options: crate::cli::ConversionOptions::default(),
        };

        let runner = ConversionRunner::new(args);
        runner._run_conversion()
    }
}

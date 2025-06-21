//! Input validation and file checking

use eyre::{eyre, Result, WrapErr};
use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};

/// Validates CLI arguments and input files
pub struct Validator;

impl Validator {
    /// Validate input file exists and is a valid Max patch
    pub fn validate_input_file<P: AsRef<Path>>(path: P) -> Result<()> {
        let path = path.as_ref();

        // Check if file exists
        if !path.exists() {
            return Err(eyre!("Input file does not exist: {}", path.display()));
        }

        // Check if it's a file (not directory)
        if !path.is_file() {
            return Err(eyre!("Input path is not a file: {}", path.display()));
        }

        // Check file extension
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("maxpat") => {
                info!("✓ Input file extension is valid: .maxpat");
            }
            Some(ext) => {
                warn!(
                    "Input file has unusual extension: .{} (expected .maxpat)",
                    ext
                );
            }
            None => {
                warn!("Input file has no extension (expected .maxpat)");
            }
        }

        // Check if file is readable
        fs::File::open(path)
            .wrap_err_with(|| format!("Cannot read input file: {}", path.display()))?;

        // Basic JSON structure validation
        let content = fs::read_to_string(path)
            .wrap_err_with(|| format!("Cannot read file content: {}", path.display()))?;

        // Check if it looks like a Max patch (basic heuristics)
        if !content.trim_start().starts_with('{') {
            return Err(eyre!("Input file does not appear to be a JSON file"));
        }

        if !content.contains("\"patcher\"") {
            warn!("Input file may not be a Max patch (no 'patcher' key found)");
        }

        let file_size = content.len();
        info!("✓ Input file validated ({} bytes)", file_size);

        if file_size > 10_000_000 {
            warn!(
                "Input file is very large ({} MB), conversion may take time",
                file_size / 1_000_000
            );
        }

        Ok(())
    }

    /// Validate speaker configuration file if provided
    pub fn validate_speaker_config<P: AsRef<Path>>(path: Option<P>) -> Result<()> {
        if let Some(path) = path {
            let path = path.as_ref();

            if !path.exists() {
                return Err(eyre!(
                    "Speaker config file does not exist: {}",
                    path.display()
                ));
            }

            if !path.is_file() {
                return Err(eyre!(
                    "Speaker config path is not a file: {}",
                    path.display()
                ));
            }

            // Check file extension
            match path.extension().and_then(|ext| ext.to_str()) {
                Some("txt") => {
                    debug!("Speaker config file extension: .txt");
                }
                Some(ext) => {
                    warn!(
                        "Speaker config has unusual extension: .{} (expected .txt)",
                        ext
                    );
                }
                None => {
                    warn!("Speaker config has no extension (expected .txt)");
                }
            }

            // Check if file is readable
            let content = fs::read_to_string(path)
                .wrap_err_with(|| format!("Cannot read speaker config: {}", path.display()))?;

            // Basic OSC format validation
            if content.trim().is_empty() {
                return Err(eyre!("Speaker config file is empty"));
            }

            // Look for OSC-like patterns
            let has_osc_pattern = content
                .lines()
                .any(|line| line.trim_start().starts_with('/') || line.contains("aed"));

            if !has_osc_pattern {
                warn!("Speaker config may not be in OSC format (no '/' or 'aed' patterns found)");
            }

            info!(
                "✓ Speaker config validated ({} lines)",
                content.lines().count()
            );
        }

        Ok(())
    }

    /// Validate output directory and permissions
    pub fn validate_output_directory<P: AsRef<Path>>(path: P, force: bool) -> Result<()> {
        let path = path.as_ref();

        // Check if output path already exists
        if path.exists() {
            if path.is_file() {
                return Err(eyre!(
                    "Output path exists and is a file: {}",
                    path.display()
                ));
            }

            if path.is_dir() {
                let entries = fs::read_dir(path).wrap_err_with(|| {
                    format!("Cannot read output directory: {}", path.display())
                })?;

                let entry_count = entries.count();
                if entry_count > 0 && !force {
                    return Err(eyre!(
                        "Output directory is not empty ({} items). Use --force to overwrite.",
                        entry_count
                    ));
                }

                if entry_count > 0 {
                    warn!(
                        "Output directory exists and will be overwritten ({} items)",
                        entry_count
                    );
                }
            }
        } else {
            // Check if parent directory exists and is writable
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    info!("Creating parent directories: {}", parent.display());
                    fs::create_dir_all(parent).wrap_err_with(|| {
                        format!("Cannot create parent directory: {}", parent.display())
                    })?;
                }

                // Test write permissions by creating a temporary file
                let test_file = parent.join(".max2sc_write_test");
                fs::write(&test_file, "test").wrap_err_with(|| {
                    format!("No write permission in directory: {}", parent.display())
                })?;
                fs::remove_file(&test_file).ok(); // Clean up, ignore errors
            }
        }

        info!("✓ Output directory validated: {}", path.display());
        Ok(())
    }

    /// Validate all CLI arguments and files
    pub fn validate_all(
        input: &Path,
        output: &Path,
        speaker_config: Option<&Path>,
        force: bool,
    ) -> Result<()> {
        info!("Validating input files and directories...");

        Self::validate_input_file(input).wrap_err("Input file validation failed")?;

        Self::validate_speaker_config(speaker_config)
            .wrap_err("Speaker config validation failed")?;

        Self::validate_output_directory(output, force)
            .wrap_err("Output directory validation failed")?;

        info!("✓ All validations passed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validate_nonexistent_file() {
        let result = Validator::validate_input_file("/nonexistent/file.maxpat");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[test]
    fn test_validate_valid_json_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.maxpat");

        fs::write(&file_path, r#"{"patcher": {"boxes": []}}"#)?;

        let result = Validator::validate_input_file(&file_path);
        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    fn test_validate_output_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path().join("new_output");

        let result = Validator::validate_output_directory(&output_path, false);
        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    fn test_validate_existing_nonempty_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(temp_dir.path().join("existing_file.txt"), "content")?;

        // Should fail without force
        let result = Validator::validate_output_directory(temp_dir.path(), false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not empty"));

        // Should succeed with force
        let result = Validator::validate_output_directory(temp_dir.path(), true);
        assert!(result.is_ok());

        Ok(())
    }
}

//! Project structure generation

use crate::CodegenError;
use max2sc_max_types::MaxPatch;
use max2sc_sc_types::SCProject;
use std::path::Path;

pub fn generate_project(patch: &MaxPatch, output_dir: &Path) -> Result<SCProject, CodegenError> {
    todo!("Implement project generation")
}

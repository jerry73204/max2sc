//! SuperCollider integration testing framework for max2sc
//!
//! This crate provides comprehensive testing capabilities for validating
//! Max to SuperCollider conversions, including:
//! - Syntax validation (compilation tests)
//! - Functional validation (runtime tests)
//! - Audio validation (output comparison)
//! - Integration testing (end-to-end workflow)

pub mod assertions;
pub mod audio;
pub mod comparison;
pub mod error;
pub mod fixtures;
pub mod functional;
pub mod phase4_tests;
pub mod runner;
pub mod spatial_functional_tests;
pub mod spatial_tests;
pub mod syntax;

pub use assertions::{Assertion, AssertionResult};
pub use audio::{AudioAnalysis, AudioComparison, AudioComparisonResult, AudioReference, AudioTest};
pub use comparison::{ComparisonConfig, ComparisonResult, Max2SCComparison, MaxRunner};
pub use error::{Result, TestError};
pub use fixtures::{TestData, TestFixture};
pub use functional::{FunctionalOutput, FunctionalTest};
pub use phase4_tests::{Phase3ValidationResults, Phase4Results, Phase4TestSuite};
pub use runner::{SCTestRunner, TestCategory, TestResult};
pub use spatial_functional_tests::{SpatialFunctionalResults, SpatialFunctionalTestSuite};
pub use spatial_tests::{SpatialTestResults, SpatialTestSuite};
pub use syntax::{CompileOutput, SyntaxTest};

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::fixtures::TestData;
    pub use crate::syntax::CompileExpectation;
    pub use crate::{
        Assertion, AudioTest, FunctionalTest, Result, SCTestRunner, SyntaxTest, TestCategory,
        TestError, TestFixture, TestResult,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_imports() {
        // Ensure all modules compile
        let _ = runner::SCTestRunner::new();
    }
}

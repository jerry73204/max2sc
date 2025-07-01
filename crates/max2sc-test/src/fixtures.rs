//! Test fixtures and data generation for max2sc testing

use crate::error::{Result, TestError};
use max2sc_max_types::{BoxContainer, BoxContent, LineContainer, MaxPatch, PatchLine, Patcher};
use serde_json::json;
use std::collections::HashMap;
use std::path::Path;
use tempfile::NamedTempFile;

/// Test fixture for managing test data
#[derive(Debug)]
pub struct TestFixture {
    /// Name of the fixture
    pub name: String,
    /// Max patch data
    pub patch: MaxPatch,
    /// Expected SuperCollider output
    pub expected_sc: Option<String>,
    /// Test metadata
    pub metadata: TestMetadata,
}

/// Test metadata
#[derive(Debug, Clone)]
pub struct TestMetadata {
    /// Description of what this test validates
    pub description: String,
    /// Test category
    pub category: TestCategory,
    /// Expected difficulty/complexity
    pub complexity: TestComplexity,
    /// Expected conversion time
    pub expected_duration: std::time::Duration,
    /// Required SuperCollider plugins
    pub required_plugins: Vec<String>,
}

/// Test categories
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TestCategory {
    /// Basic audio objects
    BasicAudio,
    /// Multichannel routing
    Multichannel,
    /// Spatial audio processing
    Spatial,
    /// Complex signal chains
    SignalChain,
    /// OSC and communication
    Communication,
    /// Edge cases and error conditions
    EdgeCase,
}

/// Test complexity levels
#[derive(Debug, Clone, PartialEq)]
pub enum TestComplexity {
    /// Simple, single-object tests
    Simple,
    /// Multiple objects with basic routing
    Medium,
    /// Complex patches with advanced features
    Complex,
    /// Stress tests with many objects
    Stress,
}

/// Test data generator
pub struct TestData;

impl TestFixture {
    /// Create a new test fixture
    pub fn new(name: impl Into<String>, patch: MaxPatch) -> Self {
        Self {
            name: name.into(),
            patch,
            expected_sc: None,
            metadata: TestMetadata::default(),
        }
    }

    /// Set expected SuperCollider output
    pub fn with_expected_sc(mut self, sc_code: impl Into<String>) -> Self {
        self.expected_sc = Some(sc_code.into());
        self
    }

    /// Set test metadata
    pub fn with_metadata(mut self, metadata: TestMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Save fixture to a temporary file
    pub async fn save_to_temp(&self) -> Result<NamedTempFile> {
        let patch_json = serde_json::to_string_pretty(&self.patch).map_err(TestError::Json)?;

        let mut temp_file = NamedTempFile::new()
            .map_err(|e| TestError::other(format!("Failed to create temp file: {e}")))?;

        use std::io::Write;
        temp_file
            .write_all(patch_json.as_bytes())
            .map_err(|e| TestError::other(format!("Failed to write temp file: {e}")))?;

        Ok(temp_file)
    }

    /// Load fixture from JSON file
    pub async fn load_from_file(path: &Path) -> Result<Self> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| TestError::other(format!("Failed to read file: {e}")))?;

        let patch: MaxPatch = serde_json::from_str(&content).map_err(TestError::Json)?;

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(Self::new(name, patch))
    }
}

impl TestMetadata {
    /// Create metadata for a basic audio test
    pub fn basic_audio(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            category: TestCategory::BasicAudio,
            complexity: TestComplexity::Simple,
            expected_duration: std::time::Duration::from_millis(100),
            required_plugins: vec![],
        }
    }

    /// Create metadata for a spatial audio test
    pub fn spatial_audio(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            category: TestCategory::Spatial,
            complexity: TestComplexity::Medium,
            expected_duration: std::time::Duration::from_millis(500),
            required_plugins: vec!["sc3-plugins".to_string(), "atk-sc3".to_string()],
        }
    }

    /// Create metadata for a multichannel test
    pub fn multichannel(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            category: TestCategory::Multichannel,
            complexity: TestComplexity::Medium,
            expected_duration: std::time::Duration::from_millis(200),
            required_plugins: vec![],
        }
    }
}

impl Default for TestMetadata {
    fn default() -> Self {
        Self {
            description: "Generic test".to_string(),
            category: TestCategory::BasicAudio,
            complexity: TestComplexity::Simple,
            expected_duration: std::time::Duration::from_millis(100),
            required_plugins: vec![],
        }
    }
}

impl TestData {
    /// Create a default empty patcher structure
    fn default_patcher() -> Patcher {
        Patcher {
            fileversion: 1,
            appversion: None,
            classnamespace: Some("box".to_string()),
            rect: [100.0, 100.0, 600.0, 400.0],
            bglocked: 0,
            openinpresentation: 0,
            default_fontsize: 12.0,
            default_fontface: 0,
            default_fontname: None,
            gridonopen: 1,
            gridsize: Some([15.0, 15.0]),
            gridsnaponopen: 1,
            objectsnaponopen: 1,
            statusbarvisible: 2,
            toolbarvisible: 1,
            boxes: vec![],
            lines: vec![],
            extra_fields: HashMap::new(),
        }
    }

    /// Create a box container with the given content
    fn create_box(
        id: &str,
        maxclass: &str,
        text: Option<String>,
        numinlets: u32,
        numoutlets: u32,
        rect: [f32; 4],
        outlettype: Option<Vec<String>>,
    ) -> BoxContainer {
        BoxContainer {
            content: BoxContent {
                id: id.to_string(),
                maxclass: maxclass.to_string(),
                text,
                numinlets,
                numoutlets,
                patching_rect: Some(rect),
                outlettype,
                parameter_enable: None,
                attributes: HashMap::new(),
            },
        }
    }

    /// Create a patch line connection
    fn create_line(
        source_id: &str,
        source_outlet: u32,
        dest_id: &str,
        dest_inlet: u32,
    ) -> LineContainer {
        LineContainer {
            patchline: PatchLine {
                source: json!([source_id, source_outlet]),
                destination: json!([dest_id, dest_inlet]),
                midpoints: None,
                extra_fields: HashMap::new(),
            },
        }
    }
    /// Generate a simple sine wave patch
    pub fn simple_sine(frequency: f32) -> TestFixture {
        let mut patcher = Self::default_patcher();

        patcher.boxes = vec![
            Self::create_box(
                "obj-1",
                "newobj",
                Some(format!("cycle~ {frequency}")),
                2,
                1,
                [50.0, 50.0, 100.0, 20.0],
                Some(vec!["signal".to_string()]),
            ),
            Self::create_box(
                "obj-2",
                "newobj",
                Some("dac~".to_string()),
                2,
                0,
                [50.0, 100.0, 50.0, 20.0],
                Some(vec![]),
            ),
        ];

        patcher.lines = vec![
            Self::create_line("obj-1", 0, "obj-2", 0),
            Self::create_line("obj-1", 0, "obj-2", 1),
        ];

        let patch = MaxPatch { patcher };

        TestFixture::new(format!("simple_sine_{frequency}"), patch)
            .with_metadata(TestMetadata::basic_audio(format!(
                "Simple sine wave at {frequency} Hz"
            )))
            .with_expected_sc(format!(
                "SynthDef(\\simpleSine, {{ Out.ar(0, SinOsc.ar({frequency}, 0, 0.5).dup) }}).play;"
            ))
    }

    /// Generate a multichannel test patch
    pub fn multichannel_router(channels: u32) -> TestFixture {
        // Simplified version for now - would be expanded in full implementation
        let mut patcher = Self::default_patcher();
        patcher.boxes = vec![
            Self::create_box(
                "obj-1",
                "newobj",
                Some("noise~".to_string()),
                0,
                1,
                [50.0, 50.0, 50.0, 20.0],
                Some(vec!["signal".to_string()]),
            ),
            Self::create_box(
                "obj-2",
                "newobj",
                Some(format!("mc.dac~ 1-{channels}")),
                1,
                0,
                [50.0, 100.0, 100.0, 20.0],
                Some(vec![]),
            ),
        ];
        patcher.lines = vec![Self::create_line("obj-1", 0, "obj-2", 0)];
        let patch = MaxPatch { patcher };

        TestFixture::new(format!("multichannel_router_{channels}"), patch).with_metadata(
            TestMetadata::multichannel(format!("{channels}-channel noise router")),
        )
    }

    /// Generate a simplified SPAT5 panoramix test patch  
    pub fn spat5_panoramix(_inputs: u32, _outputs: u32) -> TestFixture {
        // Simplified version for now
        let mut patcher = Self::default_patcher();
        patcher.boxes = vec![
            Self::create_box(
                "obj-1",
                "newobj",
                Some("noise~".to_string()),
                0,
                1,
                [50.0, 50.0, 50.0, 20.0],
                Some(vec!["signal".to_string()]),
            ),
            Self::create_box(
                "obj-2",
                "newobj",
                Some("spat5.panoramix~".to_string()),
                1,
                8,
                [50.0, 100.0, 150.0, 20.0],
                Some(vec!["multichannelsignal".to_string()]),
            ),
        ];
        let patch = MaxPatch { patcher };

        TestFixture::new("spat5_panoramix_test".to_string(), patch)
            .with_metadata(TestMetadata::spatial_audio("SPAT5 panoramix test"))
    }

    /// Generate a simplified VBAP test patch
    pub fn vbap_panner(_speakers: u32) -> TestFixture {
        // Simplified version for now
        let mut patcher = Self::default_patcher();
        patcher.boxes = vec![
            Self::create_box(
                "obj-1",
                "newobj",
                Some("cycle~ 440".to_string()),
                2,
                1,
                [50.0, 50.0, 80.0, 20.0],
                Some(vec!["signal".to_string()]),
            ),
            Self::create_box(
                "obj-2",
                "newobj",
                Some("spat5.vbap~".to_string()),
                1,
                8,
                [50.0, 100.0, 100.0, 20.0],
                Some(vec!["multichannelsignal".to_string()]),
            ),
        ];
        let patch = MaxPatch { patcher };

        TestFixture::new("vbap_panner_test".to_string(), patch)
            .with_metadata(TestMetadata::spatial_audio("VBAP panner test"))
    }

    /// Generate a simplified HOA test patch
    pub fn hoa_chain(_order: u32, _speakers: u32) -> TestFixture {
        // Simplified version for now
        let mut patcher = Self::default_patcher();
        patcher.boxes = vec![
            Self::create_box(
                "obj-1",
                "newobj",
                Some("cycle~ 440".to_string()),
                2,
                1,
                [50.0, 50.0, 80.0, 20.0],
                Some(vec!["signal".to_string()]),
            ),
            Self::create_box(
                "obj-2",
                "newobj",
                Some("spat5.hoa.encoder~".to_string()),
                1,
                4,
                [50.0, 100.0, 150.0, 20.0],
                Some(vec!["multichannelsignal".to_string()]),
            ),
        ];
        let patch = MaxPatch { patcher };

        TestFixture::new("hoa_chain_test".to_string(), patch)
            .with_metadata(TestMetadata::spatial_audio("HOA chain test"))
    }

    /// Get all standard test fixtures
    pub fn standard_fixtures() -> Vec<TestFixture> {
        vec![
            Self::simple_sine(440.0),
            Self::simple_sine(880.0),
            Self::multichannel_router(4),
            Self::multichannel_router(8),
            Self::spat5_panoramix(1, 8),
            Self::vbap_panner(8),
            Self::hoa_chain(1, 8),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_sine_fixture() {
        let fixture = TestData::simple_sine(440.0);
        assert_eq!(fixture.name, "simple_sine_440");
        assert_eq!(fixture.patch.patcher.boxes.len(), 2);
        assert!(fixture.expected_sc.is_some());
    }

    #[test]
    fn test_multichannel_fixture() {
        let fixture = TestData::multichannel_router(4);
        assert_eq!(fixture.name, "multichannel_router_4");
        assert_eq!(fixture.patch.patcher.boxes.len(), 2); // Simplified version has 2 boxes
        assert_eq!(fixture.metadata.category, TestCategory::Multichannel);
    }

    #[test]
    fn test_spatial_fixtures() {
        let panoramix = TestData::spat5_panoramix(1, 8);
        assert_eq!(panoramix.metadata.category, TestCategory::Spatial);
        assert!(panoramix
            .metadata
            .required_plugins
            .contains(&"atk-sc3".to_string()));

        let vbap = TestData::vbap_panner(8);
        assert_eq!(vbap.metadata.category, TestCategory::Spatial);

        let hoa = TestData::hoa_chain(1, 8);
        assert_eq!(hoa.metadata.category, TestCategory::Spatial);
    }

    #[test]
    fn test_standard_fixtures() {
        let fixtures = TestData::standard_fixtures();
        assert_eq!(fixtures.len(), 7); // Updated count for simplified fixtures

        // Check variety of categories
        let categories: std::collections::HashSet<_> =
            fixtures.iter().map(|f| &f.metadata.category).collect();
        assert!(categories.contains(&TestCategory::BasicAudio));
        assert!(categories.contains(&TestCategory::Multichannel));
        assert!(categories.contains(&TestCategory::Spatial));
    }
}

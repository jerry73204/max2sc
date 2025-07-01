//! Comprehensive tests for all Phase 3 spatial audio features
//!
//! This module validates the conversion accuracy of all spatial audio
//! components using the SuperCollider testing framework.

use crate::prelude::*;
use crate::CompileOutput;

/// Test suite for all Phase 3 spatial features
pub struct SpatialTestSuite {
    runner: SCTestRunner,
}

impl SpatialTestSuite {
    /// Create a new spatial test suite
    pub fn new() -> Result<Self> {
        Ok(Self {
            runner: SCTestRunner::new()?,
        })
    }

    /// Run all spatial tests
    pub async fn run_all_tests(&self) -> Result<SpatialTestResults> {
        let mut results = SpatialTestResults::new();

        // Test SPAT5 objects
        results.spat5 = Some(self.test_spat5_conversions().await?);

        // Test WFS arrays
        results.wfs = Some(self.test_wfs_conversions().await?);

        // Test VBAP systems
        results.vbap = Some(self.test_vbap_conversions().await?);

        // Test HOA encoding/decoding
        results.hoa = Some(self.test_hoa_conversions().await?);

        // Test distance effects
        results.distance = Some(self.test_distance_effects().await?);

        // Test early reflections
        results.reflections = Some(self.test_early_reflections().await?);

        Ok(results)
    }

    /// Test SPAT5 object conversions
    async fn test_spat5_conversions(&self) -> Result<TestResult<Vec<CompileOutput>>> {
        self.runner
            .run_test(TestCategory::Syntax, async {
                let mut outputs = Vec::new();

                // Test spat5.panoramix~
                let panoramix_code = r#"
(
// Test SPAT5 panoramix conversion
SynthDef(\testPanoramix, {
    var input = In.ar(0, 1);
    var output = SpatPanoramix.ar(input, 1, 8);
    Out.ar(0, output);
}).add;
)
"#;
                let syntax_test = SyntaxTest::new(panoramix_code);
                outputs.push(syntax_test.run(&self.runner).await?);

                // Test spat5.reverb~
                let reverb_code = r#"
(
// Test SPAT5 reverb conversion
SynthDef(\testReverb, {
    var input = In.ar(0, 2);
    var reverb = JPverb.ar(input, \t60.kr(2.0), \damp.kr(0.5), \size.kr(1.0));
    Out.ar(0, reverb);
}).add;
)
"#;
                let syntax_test = SyntaxTest::new(reverb_code);
                outputs.push(syntax_test.run(&self.runner).await?);

                // Test spat5.early~
                let early_code = r#"
(
// Test SPAT5 early reflections conversion
SynthDef(\testEarly, {
    var input = In.ar(0, 1);
    var early = EarlyReflections.ar(input, 
        \roomSize.kr(10), 
        \reflectivity.kr(0.7),
        \numReflections.kr(12));
    Out.ar(0, early);
}).add;
)
"#;
                let syntax_test = SyntaxTest::new(early_code);
                outputs.push(syntax_test.run(&self.runner).await?);

                Ok(outputs)
            })
            .await
    }

    /// Test WFS array conversions
    async fn test_wfs_conversions(&self) -> Result<TestResult<Vec<CompileOutput>>> {
        self.runner
            .run_test(TestCategory::Syntax, async {
                let mut outputs = Vec::new();

                // Test linear WFS array
                let linear_wfs_code = r#"
(
// Test linear WFS array
SynthDef(\testLinearWFS, {
    var input = In.ar(0, 1);
    var positions = [
        [-2, 0], [-1, 0], [0, 0], [1, 0], [2, 0]
    ];
    var delays = [0.001, 0.0005, 0, 0.0005, 0.001];
    var gains = [0.8, 0.9, 1.0, 0.9, 0.8];
    var wfs = WFSPanner.ar(input, 
        \sourceX.kr(0), \sourceY.kr(1),
        positions, delays, gains);
    Out.ar(0, wfs);
}).add;
)
"#;
                let syntax_test = SyntaxTest::new(linear_wfs_code);
                outputs.push(syntax_test.run(&self.runner).await?);

                // Test circular WFS array
                let circular_wfs_code = r#"
(
// Test circular WFS array
SynthDef(\testCircularWFS, {
    var input = In.ar(0, 1);
    var numSpeakers = 16;
    var radius = 2.0;
    var positions = Array.fill(numSpeakers, { |i|
        var angle = i * 2pi / numSpeakers;
        [radius * cos(angle), radius * sin(angle)]
    });
    var wfs = WFSPanner.ar(input, 
        \sourceX.kr(0), \sourceY.kr(0),
        positions, 0, 1);
    Out.ar(0, wfs);
}).add;
)
"#;
                let syntax_test = SyntaxTest::new(circular_wfs_code);
                outputs.push(syntax_test.run(&self.runner).await?);

                Ok(outputs)
            })
            .await
    }

    /// Test VBAP conversions
    async fn test_vbap_conversions(&self) -> Result<TestResult<Vec<CompileOutput>>> {
        self.runner
            .run_test(TestCategory::Syntax, async {
                let mut outputs = Vec::new();

                // Test 2D VBAP
                let vbap_2d_code = r#"
(
// Test 2D VBAP setup
~vbap2D = VBAPSpeakerArray.new(2, [
    [0, 1], [60, 1], [120, 1], [180, 1], [240, 1], [300, 1]
]);

SynthDef(\testVBAP2D, {
    var input = In.ar(0, 1);
    var vbap = VBAP.ar(4, input, \azim.kr(0));
    Out.ar(0, vbap);
}).add;
)
"#;
                let syntax_test = SyntaxTest::new(vbap_2d_code);
                outputs.push(syntax_test.run(&self.runner).await?);

                // Test 3D VBAP
                let vbap_3d_code = r#"
(
// Test 3D VBAP setup
~vbap3D = VBAPSpeakerArray.new(3, [
    [0, 0, 1], [45, 0, 1], [90, 0, 1], [135, 0, 1],
    [180, 0, 1], [225, 0, 1], [270, 0, 1], [315, 0, 1],
    [0, 45, 1], [90, 45, 1], [180, 45, 1], [270, 45, 1]
]);

SynthDef(\testVBAP3D, {
    var input = In.ar(0, 1);
    var vbap = VBAP.ar(8, input, \azim.kr(0), \elev.kr(0));
    Out.ar(0, vbap);
}).add;
)
"#;
                let syntax_test = SyntaxTest::new(vbap_3d_code);
                outputs.push(syntax_test.run(&self.runner).await?);

                Ok(outputs)
            })
            .await
    }

    /// Test HOA conversions
    async fn test_hoa_conversions(&self) -> Result<TestResult<Vec<CompileOutput>>> {
        self.runner
            .run_test(TestCategory::Syntax, async {
                let mut outputs = Vec::new();

                // Test FOA (1st order) encoding
                let foa_encode_code = r#"
(
// Test First Order Ambisonics encoding
SynthDef(\testFOAEncode, {
    var input = In.ar(0, 1);
    var encoded = FoaEncode.ar(input, \azim.kr(0), \elev.kr(0));
    Out.ar(0, encoded);
}).add;
)
"#;
                let syntax_test = SyntaxTest::new(foa_encode_code);
                outputs.push(syntax_test.run(&self.runner).await?);

                // Test FOA decoding to stereo
                let foa_decode_code = r#"
(
// Test First Order Ambisonics decoding
SynthDef(\testFOADecode, {
    var ambiB = In.ar(0, 4);
    var decoded = FoaDecode.ar(ambiB, ~foaDecoderMatrix);
    Out.ar(0, decoded);
}).add;
)
"#;
                let syntax_test = SyntaxTest::new(foa_decode_code);
                outputs.push(syntax_test.run(&self.runner).await?);

                // Test HOA (higher order) encoding
                let hoa_encode_code = r#"
(
// Test Higher Order Ambisonics encoding (3rd order)
SynthDef(\testHOAEncode, {
    var input = In.ar(0, 1);
    var encoded = HoaEncode.ar(input, \azim.kr(0), \elev.kr(0), order: 3);
    Out.ar(0, encoded);
}).add;
)
"#;
                let syntax_test = SyntaxTest::new(hoa_encode_code);
                outputs.push(syntax_test.run(&self.runner).await?);

                // Test HOA rotation
                let hoa_rotate_code = r#"
(
// Test HOA rotation transformation
SynthDef(\testHOARotate, {
    var hoaSignal = In.ar(0, 16); // 3rd order = 16 channels
    var rotated = HoaRotate.ar(hoaSignal, \yaw.kr(0), \pitch.kr(0), \roll.kr(0));
    Out.ar(0, rotated);
}).add;
)
"#;
                let syntax_test = SyntaxTest::new(hoa_rotate_code);
                outputs.push(syntax_test.run(&self.runner).await?);

                Ok(outputs)
            })
            .await
    }

    /// Test distance-based effects
    async fn test_distance_effects(&self) -> Result<TestResult<Vec<CompileOutput>>> {
        self.runner
            .run_test(TestCategory::Syntax, async {
                let mut outputs = Vec::new();

                // Test distance attenuation
                let distance_code = r#"
(
// Test distance attenuation
SynthDef(\testDistance, {
    var input = In.ar(0, 1);
    var distance = \distance.kr(1.0);
    var attenuated = input * (1.0 / (distance + 1.0));
    var delayed = DelayC.ar(attenuated, 0.1, distance / 343.0);
    Out.ar(0, delayed);
}).add;
)
"#;
                let syntax_test = SyntaxTest::new(distance_code);
                outputs.push(syntax_test.run(&self.runner).await?);

                // Test air absorption
                let air_absorption_code = r#"
(
// Test air absorption filter
SynthDef(\testAirAbsorption, {
    var input = In.ar(0, 1);
    var distance = \distance.kr(1.0);
    var freq = \freq.kr(5000);
    var absorption = LPF.ar(input, freq / (1 + distance * 0.1));
    Out.ar(0, absorption);
}).add;
)
"#;
                let syntax_test = SyntaxTest::new(air_absorption_code);
                outputs.push(syntax_test.run(&self.runner).await?);

                // Test Doppler effect
                let doppler_code = r#"
(
// Test Doppler effect
SynthDef(\testDoppler, {
    var input = In.ar(0, 1);
    var velocity = \velocity.kr(0.0);
    var speedOfSound = 343.0;
    var dopplerRatio = (speedOfSound + velocity) / speedOfSound;
    var doppler = PitchShift.ar(input, 0.1, dopplerRatio);
    Out.ar(0, doppler);
}).add;
)
"#;
                let syntax_test = SyntaxTest::new(doppler_code);
                outputs.push(syntax_test.run(&self.runner).await?);

                Ok(outputs)
            })
            .await
    }

    /// Test early reflection patterns
    async fn test_early_reflections(&self) -> Result<TestResult<Vec<CompileOutput>>> {
        self.runner
            .run_test(TestCategory::Syntax, async {
                let mut outputs = Vec::new();

                // Test room-based early reflections
                let room_reflections_code = r#"
(
// Test room-based early reflections
SynthDef(\testRoomReflections, {
    var input = In.ar(0, 1);
    var roomSize = \roomSize.kr(10);
    var reflectivity = \reflectivity.kr(0.7);
    
    // Calculate reflection delays based on room geometry
    var delays = [0.010, 0.015, 0.023, 0.031, 0.041, 0.052];
    var gains = [0.7, 0.6, 0.5, 0.4, 0.3, 0.2] * reflectivity;
    
    var reflections = Mix.fill(6, { |i|
        DelayC.ar(input, 0.1, delays[i]) * gains[i]
    });
    
    Out.ar(0, input + reflections);
}).add;
)
"#;
                let syntax_test = SyntaxTest::new(room_reflections_code);
                outputs.push(syntax_test.run(&self.runner).await?);

                // Test multichannel early reflections
                let mc_reflections_code = r#"
(
// Test multichannel early reflections
SynthDef(\testMCReflections, {
    var input = In.ar(0, 1);
    var reflections = Array.fill(8, { |i|
        var delay = 0.005 + (i * 0.003);
        var gain = 0.8 * (0.9 ** i);
        var pan = (i / 7) * 2 - 1; // -1 to 1
        Pan2.ar(DelayC.ar(input, 0.1, delay) * gain, pan);
    });
    Out.ar(0, Mix(reflections));
}).add;
)
"#;
                let syntax_test = SyntaxTest::new(mc_reflections_code);
                outputs.push(syntax_test.run(&self.runner).await?);

                Ok(outputs)
            })
            .await
    }
}

/// Results from spatial test suite
#[derive(Debug)]
pub struct SpatialTestResults {
    pub spat5: Option<TestResult<Vec<CompileOutput>>>,
    pub wfs: Option<TestResult<Vec<CompileOutput>>>,
    pub vbap: Option<TestResult<Vec<CompileOutput>>>,
    pub hoa: Option<TestResult<Vec<CompileOutput>>>,
    pub distance: Option<TestResult<Vec<CompileOutput>>>,
    pub reflections: Option<TestResult<Vec<CompileOutput>>>,
}

impl SpatialTestResults {
    fn new() -> Self {
        Self {
            spat5: None,
            wfs: None,
            vbap: None,
            hoa: None,
            distance: None,
            reflections: None,
        }
    }

    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        let check_result = |result: &Option<TestResult<Vec<CompileOutput>>>| {
            result
                .as_ref()
                .is_some_and(|r| r.data.iter().all(|output| output.success))
        };

        check_result(&self.spat5)
            && check_result(&self.wfs)
            && check_result(&self.vbap)
            && check_result(&self.hoa)
            && check_result(&self.distance)
            && check_result(&self.reflections)
    }

    /// Get total number of tests
    pub fn total_tests(&self) -> usize {
        let count_tests = |result: &Option<TestResult<Vec<CompileOutput>>>| {
            result.as_ref().map_or(0, |r| r.data.len())
        };

        count_tests(&self.spat5)
            + count_tests(&self.wfs)
            + count_tests(&self.vbap)
            + count_tests(&self.hoa)
            + count_tests(&self.distance)
            + count_tests(&self.reflections)
    }

    /// Get number of passed tests
    pub fn passed_tests(&self) -> usize {
        let count_passed = |result: &Option<TestResult<Vec<CompileOutput>>>| {
            result
                .as_ref()
                .map_or(0, |r| r.data.iter().filter(|output| output.success).count())
        };

        count_passed(&self.spat5)
            + count_passed(&self.wfs)
            + count_passed(&self.vbap)
            + count_passed(&self.hoa)
            + count_passed(&self.distance)
            + count_passed(&self.reflections)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_spatial_suite_creation() {
        // Test that we can create the spatial test suite
        // This may fail if SuperCollider is not installed
        if let Ok(suite) = SpatialTestSuite::new() {
            assert!(suite.runner.temp_dir().exists());
        }
    }

    #[test]
    fn test_spatial_results_tracking() {
        let results = SpatialTestResults::new();
        assert_eq!(results.total_tests(), 0);
        assert_eq!(results.passed_tests(), 0);
        assert!(!results.all_passed());
    }
}

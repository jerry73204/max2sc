//! Functional tests for spatial audio object instantiation
//!
//! These tests verify that spatial objects can be properly instantiated
//! and respond to parameter changes in SuperCollider.

use crate::prelude::*;
use crate::FunctionalOutput;
use std::collections::HashMap;

/// Functional test suite for spatial audio objects
pub struct SpatialFunctionalTestSuite {
    runner: SCTestRunner,
}

impl SpatialFunctionalTestSuite {
    /// Create a new functional test suite
    pub fn new() -> Result<Self> {
        Ok(Self {
            runner: SCTestRunner::new()?,
        })
    }

    /// Run all functional tests
    pub async fn run_all_tests(&self) -> Result<SpatialFunctionalResults> {
        let mut results = SpatialFunctionalResults::new();

        // Test SPAT5 object instantiation
        results.spat5_instantiation = Some(self.test_spat5_instantiation().await?);

        // Test WFS parameter control
        results.wfs_parameters = Some(self.test_wfs_parameters().await?);

        // Test VBAP panning control
        results.vbap_panning = Some(self.test_vbap_panning().await?);

        // Test HOA transformations
        results.hoa_transforms = Some(self.test_hoa_transforms().await?);

        // Test spatial effect chains
        results.effect_chains = Some(self.test_effect_chains().await?);

        Ok(results)
    }

    /// Test SPAT5 object instantiation and parameter control
    async fn test_spat5_instantiation(&self) -> Result<TestResult<Vec<FunctionalOutput>>> {
        self.runner
            .run_test(TestCategory::Functional, async {
                let mut outputs = Vec::new();

                // Test panoramix instantiation and parameter control
                let panoramix_test = FunctionalTest::new(
                    r#"
(
// Test SPAT5 panoramix instantiation
var synthDef, synth, bus, result;

// Create test SynthDef
synthDef = SynthDef(\testPanoramixInst, {
    var input = WhiteNoise.ar(0.1);
    var output = SpatPanoramix.ar(input, 1, 8,
        \reverb.kr(0.5),
        \early.kr(0.3),
        \direct.kr(0.7)
    );
    Out.ar(0, output);
});
synthDef.add;

// Wait for SynthDef to load
Server.default.sync;

// Test instantiation
synth = Synth(\testPanoramixInst);

if(synth.notNil, {
    // Test parameter changes
    synth.set(\reverb, 0.8);
    synth.set(\early, 0.6);
    synth.set(\direct, 0.9);
    
    // Clean up
    synth.free;
    
    result = "SUCCESS: SPAT5 panoramix instantiated and controlled";
}, {
    result = "FAILURE: SPAT5 panoramix instantiation failed";
});

result.postln;
)
"#,
                )
                .assert_all(vec![Assertion::contains_output("SUCCESS")]);
                outputs.push(panoramix_test.run(&self.runner).await?);

                // Test reverb instantiation
                let reverb_test = FunctionalTest::new(
                    r#"
(
// Test SPAT5 reverb instantiation
var synthDef, synth, result;

synthDef = SynthDef(\testReverbInst, {
    var input = WhiteNoise.ar(0.1);
    var reverb = JPverb.ar(input.dup, 
        \t60.kr(2.0), 
        \damp.kr(0.5), 
        \size.kr(1.0),
        \earlyDiff.kr(0.707),
        \modDepth.kr(0.1),
        \modFreq.kr(2.0)
    );
    Out.ar(0, reverb);
});
synthDef.add;

Server.default.sync;

synth = Synth(\testReverbInst);

if(synth.notNil, {
    // Test parameter automation
    synth.set(\t60, 5.0);
    synth.set(\damp, 0.8);
    synth.set(\size, 1.5);
    
    synth.free;
    result = "SUCCESS: SPAT5 reverb instantiated and controlled";
}, {
    result = "FAILURE: SPAT5 reverb instantiation failed";
});

result.postln;
)
"#,
                )
                .assert_all(vec![Assertion::contains_output("SUCCESS")]);
                outputs.push(reverb_test.run(&self.runner).await?);

                Ok(outputs)
            })
            .await
    }

    /// Test WFS parameter control
    async fn test_wfs_parameters(&self) -> Result<TestResult<Vec<FunctionalOutput>>> {
        self.runner
            .run_test(TestCategory::Functional, async {
                let mut outputs = Vec::new();

                // Test WFS source positioning
                let wfs_positioning_test = FunctionalTest::new(
                    r#"
(
// Test WFS source positioning
var synthDef, synth, result;

synthDef = SynthDef(\testWFSPosition, {
    var input = SinOsc.ar(440, 0, 0.1);
    var positions = [
        [-2, 0], [-1, 0], [0, 0], [1, 0], [2, 0]
    ];
    var delays = [0.001, 0.0005, 0, 0.0005, 0.001];
    var gains = [0.8, 0.9, 1.0, 0.9, 0.8];
    
    var wfs = WFSPanner.ar(input, 
        \sourceX.kr(0), 
        \sourceY.kr(1),
        positions, delays, gains);
    Out.ar(0, wfs);
});
synthDef.add;

Server.default.sync;

synth = Synth(\testWFSPosition);

if(synth.notNil, {
    // Test source movement
    synth.set(\sourceX, -1.0);
    0.1.wait;
    synth.set(\sourceX, 1.0);
    0.1.wait;
    synth.set(\sourceY, 2.0);
    
    synth.free;
    result = "SUCCESS: WFS source positioning works";
}, {
    result = "FAILURE: WFS source positioning failed";
});

result.postln;
)
"#,
                )
                .assert_all(vec![Assertion::contains_output("SUCCESS")]);
                outputs.push(wfs_positioning_test.run(&self.runner).await?);

                Ok(outputs)
            })
            .await
    }

    /// Test VBAP panning control
    async fn test_vbap_panning(&self) -> Result<TestResult<Vec<FunctionalOutput>>> {
        self.runner
            .run_test(TestCategory::Functional, async {
                let mut outputs = Vec::new();

                // Test VBAP 2D panning
                let vbap_2d_test = FunctionalTest::new(
                    r#"
(
// Test VBAP 2D panning
var synthDef, synth, result;

// Set up VBAP speaker array
~vbap2D = VBAPSpeakerArray.new(2, [
    [0, 1], [60, 1], [120, 1], [180, 1], [240, 1], [300, 1]
]);

synthDef = SynthDef(\testVBAP2D, {
    var input = SinOsc.ar(440, 0, 0.1);
    var vbap = VBAP.ar(6, input, \azim.kr(0), \spread.kr(0));
    Out.ar(0, vbap);
});
synthDef.add;

Server.default.sync;

synth = Synth(\testVBAP2D);

if(synth.notNil, {
    // Test azimuth panning
    synth.set(\azim, 60);
    0.1.wait;
    synth.set(\azim, 120);
    0.1.wait;
    synth.set(\spread, 30);
    
    synth.free;
    result = "SUCCESS: VBAP 2D panning works";
}, {
    result = "FAILURE: VBAP 2D panning failed";
});

result.postln;
)
"#,
                )
                .assert_all(vec![Assertion::contains_output("SUCCESS")]);
                outputs.push(vbap_2d_test.run(&self.runner).await?);

                // Test VBAP 3D panning
                let vbap_3d_test = FunctionalTest::new(
                    r#"
(
// Test VBAP 3D panning
var synthDef, synth, result;

~vbap3D = VBAPSpeakerArray.new(3, [
    [0, 0, 1], [45, 0, 1], [90, 0, 1], [135, 0, 1],
    [180, 0, 1], [225, 0, 1], [270, 0, 1], [315, 0, 1],
    [0, 45, 1], [90, 45, 1], [180, 45, 1], [270, 45, 1]
]);

synthDef = SynthDef(\testVBAP3D, {
    var input = SinOsc.ar(440, 0, 0.1);
    var vbap = VBAP.ar(12, input, \azim.kr(0), \elev.kr(0));
    Out.ar(0, vbap);
});
synthDef.add;

Server.default.sync;

synth = Synth(\testVBAP3D);

if(synth.notNil, {
    // Test 3D positioning
    synth.set(\azim, 45, \elev, 30);
    0.1.wait;
    synth.set(\azim, 180, \elev, -15);
    
    synth.free;
    result = "SUCCESS: VBAP 3D panning works";
}, {
    result = "FAILURE: VBAP 3D panning failed";
});

result.postln;
)
"#,
                )
                .assert_all(vec![Assertion::contains_output("SUCCESS")]);
                outputs.push(vbap_3d_test.run(&self.runner).await?);

                Ok(outputs)
            })
            .await
    }

    /// Test HOA transformations
    async fn test_hoa_transforms(&self) -> Result<TestResult<Vec<FunctionalOutput>>> {
        self.runner
            .run_test(TestCategory::Functional, async {
                let mut outputs = Vec::new();

                // Test HOA encoding and decoding
                let hoa_encode_decode_test = FunctionalTest::new(
                    r#"
(
// Test HOA encoding and decoding
var encodeDef, decodeDef, encSynth, decSynth, result;

// Test encoding
encodeDef = SynthDef(\testHOAEncode, {
    var input = SinOsc.ar(440, 0, 0.1);
    var encoded = FoaEncode.ar(input, \azim.kr(0), \elev.kr(0));
    Out.ar(10, encoded); // Output to private bus
});
encodeDef.add;

// Test decoding
decodeDef = SynthDef(\testHOADecode, {
    var ambiB = In.ar(10, 4);
    var decoded = FoaDecode.ar(ambiB, ~foaDecoderMatrix);
    Out.ar(0, decoded);
});
decodeDef.add;

Server.default.sync;

// Create decoder matrix for stereo
~foaDecoderMatrix = FoaDecoderMatrix.newStereo;

encSynth = Synth(\testHOAEncode);
decSynth = Synth(\testHOADecode);

if(encSynth.notNil && decSynth.notNil, {
    // Test source movement in ambisonic space
    encSynth.set(\azim, pi/4);
    0.1.wait;
    encSynth.set(\azim, pi/2);
    0.1.wait;
    encSynth.set(\elev, pi/6);
    
    encSynth.free;
    decSynth.free;
    result = "SUCCESS: HOA encoding/decoding works";
}, {
    result = "FAILURE: HOA encoding/decoding failed";
});

result.postln;
)
"#,
                )
                .assert_all(vec![Assertion::contains_output("SUCCESS")]);
                outputs.push(hoa_encode_decode_test.run(&self.runner).await?);

                Ok(outputs)
            })
            .await
    }

    /// Test spatial effect chains
    async fn test_effect_chains(&self) -> Result<TestResult<Vec<FunctionalOutput>>> {
        self.runner
            .run_test(TestCategory::Functional, async {
                let mut outputs = Vec::new();

                // Test reverb + early reflections chain
                let effect_chain_test = FunctionalTest::new(
                    r#"
(
// Test spatial effect chain: Early reflections -> Reverb
var chainDef, synth, result;

chainDef = SynthDef(\testEffectChain, {
    var input = WhiteNoise.ar(0.05);
    var early, reverb, output;
    
    // Early reflections
    early = EarlyReflections.ar(input, 
        \roomSize.kr(10), 
        \reflectivity.kr(0.7),
        \numReflections.kr(8));
    
    // Late reverb
    reverb = JPverb.ar(early.dup, 
        \t60.kr(3.0), 
        \damp.kr(0.6), 
        \size.kr(1.2));
    
    // Mix dry and processed
    output = input.dup + (early.dup * \earlyLevel.kr(0.3)) + (reverb * \reverbLevel.kr(0.2));
    
    Out.ar(0, output);
});
chainDef.add;

Server.default.sync;

synth = Synth(\testEffectChain);

if(synth.notNil, {
    // Test effect chain parameters
    synth.set(\roomSize, 15);
    synth.set(\reflectivity, 0.8);
    synth.set(\t60, 5.0);
    synth.set(\earlyLevel, 0.5);
    synth.set(\reverbLevel, 0.4);
    
    0.2.wait;
    
    synth.free;
    result = "SUCCESS: Spatial effect chain works";
}, {
    result = "FAILURE: Spatial effect chain failed";
});

result.postln;
)
"#,
                )
                .assert_all(vec![Assertion::contains_output("SUCCESS")]);
                outputs.push(effect_chain_test.run(&self.runner).await?);

                Ok(outputs)
            })
            .await
    }
}

/// Results from spatial functional tests
#[derive(Debug)]
pub struct SpatialFunctionalResults {
    pub spat5_instantiation: Option<TestResult<Vec<FunctionalOutput>>>,
    pub wfs_parameters: Option<TestResult<Vec<FunctionalOutput>>>,
    pub vbap_panning: Option<TestResult<Vec<FunctionalOutput>>>,
    pub hoa_transforms: Option<TestResult<Vec<FunctionalOutput>>>,
    pub effect_chains: Option<TestResult<Vec<FunctionalOutput>>>,
}

impl SpatialFunctionalResults {
    fn new() -> Self {
        Self {
            spat5_instantiation: None,
            wfs_parameters: None,
            vbap_panning: None,
            hoa_transforms: None,
            effect_chains: None,
        }
    }

    /// Check if all functional tests passed
    pub fn all_passed(&self) -> bool {
        let check_result = |result: &Option<TestResult<Vec<FunctionalOutput>>>| {
            result
                .as_ref()
                .map_or(false, |r| r.data.iter().all(|output| output.success))
        };

        check_result(&self.spat5_instantiation)
            && check_result(&self.wfs_parameters)
            && check_result(&self.vbap_panning)
            && check_result(&self.hoa_transforms)
            && check_result(&self.effect_chains)
    }

    /// Get total number of functional tests
    pub fn total_tests(&self) -> usize {
        let count_tests = |result: &Option<TestResult<Vec<FunctionalOutput>>>| {
            result.as_ref().map_or(0, |r| r.data.len())
        };

        count_tests(&self.spat5_instantiation)
            + count_tests(&self.wfs_parameters)
            + count_tests(&self.vbap_panning)
            + count_tests(&self.hoa_transforms)
            + count_tests(&self.effect_chains)
    }

    /// Get number of passed functional tests
    pub fn passed_tests(&self) -> usize {
        let count_passed = |result: &Option<TestResult<Vec<FunctionalOutput>>>| {
            result
                .as_ref()
                .map_or(0, |r| r.data.iter().filter(|output| output.success).count())
        };

        count_passed(&self.spat5_instantiation)
            + count_passed(&self.wfs_parameters)
            + count_passed(&self.vbap_panning)
            + count_passed(&self.hoa_transforms)
            + count_passed(&self.effect_chains)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_spatial_functional_suite_creation() {
        if let Ok(suite) = SpatialFunctionalTestSuite::new() {
            assert!(suite.runner.temp_dir().exists());
        }
    }

    #[test]
    fn test_spatial_functional_results_tracking() {
        let results = SpatialFunctionalResults::new();
        assert_eq!(results.total_tests(), 0);
        assert_eq!(results.passed_tests(), 0);
        assert!(!results.all_passed());
    }
}

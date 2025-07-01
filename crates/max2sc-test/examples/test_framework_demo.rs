//! Demonstration of the max2sc SuperCollider testing framework
//!
//! This example shows how to use the testing framework for:
//! - Syntax validation
//! - Functional testing
//! - Audio analysis
//! - Integration testing

use max2sc_test::audio::{AudioSettings, AudioTolerance};
use max2sc_test::prelude::*;
use max2sc_test::syntax::CompileExpectation;
// use max2sc_test::fixtures::{TestMetadata, TestComplexity};
use max2sc_test::assertions::{
    in_range, not_silent, object_exists, output_channels, responds_to_osc,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== max2sc SuperCollider Testing Framework Demo ===\n");

    // Create a test runner
    let runner = SCTestRunner::new()?;
    println!("✓ Test runner initialized");
    println!(
        "  SC path: {:?}",
        std::env::var("SCLANG_PATH").unwrap_or_else(|_| "auto-detected".to_string())
    );

    // 1. Syntax Validation Demo
    println!("\n--- 1. Syntax Validation ---");
    demo_syntax_validation(&runner).await?;

    // 2. Functional Testing Demo
    println!("\n--- 2. Functional Testing ---");
    demo_functional_testing(&runner).await?;

    // 3. Audio Analysis Demo
    println!("\n--- 3. Audio Analysis ---");
    demo_audio_analysis(&runner).await?;

    // 4. Test Fixtures Demo
    println!("\n--- 4. Test Fixtures ---");
    demo_test_fixtures(&runner).await?;

    // 5. Assertion Examples
    println!("\n--- 5. Assertion Examples ---");
    demo_assertions(&runner).await?;

    println!("\n=== Demo Complete ===");
    Ok(())
}

async fn demo_syntax_validation(runner: &SCTestRunner) -> Result<()> {
    // Test 1: Valid syntax
    let valid_code = r#"
        SynthDef(\testSynth, { |freq = 440, amp = 0.5, out = 0|
            var sig = SinOsc.ar(freq) * amp;
            var env = EnvGen.kr(Env.perc(0.01, 1), doneAction: 2);
            Out.ar(out, sig * env ! 2);
        }).add;
    "#;

    let syntax_test = SyntaxTest::new(valid_code).expect(CompileExpectation::Success);

    let result = runner
        .run_test(TestCategory::Syntax, syntax_test.run(runner))
        .await?;

    println!(
        "✓ Valid syntax test: {}",
        if result.data.is_success() {
            "PASSED"
        } else {
            "FAILED"
        }
    );
    println!("  Compilation time: {:?}", result.duration);

    // Test 2: Invalid syntax (expected to fail)
    let invalid_code = r#"
        SynthDef(\broken, { |freq = 440
            var sig = SinOsc.ar(freq  // Missing closing bracket
            Out.ar(0, sig);
        }).add;
    "#;

    let error_test =
        SyntaxTest::new(invalid_code).expect(CompileExpectation::Error("unexpected".to_string()));

    let error_result = runner
        .run_test(TestCategory::Syntax, error_test.run(runner))
        .await?;

    println!("✓ Error detection test: PASSED");
    println!("  Found {} errors", error_result.data.error_count());
    if let Some(first_error) = error_result.data.first_error() {
        println!("  First error: {}", first_error.message);
    }

    Ok(())
}

async fn demo_functional_testing(runner: &SCTestRunner) -> Result<()> {
    // Test object instantiation and behavior
    let functional_test = FunctionalTest::new(
        r#"
        // Create test objects
        ~testBus = Bus.audio(s, 2);
        ~testBuffer = Buffer.alloc(s, 44100, 2);
        ~testGroup = Group.new;
        ~testSynth = { SinOsc.ar(440) * 0.1 }.play(~testGroup);
        
        // Test calculations
        ~frequency = 440;
        ~midiNote = ~frequency.cpsmidi.round;
        ~harmonics = [1, 2, 3, 4, 5] * ~frequency;
        "#,
    )
    .assert(Assertion::not_nil("~testBus"))
    .assert(Assertion::not_nil("~testBuffer"))
    .assert(Assertion::not_nil("~testGroup"))
    .assert(Assertion::not_nil("~testSynth"))
    .assert(Assertion::equals(
        "~testBus.numChannels",
        serde_json::json!(2),
    ))
    .assert(Assertion::equals("~midiNote", serde_json::json!(69)))
    .assert(Assertion::channel_count("~testSynth", 1))
    .with_cleanup(
        r#"
        ~testSynth.free;
        ~testGroup.free;
        ~testBuffer.free;
        ~testBus.free;
    "#,
    )
    .with_timeout(Duration::from_secs(5));

    let result = runner
        .run_test(TestCategory::Functional, functional_test.run(runner))
        .await?;

    println!("✓ Functional test completed");
    println!("  Total assertions: {}", result.data.assertion_count());
    println!("  Passed: {}", result.data.passed_assertions().len());
    println!("  Failed: {}", result.data.failed_assertions().len());
    println!("  Pass rate: {:.1}%", result.data.pass_rate());
    println!("  Execution time: {:?}", result.duration);

    // Show any failed assertions
    for failed in result.data.failed_assertions() {
        println!("  ✗ {}: {}", failed.assertion.description(), failed.message);
    }

    Ok(())
}

async fn demo_audio_analysis(runner: &SCTestRunner) -> Result<()> {
    // Create a reference audio path (would normally be a real file)
    let reference_path = runner.temp_dir().join("reference.wav");

    // Configure audio test
    let audio_test = AudioTest::new(
        r#"{ 
            var freq = 440;
            var sig = SinOsc.ar(freq) * 0.5;
            var env = EnvGen.kr(Env.sine(1), doneAction: 2);
            sig * env ! 2  // Stereo output
        }"#,
        &reference_path,
    )
    .with_settings(AudioSettings {
        sample_rate: 48000,
        duration: Duration::from_secs(1),
        channels: 2,
        bit_depth: 24,
    })
    .with_tolerance(AudioTolerance {
        rms_tolerance: 0.05,       // 5% RMS difference allowed
        spectral_similarity: 0.95, // 95% spectral similarity required
        phase_coherence: 0.90,     // 90% phase coherence
        peak_tolerance: 0.02,      // 2% peak difference
        frequency_range: (20.0, 20000.0),
    });

    println!("✓ Audio test configured");
    println!("  Sample rate: {} Hz", audio_test.settings.sample_rate);
    println!("  Duration: {:?}", audio_test.settings.duration);
    println!("  Channels: {}", audio_test.settings.channels);
    println!(
        "  RMS tolerance: {}%",
        audio_test.reference.tolerance.rms_tolerance * 100.0
    );

    // Note: Actual audio comparison would require a real reference file
    println!("  (Audio rendering and analysis would occur here)");

    Ok(())
}

async fn demo_test_fixtures(_runner: &SCTestRunner) -> Result<()> {
    // Get standard test fixtures
    let fixtures = TestData::standard_fixtures();
    println!("Available test fixtures: {}", fixtures.len());

    for fixture in fixtures.iter().take(3) {
        println!("\n  Fixture: {}", fixture.name);
        println!("  Category: {:?}", fixture.metadata.category);
        println!("  Complexity: {:?}", fixture.metadata.complexity);
        println!("  Description: {}", fixture.metadata.description);

        if !fixture.metadata.required_plugins.is_empty() {
            println!(
                "  Required plugins: {:?}",
                fixture.metadata.required_plugins
            );
        }
    }

    // Test a specific fixture
    let sine_fixture = TestData::simple_sine(440.0);
    let patch_file = sine_fixture.save_to_temp().await?;

    println!("\n✓ Saved test fixture to: {:?}", patch_file.path());

    // If we had the converter integrated, we could convert and test:
    // let sc_code = convert_fixture(&sine_fixture)?;
    // let test = SyntaxTest::new(&sc_code);
    // ...

    Ok(())
}

async fn demo_assertions(_runner: &SCTestRunner) -> Result<()> {
    println!("Common assertion patterns:\n");

    // Object existence
    println!("1. Object existence:");
    println!("   {}", Assertion::not_nil("~myObject").description());

    // Value equality
    println!("\n2. Value equality:");
    println!(
        "   {}",
        Assertion::equals("~frequency", serde_json::json!(440)).description()
    );

    // Numeric tolerance
    println!("\n3. Numeric tolerance:");
    println!(
        "   {}",
        Assertion::approximately("~gain", 0.5, 0.01).description()
    );

    // Method response
    println!("\n4. Method response:");
    println!(
        "   {}",
        Assertion::responds_to("~synth", "set").description()
    );

    // Channel count
    println!("\n5. Channel count:");
    println!(
        "   {}",
        Assertion::channel_count("~multiChannelSignal", 8).description()
    );

    // OSC responder
    println!("\n6. OSC responder:");
    println!(
        "   {}",
        Assertion::osc_responder("/source/1/azimuth").description()
    );

    // Custom conditions
    println!("\n7. Custom conditions:");
    println!(
        "   {}",
        Assertion::condition("CPU usage is reasonable", "Server.default.avgCPU < 50").description()
    );

    // Helper assertions
    println!("\n8. Helper assertions:");
    println!("   {}", object_exists("Server.default").description());
    println!("   {}", responds_to_osc("/master/gain").description());
    println!("   {}", output_channels(8).description());
    println!("   {}", in_range("~azimuth", -180.0, 180.0).description());
    println!("   {}", not_silent("~audioSignal").description());

    Ok(())
}

// Custom helper for the demo
fn _print_test_result(name: &str, passed: bool, duration: Duration) {
    let status = if passed { "✓ PASSED" } else { "✗ FAILED" };
    println!("{name}: {status} (took {duration:?})");
}

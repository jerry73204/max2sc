# max2sc-test

SuperCollider integration testing framework for max2sc. This crate provides comprehensive testing capabilities for validating Max to SuperCollider conversions.

## Features

- **Syntax Validation**: Fast compilation tests to verify generated SuperCollider code
- **Functional Testing**: Runtime object instantiation and behavior validation  
- **Audio Analysis**: Spectral comparison and audio output validation
- **Test Fixtures**: Pre-built test cases for common spatial audio scenarios
- **Assertion Framework**: Rich assertion library for SuperCollider tests

## Test Categories

| Category | Purpose | Speed | Example |
|----------|---------|-------|---------|
| **Syntax** | Compilation validation | Fast (< 1s) | Code compiles without errors |
| **Functional** | Object instantiation | Medium (1-5s) | Objects respond to methods |
| **Audio** | Output validation | Slow (5-30s) | Audio matches reference |
| **Integration** | End-to-end testing | Variable | Full conversion pipeline |

## Quick Start

```rust
use max2sc_test::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Create test runner
    let runner = SCTestRunner::new()?;
    
    // Syntax test
    let syntax_test = SyntaxTest::new("SinOsc.ar(440)");
    let result = runner.run_test(TestCategory::Syntax, async {
        syntax_test.run(&runner).await
    }).await?;
    
    println!("Test passed: {}", result.data.is_success());
    Ok(())
}
```

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Test Runner   │───▶│  SuperCollider   │───▶│  Test Results   │
│   (SCTestRunner)│    │   (sclang)       │    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │                        │                       │
         ▼                        ▼                       ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│ Test Categories │    │ Audio Analysis   │    │   Assertions    │
│ • Syntax        │    │ • Spectral       │    │ • Object exists │
│ • Functional    │    │ • Phase          │    │ • OSC response  │
│ • Audio         │    │ • Peak/RMS       │    │ • Value equals  │
│ • Integration   │    │ • Metrics        │    │ • Custom logic  │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## Test Writing

### Syntax Tests
```rust
// Test successful compilation
let test = SyntaxTest::new("SinOsc.ar(440, 0, 0.5)");

// Test expected errors  
let test = SyntaxTest::new("InvalidObject.ar()")
    .expect(CompileExpectation::Error("Class not found"));
```

### Functional Tests
```rust
let test = FunctionalTest::new("~synth = Synth(\\default)")
    .assert(Assertion::not_nil("~synth"))
    .assert(Assertion::responds_to("~synth", "set"))
    .with_cleanup("~synth.free;");
```

### Audio Tests
```rust
let test = AudioTest::new(
    "SinOsc.ar(440, 0, 0.5)",
    "reference_440hz.wav"
).with_tolerance(AudioTolerance {
    spectral_similarity: 0.95,
    rms_tolerance: 0.05,
    ..Default::default()
});
```

### Using Fixtures
```rust
// Built-in fixtures
let fixture = TestData::simple_sine(440.0);
let fixture = TestData::spat5_panoramix(1, 8);
let fixture = TestData::vbap_panner(8);

// All standard fixtures
let all_fixtures = TestData::standard_fixtures();
```

## Requirements

- **SuperCollider**: sclang must be installed and accessible
- **Rust**: 1.70+ with tokio async runtime
- **Audio Libraries**: hound for WAV processing, rustfft for analysis

## Environment Variables

- `SCLANG_PATH`: Custom path to sclang executable
- `MAX2SC_TEST_DATA`: Test data directory
- `MAX2SC_TEST_VERBOSE`: Enable verbose test output
- `MAX2SC_AUDIO_TOLERANCE`: Default audio comparison tolerance

## Examples

Run the basic example:
```bash
cargo run --example basic_test
```

Run all tests:
```bash
cargo test -p max2sc-test
```

## Integration with max2sc

This testing framework is designed to validate conversions produced by the max2sc converter:

```rust
use max2sc_test::prelude::*;
use max2sc_codegen::MaxToSCConverter;

// Convert Max patch to SuperCollider
let max_patch = load_max_patch("spatial_audio.maxpat")?;
let sc_code = MaxToSCConverter::convert(&max_patch)?;

// Test the generated code
let syntax_test = SyntaxTest::new(&sc_code);
let result = runner.run_test(TestCategory::Syntax, async {
    syntax_test.run(&runner).await
}).await?;

assert!(result.data.is_success());
```

## Future Enhancements

- [ ] Multi-version SuperCollider support
- [ ] Plugin dependency validation  
- [ ] Performance benchmarking
- [ ] Visual test result dashboards
- [ ] CI/CD integration helpers
- [ ] Property-based testing
- [ ] Fuzzing support

## Contributing

When adding new test types:

1. Implement test logic in appropriate module
2. Add corresponding assertion types
3. Update test fixtures as needed
4. Write comprehensive tests
5. Update documentation
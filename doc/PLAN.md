# Max MSP 8 to SuperCollider Converter - Project Plan

## Project Overview

A Rust-based converter that translates Max MSP 8 patches to SuperCollider code, with a focus on spatial sound features. The converter will parse Max patch files (.maxpat), analyze the signal flow and spatial processing, and generate equivalent SuperCollider code.

## Core Objectives

1. **Parse Max MSP 8 patch files** - Handle JSON-based .maxpat format
2. **Convert spatial audio objects** - Focus on multichannel, ambisonic, and spatial processing
3. **Generate idiomatic SuperCollider code** - Produce readable, maintainable SC code
4. **Preserve spatial relationships** - Maintain spatial parameters and routing

## Supported Spatial Features

### Max MSP 8 Objects (Priority)
- **Multichannel**: `mc.*` objects (mc.mixdown~, mc.dac~, mc.adc~)
- **Spatialization**: `spat5.*` package integration
- **Ambisonics**: HOA tools, ambisonic encoding/decoding
- **Basic Spatial**: pan~, pan2~, pan4~, pan8~
- **3D Audio**: HRTF processing, binaural rendering
- **Room Simulation**: reverb~, yafr~, gigaverb~

### SuperCollider Equivalents
- **Multichannel**: Array expansion, multichannel UGens
- **Spatialization**: PanAz, VBAP, Ambisonics UGens
- **Ambisonics**: HOA library, ATK (Ambisonic Toolkit)
- **Basic Spatial**: Pan2, Pan4, PanB, PanB2
- **3D Audio**: BinauralPan, HRTF UGens
- **Room Simulation**: FreeVerb, GVerb, JPverb

## Project Structure

For detailed architecture and project layout, see [ARCH.md](ARCH.md).

For current progress and task tracking, see [PROGRESS.md](PROGRESS.md).

For component mappings between Max and SuperCollider, see [MAPPING.md](MAPPING.md).

For data type specifications and serialization formats, see [DATA_TYPES.md](DATA_TYPES.md).

## Technical Considerations

### Parser Requirements
- **serde_json**: For Max patch JSON parsing
- **petgraph**: For signal flow graph representation
- **Custom DSL**: Internal representation of audio graph

### Code Generation Strategy
1. **Analyze signal flow** - Build directed graph of audio connections
2. **Identify spatial chains** - Group related spatial processing
3. **Generate SynthDefs** - Create modular, reusable synth definitions
4. **Handle control** - Map Max messages to SC patterns/routines

### Challenges & Solutions

**Challenge**: Max's visual patching vs SC's code-based approach
**Solution**: Generate well-structured, commented SC code with clear signal flow

**Challenge**: Different parameter ranges and units
**Solution**: Comprehensive mapping tables and conversion functions

**Challenge**: Max's implicit sample-accurate timing
**Solution**: Use SC's scheduling system appropriately (TempoClock, Routine)

**Challenge**: Spatial object complexity (especially spat5)
**Solution**: Start with basic objects, incrementally add complex features

## Conversion Mapping Strategy

### Object Mappings

#### Multichannel Processing
- `mc.*` objects → SC Array expansion with wrapper classes
- `mc.unpack~` → Array routing with channel distribution
- `mc.live.gain~` → Multichannel gain control arrays

#### Spatial Audio
- `spat5.panoramix~` → Modular encoder/processor/decoder architecture
- WFS configurations → `WFSArray` class with `DelayN` arrays
- VBAP → SC's VBAP UGen with speaker configuration
- HOA → Ambisonic Toolkit (ATK) integration

#### Routing & Control
- Complex routing matrices → `RoutingMatrix` class
- OSC namespace preservation (`/master/*`, `/bus/*`, `/track/*`)
- Parameter smoothing → `Lag`, `VarLag`, `Ramp` UGens

### Data Flow Architecture
1. **Input**: Max8 project directory
2. **Parse**: Extract patches, configurations, assets
3. **Analyze**: Build signal flow graph and dependencies
4. **Convert**: Generate SC classes, synths, and patterns
5. **Output**: Complete SC project with same functionality

### Output Project Structure
The converter generates a complete SuperCollider project:

```
output_project/
├── main.scd                 # Project startup file
├── config/
│   ├── speakers.yaml       # Speaker array configurations
│   ├── presets.yaml        # Converted presets/snapshots
│   └── routing.yaml        # Bus routing configuration
├── lib/
│   ├── SynthDefs.scd       # All converted SynthDefs
│   ├── Spatializers.scd    # Spatial processing classes
│   ├── OSCRouter.scd       # OSC handling
│   └── Utils.scd           # Helper functions
├── assets/
│   └── (copied audio files)
└── README.md               # Auto-generated documentation
```

## Error Handling Strategy

### Library Crates (thiserror)
All library crates (`max2sc-parser`, `max2sc-analyzer`, `max2sc-codegen`, `max2sc-spatial`) will use `thiserror` to define structured error types:

```rust
// Example from max2sc-parser
#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("Invalid Max patch format: {0}")]
    InvalidFormat(String),
    
    #[error("Unsupported Max version: {version}")]
    UnsupportedVersion { version: String },
    
    #[error("Object not found: {name}")]
    ObjectNotFound { name: String },
    
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
```

### Binary Crate (eyre)
The main `max2sc` binary will use `eyre` for application-level error handling with rich context:

```rust
use eyre::{Result, WrapErr};

fn main() -> Result<()> {
    // Install error report handler
    color_eyre::install()?;
    
    // Application logic with contextual errors
    let patch = load_patch(&args.input)
        .wrap_err("Failed to load Max patch")?;
        
    let sc_code = convert_patch(patch)
        .wrap_err_with(|| format!("Failed to convert patch: {}", args.input))?;
    
    Ok(())
}
```

## Dependencies

### Core Dependencies
```toml
[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"  # For configuration files
petgraph = "0.6"
thiserror = "2.0"  # For library error types
clap = { version = "4.0", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = "0.3"
regex = "1.0"
indexmap = "2.0"  # For ordered maps
walkdir = "2.0"  # For directory traversal

# Binary-specific dependencies
[dependencies]
eyre = "0.6"
color-eyre = "0.6"  # For pretty error reports

# Type crate dependencies
[dependencies.max2sc-core]
path = "crates/max2sc-core"

[dependencies.max2sc-max-types]
path = "crates/max2sc-max-types"

[dependencies.max2sc-sc-types]
path = "crates/max2sc-sc-types"
```

### Development Dependencies
```toml
[workspace.dev-dependencies]
criterion = "0.5"
proptest = "1.0"
pretty_assertions = "1.0"
insta = "1.0"  # For snapshot testing
tempfile = "3.0"  # For test file generation
```

## Testing Strategy

### Unit Tests
Each crate includes comprehensive unit tests:
- **Parser**: Test parsing of individual Max objects and patches
- **Analyzer**: Verify signal flow graph construction
- **Codegen**: Test SC code generation for each object type
- **Spatial**: Validate spatial calculations and conversions

### Integration Tests
Located in the main binary crate:
```rust
// crates/max2sc/tests/integration_tests.rs
use max2sc::convert_project;

#[test]
fn test_simple_stereo_panner() {
    let input = "tests/fixtures/simple_panner";
    let output_dir = tempfile::tempdir().unwrap();
    
    let result = convert_project(input, output_dir.path()).unwrap();
    
    // Verify output structure
    assert!(result.main_file.exists());
    assert!(result.config_dir.exists());
    assert!(result.lib_dir.exists());
    
    // Compare with expected output
    let main_content = std::fs::read_to_string(&result.main_file).unwrap();
    insta::assert_snapshot!(main_content);
}
```

### Component Tests
Each library crate has its own tests:
```rust
// crates/max2sc-parser/tests/spat5_tests.rs
use max2sc_parser::{parse_max_object, MaxObject};

#[test]
fn test_spat5_panoramix_parsing() {
    let obj: MaxObject = serde_json::from_str(r#"{
        "box": {
            "maxclass": "newobj",
            "text": "spat5.panoramix~ @speakers 8"
        }
    }"#).unwrap();
    
    assert_eq!(obj.class_name(), "spat5.panoramix~");
    assert_eq!(obj.get_attr("speakers"), Some(&8));
}
```

### Snapshot Testing
Use `insta` for comparing generated SC code:
- Capture generated output for reference patches
- Detect regressions in code generation
- Review changes with `cargo insta review`

### Performance Benchmarks
Each crate can have its own benchmarks:
```rust
// crates/max2sc/benches/conversion_bench.rs
use criterion::{criterion_group, criterion_main, Criterion};
use max2sc::convert_project;

fn benchmark_wfs_conversion(c: &mut Criterion) {
    let wfs_project = "tests/fixtures/wfs_basic";
    let output_dir = tempfile::tempdir().unwrap();
    
    c.bench_function("convert_32ch_wfs", |b| {
        b.iter(|| convert_project(wfs_project, output_dir.path()))
    });
}

criterion_group!(benches, benchmark_wfs_conversion);
criterion_main!(benches);
```


## Future Extensions

- GUI for patch preview and conversion options
- Reverse conversion (SC → Max)
- Support for additional Max packages (IRCAM tools, etc.)
- Live coding integration
- Web-based converter interface

## Resources

### Max MSP Documentation
- Max 8 SDK and file format documentation
- Cycling '74 spatial audio tutorials
- spat5 documentation

### SuperCollider Resources
- SC spatial audio tutorials
- Ambisonic Toolkit documentation
- VBAP implementation details

### Related Projects
- py2max (Python to Max)
- Various Max externals sources
- SuperCollider quarks for spatial audio
# max2sc Architecture

## Overview

The max2sc converter is designed as a modular Rust application that transforms Max MSP 8 projects into equivalent SuperCollider projects. The architecture emphasizes separation of concerns, extensibility, and maintainable code generation.

## Cargo Workspace Structure

```
max2sc/
├── Cargo.toml                 # Workspace root
├── PLAN.md
├── README.md
├── LICENSE
├── crates/
│   ├── max2sc/               # Main binary crate
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   └── main.rs
│   │   ├── tests/            # Integration tests
│   │   │   ├── fixtures/     # Test Max projects
│   │   │   │   ├── simple_panner/
│   │   │   │   ├── wfs_basic/
│   │   │   │   └── spat5_demo/
│   │   │   └── integration_tests.rs
│   │   └── examples/         # Example conversions
│   │       ├── basic_spatial.rs
│   │       └── convert_project.rs
│   ├── max2sc-core/          # Core types and traits
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── error.rs      # Common error types
│   │   │   ├── traits.rs     # Conversion traits
│   │   │   └── types.rs      # Shared types
│   │   └── tests/
│   │       └── core_tests.rs
│   ├── max2sc-max-types/     # Max data structures
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── patch.rs      # Patch format structures
│   │   │   ├── objects.rs    # Max object types
│   │   │   ├── connections.rs # Connection types
│   │   │   ├── attributes.rs # Attribute handling
│   │   │   └── osc.rs        # OSC configuration
│   │   └── tests/
│   │       ├── serialization_tests.rs
│   │       └── fixtures/
│   ├── max2sc-sc-types/      # SuperCollider data structures
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── synth_def.rs  # SynthDef structures
│   │   │   ├── ugen.rs       # UGen representations
│   │   │   ├── pattern.rs    # Pattern structures
│   │   │   ├── bus.rs        # Bus and routing
│   │   │   └── osc.rs        # OSC definitions
│   │   └── tests/
│   │       └── sc_types_tests.rs
│   ├── max2sc-parser/        # Max patch parser
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── json_parser.rs # JSON parsing logic
│   │   │   ├── osc_parser.rs  # OSC config parsing
│   │   │   └── validation.rs  # Input validation
│   │   └── tests/
│   │       ├── parser_tests.rs
│   │       └── spat5_tests.rs
│   ├── max2sc-analyzer/      # Signal flow analysis
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── graph.rs      # Signal flow graph
│   │   │   ├── routing.rs    # Audio routing analysis
│   │   │   └── spatial_analysis.rs
│   │   └── tests/
│   │       └── graph_tests.rs
│   ├── max2sc-codegen/       # SuperCollider code generation
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── synth_gen.rs  # SynthDef generation
│   │   │   ├── pattern_gen.rs # Pattern generation
│   │   │   ├── project_gen.rs # Project structure
│   │   │   └── formatting.rs  # Code formatting
│   │   └── tests/
│   │       ├── codegen_tests.rs
│   │       └── snapshots/    # Insta snapshots
│   └── max2sc-spatial/       # Spatial audio utilities
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs
│       │   ├── mapping.rs    # Object mapping tables
│       │   ├── conversions.rs # Parameter conversions
│       │   └── validation.rs
│       └── tests/
│           └── mapping_tests.rs
```

## Data Flow Architecture

```
┌─────────────────┐
│ Max8 Project    │
│ Directory       │
└────────┬────────┘
         │
         ▼
┌─────────────────┐     ┌──────────────────┐
│ Parser          │ ──> │ max2sc-max-types │
│ (max2sc-parser) │     │ (Serialized Max  │
└────────┬────────┘     │  structures)     │
         │              └──────────────────┘
         ▼
┌─────────────────┐     ┌──────────────────┐
│ Analyzer        │ <── │ max2sc-core      │
│ (max2sc-        │     │ (Shared types    │
│  analyzer)      │     │  and traits)     │
└────────┬────────┘     └──────────────────┘
         │
         ▼
┌─────────────────┐     ┌──────────────────┐
│ Code Generator  │ ──> │ max2sc-sc-types  │
│ (max2sc-        │     │ (SC structures   │
│  codegen)       │     │  and formats)    │
└────────┬────────┘     └──────────────────┘
         │
         ▼
┌─────────────────┐
│ SC Project      │
│ Output          │
└─────────────────┘
```

## Output Project Structure

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

## Component Architecture

### Core Types (max2sc-core)

Shared types and traits used across all crates:

- **Error Types**: Common error definitions using `thiserror`
- **Conversion Traits**: `ToSuperCollider`, `FromMax` traits
- **Common Types**: Coordinates, audio formats, channel counts
- **Result Types**: Standardized `Result<T, E>` definitions

### Max Types (max2sc-max-types)

Serializable data structures for Max MSP formats:

- **Patch Format**: 
  ```rust
  #[derive(Serialize, Deserialize)]
  struct MaxPatch {
      patcher: Patcher,
      fileversion: i32,
      appversion: AppVersion,
  }
  ```
- **Object Types**: Representations for all Max objects
- **Connection Types**: Inlet/outlet connections
- **Attributes**: Parameter storage and serialization
- **OSC Configuration**: Speaker arrays, routing tables

### SuperCollider Types (max2sc-sc-types)

Serializable data structures for SC generation:

- **SynthDef Structure**:
  ```rust
  #[derive(Serialize, Deserialize)]
  struct SynthDef {
      name: String,
      params: Vec<Parameter>,
      ugens: Vec<UGen>,
      variants: Option<Vec<Variant>>,
  }
  ```
- **UGen Representations**: Type-safe UGen definitions
- **Pattern Types**: Event patterns, routines
- **Bus Definitions**: Audio/control bus structures
- **OSC Mappings**: SC OSC responder definitions

### Parser (max2sc-parser)

Responsible for reading and parsing Max patch files:

- **Input**: `.maxpat` JSON files, OSC config files
- **Output**: Populated `max2sc-max-types` structures
- **Dependencies**: Uses `max2sc-max-types` for data structures
- **Key Functions**:
  - `parse_patch()`: Parse .maxpat files
  - `parse_osc_config()`: Parse speaker configurations
  - `validate_patch()`: Ensure patch integrity

### Analyzer (max2sc-analyzer)

Builds understanding of the patch structure:

- **Input**: `max2sc-max-types` structures from parser
- **Output**: Analyzed graph with routing information
- **Signal Flow Graph**: Uses `petgraph` to represent connections
- **Routing Analysis**: Identifies multichannel routing patterns
- **Spatial Configuration**: Extracts speaker arrays, spatial parameters
- **Dependency Resolution**: Determines processing order

### Code Generator (max2sc-codegen)

Produces SuperCollider code:

- **Input**: Analyzed structures from analyzer
- **Output**: `max2sc-sc-types` structures and SC code
- **SynthDef Generation**: Converts Max objects to SC synths
- **Pattern Generation**: Handles sequencing and control
- **OSC Router**: Maintains Max's OSC namespace
- **Project Structure**: Creates organized SC project layout

### Spatial Utilities (max2sc-spatial)

Shared spatial audio functionality:

- **Object Mappings**: Max → SC object conversion tables
- **Parameter Conversions**: Unit and range conversions
- **Validation**: Ensures spatial configurations are valid
- **Common Algorithms**: Shared spatial processing logic
- **Dependencies**: Uses both `max2sc-max-types` and `max2sc-sc-types`

## Key Design Patterns

### 1. Visitor Pattern
Used for traversing the Max patch structure and generating appropriate SC code.

### 2. Builder Pattern
Used for constructing complex SC objects like SynthDefs and spatial processors.

### 3. Strategy Pattern
Different code generation strategies for different Max object types.

### 4. Factory Pattern
Creating appropriate SC representations based on Max object types.

## Error Handling Strategy

### Library Crates (thiserror)
All library crates use `thiserror` for structured error types:

```rust
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
The main binary uses `eyre` for application-level error handling with rich context:

```rust
use eyre::{Result, WrapErr};

fn main() -> Result<()> {
    color_eyre::install()?;
    
    let patch = load_patch(&args.input)
        .wrap_err("Failed to load Max patch")?;
        
    let sc_code = convert_patch(patch)
        .wrap_err_with(|| format!("Failed to convert patch: {}", args.input))?;
    
    Ok(())
}
```

## Extension Points

### Adding New Object Mappings

1. Add mapping to `max2sc-spatial/src/mapping.rs`
2. Implement parser support in `max2sc-parser`
3. Add code generation in `max2sc-codegen`
4. Write tests for the new mapping

### Supporting New Spatial Algorithms

1. Define algorithm in `max2sc-spatial`
2. Create analyzer rules in `max2sc-analyzer`
3. Implement SC code generation
4. Add integration tests

### Custom Output Formats

The architecture allows for different output strategies:
- Single-file SC scripts
- Modular SC class libraries
- Quark packages
- Live coding setups

## Performance Considerations

### Parallel Processing
- Parse multiple patches concurrently
- Generate code for independent objects in parallel
- Use `rayon` for data parallelism where appropriate

### Memory Efficiency
- Stream large patch files
- Generate code incrementally
- Reuse common patterns and templates

### Optimization Opportunities
- Cache parsed objects
- Precompute spatial calculations
- Optimize generated SC code for performance
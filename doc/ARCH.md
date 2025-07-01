# max2sc Architecture

This document describes the project structure and component organization of the max2sc converter.

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


## Dependencies

Each crate has specific dependencies based on its role:

- **max2sc** (binary): `clap`, `eyre`, `color-eyre`, `tracing`
- **max2sc-core**: `thiserror`, `serde`
- **max2sc-max-types**: `serde`, `serde_json`
- **max2sc-sc-types**: `serde`, `serde_yaml`
- **max2sc-parser**: `serde_json`, `regex`
- **max2sc-analyzer**: `petgraph`, `indexmap`
- **max2sc-codegen**: `tera` (templating), `prettytable-rs`
- **max2sc-spatial**: Mathematical libraries as needed

## Build System

The project uses Cargo workspace features:
- Shared dependencies via `[workspace.dependencies]`
- Unified versioning
- Parallel compilation of crates
- Feature flags for optional functionality
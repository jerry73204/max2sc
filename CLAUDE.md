# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Build and Check
```bash
# Check all crates compile
cargo check --all-targets

# Build entire workspace
cargo build --all-targets

# Build with release optimizations
cargo build --all-targets --release

# Run the main CLI application
cargo run --bin max2sc
```

### Testing
```bash
# Run all tests in workspace
cargo test --all-targets

# Run tests for specific crate
cargo test -p max2sc-core

# Run integration tests
cargo test --test integration_tests

# Run tests with output
cargo test -- --nocapture
```

### Development Tools
```bash
# Format code
cargo +nightly fmt

# Run lints
cargo clippy

# Check documentation
cargo doc --no-deps --open

# Generate and view documentation for all crates
cargo doc --workspace --no-deps --open
```

## Architecture Overview

This is a **Rust workspace** that converts Max MSP 8 projects to SuperCollider code, focusing on spatial audio processing. The architecture follows a modular pipeline design:

**Data Flow**: Max8 Project → Parser → Analyzer → CodeGen → SuperCollider Project

### Core Architecture Pattern

The codebase uses a **conversion pipeline** with distinct phases:

1. **Parsing** (`max2sc-parser`): Reads `.maxpat` JSON files into structured data
2. **Analysis** (`max2sc-analyzer`): Builds signal flow graphs using `petgraph` and extracts spatial configurations  
3. **Code Generation** (`max2sc-codegen`): Produces SuperCollider SynthDefs, patterns, and project structure
4. **Output**: Complete SC project with preserved OSC namespace

### Key Type System

**Three-tier type system** for clean separation:
- `max2sc-max-types`: Max MSP patch format structures (`MaxPatch`, `BoxContent`, `PatchLine`)
- `max2sc-sc-types`: SuperCollider output structures (`SynthDef`, `UGen`, `Pattern`, `SCProject`)  
- `max2sc-core`: Shared types and conversion traits (`Position3D`, `AudioFormat`, `ToSuperCollider`)

### Conversion Traits

The core conversion system uses two key traits:
```rust
// Convert from Max representation to SC representation
trait ToSuperCollider {
    type Output;
    type Error;
    fn to_supercollider(&self) -> Result<Self::Output, Self::Error>;
}

// Create objects from Max data
trait FromMax<T> {
    type Error;
    fn from_max(max_obj: T) -> Result<Self, Self::Error>;
}
```

### Spatial Audio Focus

The converter specializes in **spatial audio objects**:
- **SPAT5** objects (spat5.panoramix~, spat5.hoa.*) → Custom SC classes + ATK integration
- **Multichannel** (mc.*) → SC Array operations  
- **WFS/VBAP** → Speaker array configurations
- **OSC namespace preservation** for compatibility with existing workflows

### Error Handling Strategy

**Dual error handling pattern**:
- **Library crates**: Use `thiserror` for structured error types
- **Binary crate**: Use `eyre` with context for rich error reporting

```rust
// Library pattern
#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("Invalid Max patch format: {0}")]
    InvalidFormat(String),
    // ...
}

// Binary pattern  
fn main() -> eyre::Result<()> {
    let patch = load_patch(&args.input)
        .wrap_err("Failed to load Max patch")?;
    // ...
}
```

## Implementation Guidelines

### TODO Pattern

Functions use `todo!()` for unimplemented features to fail explicitly rather than return dummy values. When implementing:

1. Remove the `todo!()` 
2. Implement the actual logic
3. Add comprehensive error handling
4. Write tests for the implementation

### Spatial Object Mapping

Reference `MAPPING.md` for object conversion strategies. Key mappings:
- `spat5.panoramix~` → Modular encoder/processor/decoder architecture
- `spat5.hoa.*` → ATK (Ambisonic Toolkit) integration  
- `mc.*` → Array expansion with wrapper classes

### Testing Strategy

- **Unit tests**: Per-crate in `src/` modules
- **Integration tests**: In `crates/max2sc/tests/` 
- **Fixtures**: Test Max projects in `tests/fixtures/`
- **Snapshot testing**: Use `insta` for SC code generation validation

### Max8 Project Analysis

The `max8_source/` directory contains the reference Max8 project being converted. Use the Python analysis scripts:

```bash
# Analyze patch objects and generate mappings
python scripts/analyze_max_patches.py

# Extract OSC namespace  
python scripts/extract_osc_commands.py
```

Key findings: 419 unique objects, 140+ SPAT5 types, 82-channel processing, extensive OSC control.

## Documentation Structure

- `PLAN.md`: Project overview and objectives
- `ARCH.md`: Detailed workspace structure and component architecture  
- `MAPPING.md`: Max↔SC object conversion strategies
- `DATA_TYPES.md`: Serialization format specifications
- `PROGRESS.md`: Task tracking and implementation status
- `CONVERSION_STRATEGIES.md`: Implementation approaches
- `NOTES_Max8_Project_Analysis.md`: Analysis of the source Max8 project

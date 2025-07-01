# max2sc Design Decisions

This document outlines the key design decisions made for the max2sc converter project.

## Core Design Philosophy

### 1. Modular Conversion Pipeline

**Decision**: Use a multi-stage pipeline architecture with distinct phases.

**Rationale**: 
- Clean separation of concerns
- Each phase can be tested independently
- Easier to extend or modify individual stages
- Parallel processing opportunities

**Implementation**:
```
Max8 Project → Parser → Analyzer → CodeGen → SuperCollider Project
```

### 2. Type System Architecture

**Decision**: Three-tier type system with clear separation.

**Rationale**:
- `max2sc-max-types`: Represents Max MSP patch format faithfully
- `max2sc-sc-types`: Represents SuperCollider structures idiomatically  
- `max2sc-core`: Shared types and conversion traits

This separation allows each representation to be optimal for its domain without compromise.

### 3. Conversion Trait Design

**Decision**: Use two complementary traits for conversion.

```rust
trait ToSuperCollider {
    type Output;
    type Error;
    fn to_supercollider(&self) -> Result<Self::Output, Self::Error>;
}

trait FromMax<T> {
    type Error;
    fn from_max(max_obj: T) -> Result<Self, Self::Error>;
}
```

**Rationale**:
- Explicit conversion points
- Type-safe transformations
- Clear error handling
- Extensible for new object types

### 4. Error Handling Strategy

**Decision**: Dual error handling approach.

**Library Crates**: Use `thiserror` for structured errors
```rust
#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("Invalid Max patch format: {0}")]
    InvalidFormat(String),
}
```

**Binary Crate**: Use `eyre` for rich context
```rust
let patch = load_patch(&args.input)
    .wrap_err("Failed to load Max patch")?;
```

**Rationale**:
- Libraries need precise, programmatic errors
- Applications need user-friendly error messages
- Context helps debugging complex conversions

## Spatial Audio Design

### 1. Preserve OSC Namespace

**Decision**: Maintain Max's OSC namespace structure exactly.

**Rationale**:
- Enables drop-in replacement for existing workflows
- Controllers/interfaces continue working unchanged
- Reduces migration friction

**Example**:
- `/master/gain` → Master bus gain control
- `/track/[n]/azim` → Track azimuth position
- `/bus/[n]/speaker/[m]/delay` → Speaker delays

### 2. Modular Spatialization Architecture

**Decision**: Implement spatialization as encoder → processor → decoder chain.

**Rationale**:
- Matches SPAT5's internal architecture
- Allows mixing different spatial methods
- Easier to extend with new algorithms
- Clear signal flow

### 3. ATK Integration for HOA

**Decision**: Use Ambisonic Toolkit (ATK) for all HOA processing.

**Rationale**:
- Mature, well-tested implementation
- Comprehensive feature set
- Active development and support
- Avoid reinventing complex algorithms

## Conversion Strategy

### 1. Signal Flow Graph Analysis

**Decision**: Use `petgraph` to build and analyze signal flow.

**Rationale**:
- Efficient graph algorithms
- Handles cycles and complex routing
- Enables dependency resolution
- Supports parallel processing analysis

### 2. Incremental Implementation

**Decision**: Use `todo!()` for unimplemented features.

**Rationale**:
- Fail fast rather than silent errors
- Clear implementation roadmap
- Easier to track progress
- Prevents accidental use of stubs

### 3. Project Structure Generation

**Decision**: Generate complete, organized SC projects.

```
output_project/
├── main.scd                 # Project startup
├── config/                  # Configuration files
├── lib/                     # SC code libraries
├── assets/                  # Audio files
└── README.md               # Documentation
```

**Rationale**:
- Self-contained projects
- Clear organization
- Easy to understand and modify
- Follows SC community conventions

## Performance Considerations

### 1. Lazy Evaluation

**Decision**: Parse and process only what's needed.

**Rationale**:
- Large patches (2000+ objects) need efficiency
- Not all objects require deep analysis
- Reduces memory usage

### 2. Parallel Processing

**Decision**: Design for concurrent execution where possible.

**Rationale**:
- Modern CPUs have multiple cores
- Many conversion tasks are independent
- Significant speedup for large projects

### 3. Caching Strategy

**Decision**: Cache parsed objects and analysis results.

**Rationale**:
- Avoid redundant parsing
- Speed up iterative development
- Reduce computational overhead

## Testing Philosophy

### 1. Multi-Level Testing

**Decision**: Implement unit, integration, and SC validation tests.

**Rationale**:
- Unit tests for individual components
- Integration tests for complete workflows
- SC validation ensures output actually works
- Different test levels catch different issues

### 2. Snapshot Testing

**Decision**: Use `insta` for generated code validation.

**Rationale**:
- Easy to review changes
- Catches regressions
- Documents expected output
- Simplifies test maintenance

### 3. Real-World Test Data

**Decision**: Use actual Max patches and configurations.

**Rationale**:
- Catches edge cases
- Validates against real usage
- Builds confidence in converter
- Discovers undocumented behaviors

## Extension Points

### 1. Plugin Architecture

**Decision**: Design for extensibility from the start.

**Rationale**:
- New Max objects appear regularly
- Different conversion strategies needed
- Community contributions possible
- Future-proof design

### 2. Custom Output Formats

**Decision**: Abstract output generation.

**Rationale**:
- Support different SC coding styles
- Enable Quark package generation
- Allow live coding formats
- Adapt to user preferences

### 3. Configurable Conversion

**Decision**: Provide CLI flags for conversion options.

**Rationale**:
- Different use cases need different outputs
- Performance vs. accuracy tradeoffs
- User control over process
- Debugging and development needs

## Future Considerations

### 1. Bidirectional Conversion

**Decision**: Design with potential SC→Max conversion in mind.

**Rationale**:
- Round-trip conversion valuable
- Shared infrastructure possible
- Learn from both directions

### 2. GUI Integration

**Decision**: Keep GUI separate but plan for it.

**Rationale**:
- CLI first for automation
- GUI for user-friendliness
- Different deployment scenarios
- Progressive enhancement

### 3. Web-Based Converter

**Decision**: Consider WASM compilation path.

**Rationale**:
- Browser-based tool accessibility
- No installation required
- Cross-platform by default
- Modern deployment option
# max2sc Development Guide

This guide covers setting up the development environment, building, testing, and contributing to max2sc.

## Prerequisites

### Required Software

- **Rust**: 1.70.0 or later
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  rustup default stable
  rustup component add rustfmt
  ```

- **Cargo Nextest**: For better test execution
  ```bash
  cargo install cargo-nextest
  ```

- **SuperCollider**: 3.11.0 or later (for integration tests)
  - macOS: `brew install supercollider`
  - Linux: `apt-get install supercollider` or compile from source
  - Windows: Download from https://supercollider.github.io/

### Optional Tools

- **cargo-watch**: For auto-recompilation
  ```bash
  cargo install cargo-watch
  ```

- **cargo-audit**: For security audits
  ```bash
  cargo install cargo-audit
  ```

- **cargo-outdated**: For dependency management
  ```bash
  cargo install cargo-outdated
  ```

## Environment Setup

### Clone Repository
```bash
git clone https://github.com/yourusername/max2sc.git
cd max2sc
```

### First-Time Setup
```bash
# Install development dependencies
make setup

# Verify installation
make check-env
```

### Environment Variables
```bash
# Optional: Set custom SuperCollider path
export SCLANG_PATH=/usr/local/bin/sclang

# Optional: Enable verbose test output
export MAX2SC_TEST_VERBOSE=1

# Optional: Set test data directory
export MAX2SC_TEST_DATA=./tests/data
```

## Building

### Quick Build
```bash
# Debug build (fast compilation)
make build

# Release build (optimized)
make release

# Build specific crate
make build-parser
make build-analyzer
make build-codegen
```

### Clean Build
```bash
# Clean all build artifacts
make clean

# Clean and rebuild
make rebuild
```

## Testing

### Run All Tests
```bash
# Using nextest (recommended)
make test

# Run tests without fail-fast
make test-all

# Traditional cargo test
make test-cargo
```

### Test Specific Components
```bash
# Test individual crates
make test-parser
make test-analyzer
make test-codegen
make test-spatial

# Test with output
make test-verbose
```

### Integration Tests
```bash
# Run SuperCollider integration tests
make test-integration

# Run specific integration test
make test-integration TEST=test_spat5_panoramix
```

### Coverage Report
```bash
# Generate test coverage report
make coverage

# Open coverage report in browser
make coverage-open
```

## Code Quality

### Formatting
```bash
# Format all code using nightly rustfmt
make fmt

# Check formatting without changes
make fmt-check
```

### Linting
```bash
# Run clippy lints
make lint

# Run clippy with pedantic lints
make lint-pedantic

# Fix clippy warnings automatically
make lint-fix
```

### Documentation
```bash
# Build documentation
make doc

# Build and open documentation
make doc-open

# Check documentation links
make doc-check
```

## Development Workflow

### Watch Mode
```bash
# Auto-rebuild on changes
make watch

# Watch and run tests
make watch-test
```

### Pre-commit Checks
```bash
# Run all checks before committing
make pre-commit

# This runs:
# - Format check
# - Lint check  
# - Tests
# - Doc build
```

### Benchmarking
```bash
# Run benchmarks
make bench

# Run specific benchmark
make bench BENCH=parse_large_patch

# Compare benchmarks
make bench-compare
```

## Project Structure

```
max2sc/
├── Cargo.toml              # Workspace configuration
├── Makefile               # Build automation
├── crates/                # Library crates
│   ├── max2sc/           # Main binary
│   ├── max2sc-core/      # Core types and traits
│   ├── max2sc-max-types/ # Max data structures
│   ├── max2sc-sc-types/  # SC data structures
│   ├── max2sc-parser/    # Max patch parser
│   ├── max2sc-analyzer/  # Signal flow analyzer
│   ├── max2sc-codegen/   # SC code generator
│   └── max2sc-spatial/   # Spatial audio utilities
├── tests/                 # Integration tests
├── benches/              # Benchmarks
└── doc/                  # Documentation

```

## Adding New Features

### 1. Adding a New Max Object Mapping

1. Add the mapping to `crates/max2sc-spatial/src/mapping.rs`
2. Implement parser support in `crates/max2sc-parser/src/objects/`
3. Add analyzer rules in `crates/max2sc-analyzer/src/spatial/`
4. Implement code generation in `crates/max2sc-codegen/src/converters/`
5. Write tests in the respective test modules
6. Update documentation

### 2. Adding a New Spatial Algorithm

1. Define the algorithm in `crates/max2sc-spatial/src/algorithms/`
2. Create analyzer detection in `crates/max2sc-analyzer/src/spatial_analysis.rs`
3. Implement SC code generation
4. Add integration tests
5. Document the algorithm

### 3. Adding a New Output Format

1. Define format structure in `crates/max2sc-sc-types/`
2. Implement generator in `crates/max2sc-codegen/src/formats/`
3. Add CLI option in `crates/max2sc/src/cli.rs`
4. Write tests and documentation

## Debugging

### Debug Builds
```bash
# Build with debug symbols
make debug

# Run with backtrace
RUST_BACKTRACE=1 cargo run -- input.maxpat
```

### Logging
```bash
# Set log level
RUST_LOG=debug cargo run -- input.maxpat

# Log specific modules
RUST_LOG=max2sc_parser=debug cargo run -- input.maxpat
```

### Test Debugging
```bash
# Run single test with output
make test-one TEST=test_parse_patch

# Keep test artifacts
MAX2SC_TEST_KEEP_ARTIFACTS=1 make test
```

## Release Process

### Version Bump
```bash
# Bump version (major.minor.patch)
make version VERSION=0.2.0
```

### Release Checklist
```bash
# Run full release checks
make release-check

# This includes:
# - All tests pass
# - No clippy warnings
# - Documentation builds
# - Changelog updated
# - Version bumped
```

### Create Release
```bash
# Create release build
make release

# Package for distribution
make package
```

## Troubleshooting

### Common Issues

1. **SuperCollider not found**
   ```bash
   export SCLANG_PATH=/path/to/sclang
   ```

2. **Test failures on CI**
   - Check SC version compatibility
   - Verify all test data is committed
   - Check for platform-specific issues

3. **Slow compilation**
   ```bash
   # Use sccache for faster rebuilds
   cargo install sccache
   export RUSTC_WRAPPER=sccache
   ```

4. **Memory issues with large patches**
   ```bash
   # Increase stack size
   export RUST_MIN_STACK=8388608
   ```

## Contributing Guidelines

### Code Style
- Follow Rust naming conventions
- Use `cargo +nightly fmt` before committing
- Add documentation for public APIs
- Write tests for new functionality

### Commit Messages
- Use conventional commits format
- Include relevant issue numbers
- Keep commits focused and atomic

### Pull Request Process
1. Create feature branch from `main`
2. Implement feature with tests
3. Run `make pre-commit`
4. Update documentation
5. Submit PR with description

### Review Checklist
- [ ] Tests pass
- [ ] Documentation updated
- [ ] Changelog entry added
- [ ] No clippy warnings
- [ ] Performance impact considered
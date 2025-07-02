# max2sc Makefile
# Development automation for the max2sc project

# Configuration
CARGO = cargo
CARGO_NEXTEST = cargo nextest
RUSTFMT = cargo +nightly fmt
PROJECT_NAME = max2sc

# Color codes for output
GREEN = \033[0;32m
YELLOW = \033[0;33m
RED = \033[0;31m
NC = \033[0m # No Color

# Default target
.PHONY: all
all: build

# Setup development environment
.PHONY: setup
setup:
	@echo "$(GREEN)Setting up development environment...$(NC)"
	rustup default stable
	rustup component add rustfmt --toolchain nightly
	cargo install cargo-nextest
	cargo install cargo-watch
	cargo install cargo-audit
	cargo install cargo-outdated
	@echo "$(GREEN)Setup complete!$(NC)"

# Check environment
.PHONY: check-env
check-env:
	@echo "$(GREEN)Checking environment...$(NC)"
	@rustc --version
	@cargo --version
	@cargo nextest --version || echo "$(YELLOW)Warning: cargo-nextest not installed$(NC)"
	@which sclang > /dev/null && sclang -v || echo "$(YELLOW)Warning: SuperCollider not found$(NC)"

# Build targets
.PHONY: build
build:
	@echo "$(GREEN)Building debug version...$(NC)"
	$(CARGO) build --all-targets

.PHONY: release
release:
	@echo "$(GREEN)Building release version...$(NC)"
	$(CARGO) build --all-targets --release

.PHONY: debug
debug:
	@echo "$(GREEN)Building with debug info...$(NC)"
	RUSTFLAGS="-g" $(CARGO) build --all-targets

# Individual crate builds
.PHONY: build-parser
build-parser:
	$(CARGO) build -p max2sc-parser

.PHONY: build-analyzer
build-analyzer:
	$(CARGO) build -p max2sc-analyzer

.PHONY: build-codegen
build-codegen:
	$(CARGO) build -p max2sc-codegen

.PHONY: build-spatial
build-spatial:
	$(CARGO) build -p max2sc-spatial

# Testing targets
.PHONY: test
test:
	@echo "$(GREEN)Running tests with nextest...$(NC)"
	$(CARGO_NEXTEST) run --no-fail-fast --all-targets

.PHONY: test-cargo
test-cargo:
	@echo "$(GREEN)Running tests with cargo test...$(NC)"
	$(CARGO) test --all-targets

.PHONY: test-verbose
test-verbose:
	@echo "$(GREEN)Running tests with output...$(NC)"
	$(CARGO) test --all-targets -- --nocapture

# Component tests
.PHONY: test-parser
test-parser:
	$(CARGO_NEXTEST) run -p max2sc-parser --no-fail-fast

.PHONY: test-analyzer
test-analyzer:
	$(CARGO_NEXTEST) run -p max2sc-analyzer --no-fail-fast

.PHONY: test-codegen
test-codegen:
	$(CARGO_NEXTEST) run -p max2sc-codegen --no-fail-fast

.PHONY: test-spatial
test-spatial:
	$(CARGO_NEXTEST) run -p max2sc-spatial --no-fail-fast

.PHONY: test-cli
test-cli:
	$(CARGO_NEXTEST) run -p max2sc --no-fail-fast

# Integration tests
.PHONY: test-integration
test-integration:
	@echo "$(GREEN)Running integration tests...$(NC)"
	$(CARGO) test --test integration_tests -- --nocapture

# Single test
.PHONY: test-one
test-one:
	@echo "$(GREEN)Running test: $(TEST)$(NC)"
	$(CARGO) test $(TEST) -- --nocapture

# Code quality
.PHONY: format
format:
	@echo "$(GREEN)Formatting code...$(NC)"
	$(RUSTFMT)

.PHONY: format-check
format-check:
	@echo "$(GREEN)Checking formatting...$(NC)"
	$(RUSTFMT) -- --check

.PHONY: lint
lint:
	@echo "$(GREEN)Running clippy...$(NC)"
	$(CARGO) clippy --all-targets --all-features -- -D warnings

.PHONY: lint-pedantic
lint-pedantic:
	@echo "$(GREEN)Running clippy (pedantic)...$(NC)"
	$(CARGO) clippy --all-targets --all-features -- -W clippy::pedantic -D warnings

.PHONY: lint-fix
lint-fix:
	@echo "$(GREEN)Fixing clippy warnings...$(NC)"
	$(CARGO) clippy --fix --allow-dirty --allow-staged

# Documentation
.PHONY: doc
doc:
	@echo "$(GREEN)Building documentation...$(NC)"
	$(CARGO) doc --no-deps --all-features

.PHONY: doc-open
doc-open:
	@echo "$(GREEN)Building and opening documentation...$(NC)"
	$(CARGO) doc --no-deps --all-features --open

.PHONY: doc-check
doc-check:
	@echo "$(GREEN)Checking documentation...$(NC)"
	RUSTDOCFLAGS="-D warnings" $(CARGO) doc --no-deps --all-features

# Benchmarking
.PHONY: bench
bench:
	@echo "$(GREEN)Running benchmarks...$(NC)"
	$(CARGO) bench

.PHONY: bench-compare
bench-compare:
	@echo "$(GREEN)Running benchmark comparison...$(NC)"
	$(CARGO) bench -- --save-baseline main

# Clean targets
.PHONY: clean
clean:
	@echo "$(GREEN)Cleaning build artifacts...$(NC)"
	$(CARGO) clean

.PHONY: rebuild
rebuild: clean build

# Watch targets
.PHONY: watch
watch:
	@echo "$(GREEN)Watching for changes...$(NC)"
	cargo watch -x build

.PHONY: watch-test
watch-test:
	@echo "$(GREEN)Watching and testing...$(NC)"
	cargo watch -x test

# Pre-commit checks
.PHONY: pre-commit
pre-commit: format-check lint test doc
	@echo "$(GREEN)All pre-commit checks passed!$(NC)"

# Coverage
.PHONY: coverage
coverage:
	@echo "$(GREEN)Generating coverage report...$(NC)"
	cargo tarpaulin --out Html --output-dir target/coverage

.PHONY: coverage-open
coverage-open: coverage
	@echo "$(GREEN)Opening coverage report...$(NC)"
	open target/coverage/index.html || xdg-open target/coverage/index.html

# Dependency management
.PHONY: audit
audit:
	@echo "$(GREEN)Running security audit...$(NC)"
	cargo audit

.PHONY: outdated
outdated:
	@echo "$(GREEN)Checking for outdated dependencies...$(NC)"
	cargo outdated

.PHONY: update
update:
	@echo "$(GREEN)Updating dependencies...$(NC)"
	$(CARGO) update

# Release management
.PHONY: version
version:
	@echo "$(GREEN)Bumping version to $(VERSION)...$(NC)"
	sed -i 's/version = ".*"/version = "$(VERSION)"/' Cargo.toml
	sed -i 's/version = ".*"/version = "$(VERSION)"/' crates/*/Cargo.toml

.PHONY: release-check
release-check: fmt-check lint test doc audit
	@echo "$(GREEN)Release checks complete!$(NC)"

.PHONY: package
package: release
	@echo "$(GREEN)Creating release package...$(NC)"
	mkdir -p dist
	cp target/release/$(PROJECT_NAME) dist/
	cp README.md LICENSE dist/
	cd dist && tar -czf $(PROJECT_NAME)-$(shell grep version Cargo.toml | head -1 | cut -d'"' -f2).tar.gz *

# Help target
.PHONY: help
help:
	@echo "$(GREEN)max2sc Development Commands$(NC)"
	@echo ""
	@echo "Setup & Environment:"
	@echo "  make setup          - Install development dependencies"
	@echo "  make check-env      - Check development environment"
	@echo ""
	@echo "Building:"
	@echo "  make build          - Build debug version (default)"
	@echo "  make release        - Build release version"
	@echo "  make clean          - Clean build artifacts"
	@echo "  make rebuild        - Clean and rebuild"
	@echo ""
	@echo "Testing:"
	@echo "  make test           - Run all tests with nextest"
	@echo "  make test-verbose   - Run tests with output"
	@echo "  make test-one TEST=name - Run specific test"
	@echo ""
	@echo "Code Quality:"
	@echo "  make format         - Format code with rustfmt"
	@echo "  make lint           - Run clippy lints"
	@echo "  make doc            - Build documentation"
	@echo "  make pre-commit     - Run all pre-commit checks"
	@echo ""
	@echo "Development:"
	@echo "  make watch          - Watch and rebuild on changes"
	@echo "  make watch-test     - Watch and test on changes"
	@echo "  make coverage       - Generate test coverage report"
	@echo ""
	@echo "Release:"
	@echo "  make release-check  - Run release checks"
	@echo "  make package        - Create release package"

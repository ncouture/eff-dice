.PHONY: all build check test clippy run clean

# Default target builds the release binary
all: build

# Compile the release binary
build:
	cargo build --release

# Run all unit, integration, and CLI tests
test:
	cargo test --all-targets

# Check code for compilation and lint warnings
check:
	cargo check
	cargo clippy --all-targets -- -D warnings

# Run the eff-dice binary with default settings
run:
	cargo run --release

# Clean build artifacts
clean:
	cargo clean

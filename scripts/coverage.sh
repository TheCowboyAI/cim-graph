#!/bin/bash
set -e

echo "Running code coverage analysis..."

# Clean previous coverage data
rm -rf target/coverage
rm -f *.profraw

# Build with coverage instrumentation
RUSTFLAGS='-C instrument-coverage' cargo build

# Run tests with coverage (only lib tests that compile)
RUSTFLAGS='-C instrument-coverage' LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw' cargo test --lib

# Check if grcov is installed
if ! command -v grcov &> /dev/null; then
    echo "grcov not found. Installing..."
    cargo install grcov
fi

# Generate coverage report
grcov . \
  --binary-path ./target/debug/deps/ \
  -s . \
  -t html \
  --branch \
  --ignore-not-existing \
  --ignore '../*' \
  --ignore "/*" \
  --ignore "tests/*" \
  --ignore "examples/*" \
  --ignore "benches/*" \
  -o target/coverage/

# Also generate lcov for CI
grcov . \
  --binary-path ./target/debug/deps/ \
  -s . \
  -t lcov \
  --branch \
  --ignore-not-existing \
  --ignore '../*' \
  --ignore "/*" \
  --ignore "tests/*" \
  --ignore "examples/*" \
  --ignore "benches/*" \
  -o coverage.lcov

echo "Coverage report generated at: target/coverage/index.html"

# Clean up profraw files
rm -f *.profraw
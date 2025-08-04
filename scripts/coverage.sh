#!/bin/bash
set -e

# Coverage script for CIM Graph

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}CIM Graph Coverage Report${NC}"
echo "========================="

# Clean previous coverage data
echo -e "\n${YELLOW}Cleaning previous coverage data...${NC}"
rm -f *.profraw *.profdata
cargo clean

# Install grcov if not available
if ! command -v grcov &> /dev/null; then
    echo -e "${YELLOW}Installing grcov...${NC}"
    cargo install grcov
fi

# Set coverage environment variables
export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Cinstrument-coverage"
export LLVM_PROFILE_FILE="cim-graph-%p-%m.profraw"

# Run tests with coverage
echo -e "\n${YELLOW}Running tests with coverage...${NC}"
cargo test --all-features

# Generate coverage data
echo -e "\n${YELLOW}Generating coverage report...${NC}"
grcov . \
    --binary-path ./target/debug/deps/ \
    -s . \
    -t html \
    --branch \
    --ignore-not-existing \
    --ignore "tests/*" \
    --ignore "benches/*" \
    --ignore "examples/*" \
    --ignore "*/build.rs" \
    -o ./target/coverage/

# Also generate lcov for CI integration
grcov . \
    --binary-path ./target/debug/deps/ \
    -s . \
    -t lcov \
    --branch \
    --ignore-not-existing \
    --ignore "tests/*" \
    --ignore "benches/*" \
    --ignore "examples/*" \
    --ignore "*/build.rs" \
    -o ./target/coverage.lcov

# Generate summary
echo -e "\n${YELLOW}Coverage Summary:${NC}"
grcov . \
    --binary-path ./target/debug/deps/ \
    -s . \
    -t coveralls \
    --branch \
    --ignore-not-existing \
    --ignore "tests/*" \
    --ignore "benches/*" \
    --ignore "examples/*" \
    --ignore "*/build.rs" \
    | python3 -c "
import sys, json
data = json.load(sys.stdin)
total_lines = sum(len(f['lines']) for f in data['source_files'])
covered_lines = sum(sum(1 for l in f['lines'] if l['count'] > 0) for f in data['source_files'])
coverage = (covered_lines / total_lines * 100) if total_lines > 0 else 0
print(f'Total lines: {total_lines}')
print(f'Covered lines: {covered_lines}')
print(f'Coverage: {coverage:.2f}%')

if coverage >= 90:
    print('\033[0;32m✓ Coverage target (90%) achieved!\033[0m')
elif coverage >= 80:
    print('\033[1;33m⚠ Coverage is good but below 90% target\033[0m')
else:
    print('\033[0;31m✗ Coverage is below 80%\033[0m')
"

# Clean up profraw files
rm -f *.profraw

echo -e "\n${GREEN}Coverage report generated at: target/coverage/index.html${NC}"
echo "Open in browser: firefox target/coverage/index.html"
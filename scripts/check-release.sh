#!/bin/bash
set -e

# Pre-release checks for CIM Graph

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}CIM Graph Pre-Release Checks${NC}"
echo "============================="

ERRORS=0

# Function to check command
check_command() {
    if ! command -v $1 &> /dev/null; then
        echo -e "${RED}✗ $1 not found${NC}"
        ((ERRORS++))
    else
        echo -e "${GREEN}✓ $1 available${NC}"
    fi
}

# Check required tools
echo -e "\n${YELLOW}Checking required tools:${NC}"
check_command cargo
check_command git
check_command rustfmt
check_command clippy-driver

# Check Rust version
echo -e "\n${YELLOW}Checking Rust version:${NC}"
RUST_VERSION=$(rustc --version | cut -d' ' -f2)
echo "Rust version: $RUST_VERSION"

# Check git status
echo -e "\n${YELLOW}Checking git status:${NC}"
if git diff-index --quiet HEAD --; then
    echo -e "${GREEN}✓ No uncommitted changes${NC}"
else
    echo -e "${RED}✗ Uncommitted changes detected${NC}"
    ((ERRORS++))
fi

# Check branch
BRANCH=$(git branch --show-current)
if [ "$BRANCH" == "main" ]; then
    echo -e "${GREEN}✓ On main branch${NC}"
else
    echo -e "${YELLOW}⚠ Not on main branch (current: $BRANCH)${NC}"
fi

# Run tests
echo -e "\n${YELLOW}Running tests:${NC}"
if cargo test --all-features --quiet; then
    echo -e "${GREEN}✓ All tests pass${NC}"
else
    echo -e "${RED}✗ Tests failed${NC}"
    ((ERRORS++))
fi

# Check formatting
echo -e "\n${YELLOW}Checking code formatting:${NC}"
if cargo fmt -- --check; then
    echo -e "${GREEN}✓ Code properly formatted${NC}"
else
    echo -e "${RED}✗ Code needs formatting${NC}"
    echo "  Run: cargo fmt"
    ((ERRORS++))
fi

# Run clippy
echo -e "\n${YELLOW}Running clippy:${NC}"
if cargo clippy -- -D warnings 2>/dev/null; then
    echo -e "${GREEN}✓ No clippy warnings${NC}"
else
    echo -e "${RED}✗ Clippy warnings found${NC}"
    ((ERRORS++))
fi

# Check documentation
echo -e "\n${YELLOW}Building documentation:${NC}"
if cargo doc --no-deps --quiet; then
    echo -e "${GREEN}✓ Documentation builds${NC}"
else
    echo -e "${RED}✗ Documentation build failed${NC}"
    ((ERRORS++))
fi

# Check Cargo.toml metadata
echo -e "\n${YELLOW}Checking Cargo.toml metadata:${NC}"
MISSING_FIELDS=0

check_field() {
    if grep -q "^$1 =" Cargo.toml; then
        echo -e "${GREEN}✓ $1 present${NC}"
    else
        echo -e "${RED}✗ $1 missing${NC}"
        ((MISSING_FIELDS++))
    fi
}

check_field "description"
check_field "repository"
check_field "license"
check_field "readme"
check_field "keywords"
check_field "categories"

if [ $MISSING_FIELDS -eq 0 ]; then
    ((ERRORS += MISSING_FIELDS))
fi

# Check for required files
echo -e "\n${YELLOW}Checking required files:${NC}"
REQUIRED_FILES=(
    "README.md"
    "LICENSE"
    "CHANGELOG.md"
    "CONTRIBUTING.md"
    ".gitignore"
)

for file in "${REQUIRED_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo -e "${GREEN}✓ $file exists${NC}"
    else
        echo -e "${RED}✗ $file missing${NC}"
        ((ERRORS++))
    fi
done

# Check examples
echo -e "\n${YELLOW}Checking examples:${NC}"
if [ -d "examples" ]; then
    echo -e "${GREEN}✓ Examples directory exists${NC}"
    EXAMPLE_COUNT=$(ls examples/*.rs 2>/dev/null | wc -l)
    echo "  Found $EXAMPLE_COUNT examples"
else
    echo -e "${YELLOW}⚠ No examples directory${NC}"
fi

# Summary
echo -e "\n${YELLOW}Summary:${NC}"
echo "========"
if [ $ERRORS -eq 0 ]; then
    echo -e "${GREEN}✓ All checks passed!${NC}"
    echo -e "Ready for release 🚀"
    exit 0
else
    echo -e "${RED}✗ Found $ERRORS issues${NC}"
    echo -e "Please fix these issues before releasing"
    exit 1
fi
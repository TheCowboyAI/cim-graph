#!/bin/bash
set -e

# Release script for CIM Graph

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}CIM Graph Release Script${NC}"
echo "========================="

# Check if version argument provided
if [ -z "$1" ]; then
    echo -e "${RED}Error: Version number required${NC}"
    echo "Usage: ./scripts/release.sh <version>"
    echo "Example: ./scripts/release.sh 0.1.0"
    exit 1
fi

VERSION=$1
echo -e "Preparing release ${YELLOW}v$VERSION${NC}"

# Check if on main branch
BRANCH=$(git branch --show-current)
if [ "$BRANCH" != "main" ]; then
    echo -e "${RED}Error: Must be on main branch to release${NC}"
    echo "Current branch: $BRANCH"
    exit 1
fi

# Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
    echo -e "${RED}Error: Uncommitted changes detected${NC}"
    echo "Please commit or stash your changes before releasing"
    exit 1
fi

# Update version in Cargo.toml
echo -e "\n${YELLOW}Updating version in Cargo.toml...${NC}"
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml

# Update version in VERSION file
echo "$VERSION" > VERSION

# Update CHANGELOG.md
echo -e "${YELLOW}Updating CHANGELOG.md...${NC}"
DATE=$(date +%Y-%m-%d)
sed -i "s/## \[Unreleased\]/## [Unreleased]\n\n## [$VERSION] - $DATE/" CHANGELOG.md

# Run tests
echo -e "\n${YELLOW}Running tests...${NC}"
cargo test --all-features

# Run clippy
echo -e "\n${YELLOW}Running clippy...${NC}"
cargo clippy -- -D warnings

# Build release
echo -e "\n${YELLOW}Building release...${NC}"
cargo build --release

# Generate documentation
echo -e "\n${YELLOW}Generating documentation...${NC}"
cargo doc --no-deps

# Commit version bump
echo -e "\n${YELLOW}Committing version bump...${NC}"
git add Cargo.toml VERSION CHANGELOG.md
git commit -m "Release v$VERSION"

# Create tag
echo -e "\n${YELLOW}Creating tag v$VERSION...${NC}"
git tag -a "v$VERSION" -m "Release version $VERSION"

# Push changes
echo -e "\n${YELLOW}Pushing changes and tag...${NC}"
git push origin main
git push origin "v$VERSION"

echo -e "\n${GREEN}Release v$VERSION prepared successfully!${NC}"
echo -e "The CI/CD pipeline will now:"
echo -e "  - Create a GitHub release"
echo -e "  - Publish to crates.io"
echo -e "  - Build release binaries"
echo -e "\nMonitor the progress at: https://github.com/thecowboyai/cim-graph/actions"
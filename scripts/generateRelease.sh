#!/bin/bash
set -e

# Check if git-cliff is installed
if ! command -v git-cliff &> /dev/null; then
    echo "Error: git-cliff is not installed"
    echo "Install it with: cargo install git-cliff"
    exit 1
fi

# Check if we're on trunk branch
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "trunk" ]; then
    echo "Error: You must be on the trunk branch to create a release"
    echo "Current branch: $CURRENT_BRANCH"
    exit 1
fi

# Check if working directory is clean
if ! git diff-index --quiet HEAD --; then
    echo "Error: Working directory is not clean"
    echo "Please commit or stash your changes before creating a release"
    exit 1
fi

# Get the next version from git-cliff
echo "Calculating next version..."
VERSION=$(git-cliff --bumped-version)

if [ -z "$VERSION" ]; then
    echo "Error: Could not determine next version"
    exit 1
fi

echo "Next version: $VERSION"

# Update the changelog
echo "Updating CHANGELOG.md..."
git-cliff --tag "$VERSION" -o CHANGELOG.md

# Stage the changelog
git add CHANGELOG.md

# Commit the changelog if there are changes
if ! git diff-index --quiet HEAD --; then
    git commit -m "chore(release): prepare for $VERSION"
fi

# Create a signed tag
echo "Creating signed tag v$VERSION..."
git tag -s "v$VERSION" -m "Release v$VERSION"

echo ""
echo "Release v$VERSION created successfully!"
echo ""
echo "To push the release, run:"
echo "  git push origin trunk"
echo "  git push origin v$VERSION"

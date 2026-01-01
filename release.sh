#!/bin/bash
# Release script for pack-league
# Usage: ./release.sh [patch|minor|major|X.Y.Z]
# Default: patch

set -e

# Get current version from config.json
CURRENT=$(grep '"version"' config.json | head -1 | sed 's/.*"version": *"\([^"]*\)".*/\1/')
echo "Current version: $CURRENT"

# Parse version components
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT"

# Determine new version
case "${1:-patch}" in
  patch)
    PATCH=$((PATCH + 1))
    NEW_VERSION="$MAJOR.$MINOR.$PATCH"
    ;;
  minor)
    MINOR=$((MINOR + 1))
    PATCH=0
    NEW_VERSION="$MAJOR.$MINOR.$PATCH"
    ;;
  major)
    MAJOR=$((MAJOR + 1))
    MINOR=0
    PATCH=0
    NEW_VERSION="$MAJOR.$MINOR.$PATCH"
    ;;
  *)
    # Explicit version provided
    NEW_VERSION="$1"
    ;;
esac

echo "New version: $NEW_VERSION"

# Update config.json
if [[ "$OSTYPE" == "darwin"* ]]; then
  sed -i '' "s/\"version\": \"$CURRENT\"/\"version\": \"$NEW_VERSION\"/" config.json
else
  sed -i "s/\"version\": \"$CURRENT\"/\"version\": \"$NEW_VERSION\"/" config.json
fi

# Commit, tag, and push
git add config.json
git commit -m "chore: release v$NEW_VERSION"
git tag "v$NEW_VERSION"
git push && git push --tags

echo ""
echo "✅ Released v$NEW_VERSION"
echo ""
echo "Next steps:"
echo "  1. Wait for GitHub Actions to build (~3-5 min)"
echo "  2. Update packs-index:"
echo "     cd ~/Projects/packs-index"
echo "     # Edit index.json: \"version\": \"$NEW_VERSION\""
echo "     git add -A && git commit -m \"chore: update league pack to v$NEW_VERSION\" && git push"

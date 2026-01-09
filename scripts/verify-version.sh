#!/usr/bin/env bash
set -euo pipefail

# Verify Cargo.toml version matches the git tag
# Usage: verify-version.sh <tag>

TAG="$1"
TAG_VERSION="${TAG#v}"

CARGO_VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')

if [ "$CARGO_VERSION" != "$TAG_VERSION" ]; then
    echo "Version mismatch: Cargo.toml has $CARGO_VERSION but tag is $TAG_VERSION" >&2
    exit 1
fi

echo "Version verified: $CARGO_VERSION"

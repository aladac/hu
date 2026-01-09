#!/usr/bin/env bash
set -euo pipefail

# Check if a version tag is a prerelease
# Usage: is-prerelease.sh <tag>
# Exits 0 (true) if prerelease, 1 (false) otherwise

TAG="$1"

if [[ "$TAG" == *"pre"* ]] || [[ "$TAG" == *"alpha"* ]] || [[ "$TAG" == *"beta"* ]] || [[ "$TAG" == *"rc"* ]]; then
    echo "true"
else
    echo "false"
fi

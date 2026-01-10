#!/usr/bin/env bash
set -euo pipefail

# Install cross-compilation dependencies
# Usage: install-cross-deps.sh <target>

TARGET="$1"

case "$TARGET" in
    aarch64-unknown-linux-gnu)
        sudo apt-get update
        sudo apt-get install -y gcc-aarch64-linux-gnu
        ;;
    *)
        # No extra dependencies needed
        ;;
esac

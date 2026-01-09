#!/usr/bin/env bash
set -euo pipefail

# Package a release binary
# Usage: package.sh <target> <version>
# Example: package.sh x86_64-unknown-linux-gnu 0.1.0

TARGET="$1"
VERSION="$2"

case "$TARGET" in
    *-linux-*)
        BINARY="target/${TARGET}/release/hu"
        case "$TARGET" in
            x86_64-*)  OUTPUT="hu-${VERSION}-linux-x86_64.gz" ;;
            aarch64-*) OUTPUT="hu-${VERSION}-linux-arm64.gz" ;;
        esac
        gzip -c "$BINARY" > "$OUTPUT"
        ;;
    *-apple-darwin)
        BINARY="target/${TARGET}/release/hu"
        case "$TARGET" in
            x86_64-*)  OUTPUT="hu-${VERSION}-mac-x86_64.gz" ;;
            aarch64-*) OUTPUT="hu-${VERSION}-mac-arm64.gz" ;;
        esac
        gzip -c "$BINARY" > "$OUTPUT"
        ;;
    *-windows-*)
        BINARY="target/${TARGET}/release/hu.exe"
        case "$TARGET" in
            x86_64-*)  OUTPUT="hu-${VERSION}-windows-x86_64.zip" ;;
            aarch64-*) OUTPUT="hu-${VERSION}-windows-arm64.zip" ;;
        esac
        7z a "$OUTPUT" "$BINARY"
        ;;
    *)
        echo "Unknown target: $TARGET" >&2
        exit 1
        ;;
esac

echo "$OUTPUT"

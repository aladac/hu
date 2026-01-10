#!/usr/bin/env bash
set -euo pipefail

# Install cross-compilation dependencies
# Usage: install-cross-deps.sh <target>

TARGET="$1"

case "$TARGET" in
    aarch64-unknown-linux-gnu)
        sudo dpkg --add-architecture arm64
        
        # Create a dedicated sources file for arm64 from ports.ubuntu.com
        CODENAME=$(lsb_release -cs)
        cat <<EOF | sudo tee /etc/apt/sources.list.d/arm64-ports.list
deb [arch=arm64] http://ports.ubuntu.com/ubuntu-ports ${CODENAME} main restricted universe multiverse
deb [arch=arm64] http://ports.ubuntu.com/ubuntu-ports ${CODENAME}-updates main restricted universe multiverse
deb [arch=arm64] http://ports.ubuntu.com/ubuntu-ports ${CODENAME}-security main restricted universe multiverse
EOF
        
        sudo apt-get update
        sudo apt-get install -y gcc-aarch64-linux-gnu libssl-dev:arm64 pkg-config
        ;;
    *)
        # No extra dependencies needed
        ;;
esac

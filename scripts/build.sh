#!/bin/bash
# Build the VWF project
# Usage: ./scripts/build.sh [--release]

set -e

cd "$(dirname "$0")/.."

RELEASE_FLAG=""
if [ "$1" = "--release" ]; then
    RELEASE_FLAG="--release"
    echo "Building in release mode..."
else
    echo "Building in debug mode..."
fi

# Build vwf-engine (core libraries)
echo "Building vwf-engine..."
(cd components/vwf-engine && cargo build $RELEASE_FLAG)

# Build vwf-apps (CLI and web)
echo "Building vwf-apps..."
(cd components/vwf-apps && cargo build $RELEASE_FLAG)

echo "Build complete!"

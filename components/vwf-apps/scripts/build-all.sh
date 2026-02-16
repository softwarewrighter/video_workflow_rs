#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPONENT_DIR="$(dirname "$SCRIPT_DIR")"
COMPONENTS_DIR="$(dirname "$COMPONENT_DIR")"

# Build dependencies first
"$COMPONENTS_DIR/vwf-engine/scripts/build-all.sh" "$@"

echo "=== Building vwf-apps ==="
cd "$COMPONENT_DIR"
cargo build "$@"

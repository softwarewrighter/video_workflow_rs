#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPONENT_DIR="$(dirname "$SCRIPT_DIR")"
COMPONENTS_DIR="$(dirname "$COMPONENT_DIR")"

# Test dependencies first
"$COMPONENTS_DIR/vwf-engine/scripts/test-all.sh" "$@"

echo "=== Testing vwf-apps ==="
cd "$COMPONENT_DIR"
cargo test "$@"

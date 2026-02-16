#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPONENT_DIR="$(dirname "$SCRIPT_DIR")"

echo "=== Testing vwf-foundation ==="
cd "$COMPONENT_DIR"
cargo test "$@"

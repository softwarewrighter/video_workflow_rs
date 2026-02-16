#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

echo "=== Building all components ==="

# Build in dependency order
"$ROOT_DIR/components/vwf-foundation/scripts/build-all.sh" "$@"
"$ROOT_DIR/components/vwf-engine/scripts/build-all.sh" "$@"
"$ROOT_DIR/components/vwf-apps/scripts/build-all.sh" "$@"

echo "=== All components built successfully ==="

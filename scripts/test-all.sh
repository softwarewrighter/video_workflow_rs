#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

echo "=== Testing all components ==="

# Test in dependency order
"$ROOT_DIR/components/vwf-foundation/scripts/test-all.sh" "$@"
"$ROOT_DIR/components/vwf-engine/scripts/test-all.sh" "$@"
"$ROOT_DIR/components/vwf-apps/scripts/test-all.sh" "$@"

echo "=== All tests passed ==="

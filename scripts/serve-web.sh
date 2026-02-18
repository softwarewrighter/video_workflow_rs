#!/bin/bash
# Serve the VWF web UI
# Usage: ./scripts/serve-web.sh [port]

set -e

cd "$(dirname "$0")/.."

PORT="${1:-8090}"

echo "Building and serving VWF Web UI..."
echo "Open http://localhost:$PORT in your browser"
echo ""

(cd components/vwf-apps/crates/vwf-web && trunk serve --port "$PORT")

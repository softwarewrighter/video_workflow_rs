#!/bin/bash
# Show/debug workflow parsing
# Usage: ./scripts/show.sh <workflow.yaml>

set -e

cd "$(dirname "$0")/.."

if [ -z "$1" ]; then
    echo "Usage: ./scripts/show.sh <workflow.yaml>"
    exit 1
fi

WORKFLOW="$1"

echo "Parsing workflow: $WORKFLOW"
echo ""

(cd components/vwf-apps && cargo run -p vwf-cli --quiet -- show "../../$WORKFLOW")

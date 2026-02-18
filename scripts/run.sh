#!/bin/bash
# Run a workflow
# Usage: ./scripts/run.sh <workflow.yaml> [options]
#
# Examples:
#   ./scripts/run.sh projects/self/workflow.yaml --resume
#   ./scripts/run.sh projects/demo/workflow.yaml --dry-run
#   ./scripts/run.sh projects/self/workflow.yaml --resume --allow ffmpeg --allow bash

set -e

cd "$(dirname "$0")/.."

if [ -z "$1" ]; then
    echo "Usage: ./scripts/run.sh <workflow.yaml> [options]"
    echo ""
    echo "Options:"
    echo "  --resume       Skip steps with existing output"
    echo "  --dry-run      Preview without executing"
    echo "  --allow <cmd>  Allow specific shell commands"
    echo "  --var k=v      Override workflow variable"
    echo ""
    echo "Examples:"
    echo "  ./scripts/run.sh projects/self/workflow.yaml --resume"
    echo "  ./scripts/run.sh projects/demo/workflow.yaml --dry-run"
    exit 1
fi

WORKFLOW="$1"
shift

# Extract workdir from workflow path (parent directory)
WORKDIR=$(dirname "$WORKFLOW")

echo "Running workflow: $WORKFLOW"
echo "Working directory: $WORKDIR"
echo ""

(cd components/vwf-apps && cargo run -p vwf-cli --quiet -- run "../../$WORKFLOW" --workdir "../../$WORKDIR" "$@")

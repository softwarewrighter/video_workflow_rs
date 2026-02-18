#!/bin/bash
# Format all Rust code
# Usage: ./scripts/fmt.sh [--check]

set -e

cd "$(dirname "$0")/.."

CHECK_FLAG=""
if [ "$1" = "--check" ]; then
    CHECK_FLAG="--check"
    echo "Checking format..."
else
    echo "Formatting code..."
fi

(cd components/vwf-engine && cargo fmt --all $CHECK_FLAG)
(cd components/vwf-apps && cargo fmt --all $CHECK_FLAG)

echo "Format complete!"

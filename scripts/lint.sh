#!/bin/bash
# Run clippy linting (zero warnings required)
# Usage: ./scripts/lint.sh

set -e

cd "$(dirname "$0")/.."

echo "Running clippy on vwf-engine..."
(cd components/vwf-engine && cargo clippy --all-targets --all-features -- -D warnings)

echo "Running clippy on vwf-apps..."
(cd components/vwf-apps && cargo clippy --all-targets --all-features -- -D warnings)

echo "Lint passed!"

#!/bin/bash
# Run all tests
# Usage: ./scripts/test.sh [crate-name]

set -e

cd "$(dirname "$0")/.."

if [ -n "$1" ]; then
    echo "Testing crate: $1"
    (cd components/vwf-engine && cargo test -p "$1")
else
    echo "Running all tests..."

    # Test vwf-engine crates
    echo "Testing vwf-engine..."
    (cd components/vwf-engine && cargo test)

    # Test vwf-apps crates
    echo "Testing vwf-apps..."
    (cd components/vwf-apps && cargo test)

    # Run Python hygiene checks
    echo "Running Python hygiene checks..."
    ./scripts/check-python-hygiene.sh
fi

echo "All tests passed!"

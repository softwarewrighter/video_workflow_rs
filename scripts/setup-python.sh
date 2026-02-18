#!/bin/bash
# Set up Python virtual environment with required packages
# Usage: ./scripts/setup-python.sh

set -e

cd "$(dirname "$0")/.."

echo "Setting up Python environment..."

# Create venv if it doesn't exist
if [ ! -d ".venv" ]; then
    echo "Creating virtual environment..."
    uv venv
fi

# Activate and install dependencies
echo "Installing Python dependencies..."
source .venv/bin/activate
uv pip install -r requirements.txt

# Verify installation
echo "Verifying installation..."
python -c "import requests; from gradio_client import Client; print('All packages installed successfully!')"

echo ""
echo "Python environment ready!"
echo "Venv Python path: $(pwd)/.venv/bin/python"
echo ""
echo "Add this to your workflow.yaml vars:"
echo "  python_path: \"$(pwd)/.venv/bin/python\""

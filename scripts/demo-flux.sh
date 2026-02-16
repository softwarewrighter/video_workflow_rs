#!/bin/bash
# Demo: Generate sample images with FLUX in all orientations
# Tests portrait (9:16), landscape (16:9), and square formats
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="$REPO_DIR/test-projects/flux-demo"

mkdir -p "$OUTPUT_DIR"

# Ensure venv with requests
if [[ ! -f "$REPO_DIR/.venv/bin/activate" ]]; then
    echo "Creating Python venv..."
    uv venv "$REPO_DIR/.venv"
fi
source "$REPO_DIR/.venv/bin/activate"
if ! python -c "import requests" 2>/dev/null; then
    echo "Installing requests..."
    uv pip install requests
fi

echo "=============================================="
echo "FLUX Image Generation Demo"
echo "=============================================="
echo
echo "Output: $OUTPUT_DIR"
echo

# Portrait (9:16) - for YouTube Shorts backgrounds
echo "1/3: Generating portrait image (720x1280 → scales 1.5x to 1080x1920)..."
python "$SCRIPT_DIR/flux_client.py" \
    --prompt "Abstract dark blue gradient with subtle grid lines, futuristic tech aesthetic, vertical composition, minimalist" \
    --orientation portrait \
    --output "$OUTPUT_DIR/bg-portrait.png"
echo

# Landscape (16:9) - for explainer video backgrounds
echo "2/3: Generating landscape image (1280x720 → scales 1.5x to 1920x1080)..."
python "$SCRIPT_DIR/flux_client.py" \
    --prompt "Modern software development workspace, multiple monitors with code, dark theme, dramatic lighting, cinematic" \
    --orientation landscape \
    --output "$OUTPUT_DIR/bg-landscape.png"
echo

# Square - for thumbnails
echo "3/3: Generating square image (1024x1024)..."
python "$SCRIPT_DIR/flux_client.py" \
    --prompt "VWF logo concept, stylized letters V W F, dark background, cyan accent glow, professional, clean" \
    --orientation square \
    --output "$OUTPUT_DIR/bg-square.png"
echo

echo "=============================================="
echo "Demo complete!"
echo "=============================================="
echo
echo "Generated images:"
for f in "$OUTPUT_DIR"/bg-*.png; do
    dims=$(file "$f" | grep -oE '[0-9]+ x [0-9]+')
    echo "  $(basename "$f"): $dims"
done
echo
echo "Opening images..."
open "$OUTPUT_DIR"/bg-*.png

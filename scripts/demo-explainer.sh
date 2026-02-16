#!/bin/bash
# Demo: Generate a sample explainer (longer format)
# This script runs the complete video generation pipeline
# Currently uses the same sample-video project; future versions will support longer content
set -e

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROJECT_DIR="$REPO_DIR/test-projects/sample-video"

echo "============================================"
echo "VWF Demo: Generate Sample Explainer"
echo "============================================"
echo
echo "Project: What is a Workflow?"
echo "Format: Explainer video with 5 sections"
echo "Location: $PROJECT_DIR"
echo

# Step 1: Generate scripts via workflow
echo "Step 1/4: Generating narration scripts..."
cd "$PROJECT_DIR"
cargo run -p vwf-cli --release -- run workflow.yaml --workdir . --resume
echo

# Step 2: Generate TTS audio
echo "Step 2/4: Generating voice-cloned TTS audio..."
echo "  Server: http://curiosity:7860 (VoxCPM)"
if [[ ! -f "$PROJECT_DIR/.venv/bin/activate" ]]; then
    echo "  Creating Python venv..."
    uv venv "$PROJECT_DIR/.venv"
fi
source "$PROJECT_DIR/.venv/bin/activate"
if ! python -c "import gradio_client" 2>/dev/null; then
    echo "  Installing gradio_client..."
    uv pip install gradio_client
fi
"$PROJECT_DIR/scripts/generate-tts.sh"
echo

# Step 3: Build video
echo "Step 3/4: Building video..."
echo "  - Generating slide images (vid-slide)"
echo "  - Combining slides + audio (vid-image)"
echo "  - Concatenating clips (vid-concat)"
"$PROJECT_DIR/scripts/build-video.sh"
echo

# Step 4: Open preview
echo "Step 4/4: Opening preview..."
PREVIEW="$PROJECT_DIR/output/preview.mp4"
if [[ -f "$PREVIEW" ]]; then
    dur=$(ffprobe -i "$PREVIEW" -show_entries format=duration -v quiet -of csv="p=0")
    printf "Preview: %s (%.1f seconds)\n" "$PREVIEW" "$dur"
    open "$PREVIEW"
else
    echo "ERROR: Preview not found at $PREVIEW"
    exit 1
fi

echo
echo "============================================"
echo "Explainer demo complete!"
echo "============================================"
echo
echo "Generated artifacts:"
echo "  Scripts: $PROJECT_DIR/work/scripts/*.txt"
echo "  Audio:   $PROJECT_DIR/work/audio/*.wav"
echo "  Slides:  $PROJECT_DIR/work/slides/*.png"
echo "  Clips:   $PROJECT_DIR/work/clips/*.mp4"
echo "  Preview: $PREVIEW"
echo
echo "To verify TTS quality: $PROJECT_DIR/scripts/verify-tts.sh"

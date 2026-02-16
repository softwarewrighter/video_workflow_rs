#!/bin/bash
# Demo: Generate a sample short (~32 sec)
# This script runs the complete video generation pipeline
set -e

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROJECT_DIR="$REPO_DIR/test-projects/sample-video"

echo "=========================================="
echo "VWF Demo: Generate Sample Short (~32 sec)"
echo "=========================================="
echo
echo "Project: What is a Workflow?"
echo "Location: $PROJECT_DIR"
echo

# Step 1: Generate scripts via workflow
echo "Step 1/4: Generating narration scripts..."
cargo run -p vwf-cli --manifest-path "$REPO_DIR/components/vwf-apps/Cargo.toml" --release -- run "$PROJECT_DIR/workflow.yaml" --workdir "$PROJECT_DIR" --resume
echo

# Step 2: Generate TTS audio
echo "Step 2/4: Generating voice-cloned TTS audio..."
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
echo "Step 3/4: Building video (slides + audio + concat)..."
"$PROJECT_DIR/scripts/build-video.sh"
echo

# Step 4: Rename and open preview
echo "Step 4/4: Opening preview..."
SRC="$PROJECT_DIR/output/preview.mp4"
PREVIEW="$PROJECT_DIR/output/preview-sample-short.mp4"
if [[ -f "$SRC" ]]; then
    mv "$SRC" "$PREVIEW"
    dur=$(ffprobe -i "$PREVIEW" -show_entries format=duration -v quiet -of csv="p=0")
    printf "Preview: %s (%.1f seconds)\n" "$PREVIEW" "$dur"
    open "$PREVIEW"
else
    echo "ERROR: Preview not found at $SRC"
    exit 1
fi

echo
echo "=========================================="
echo "Demo complete!"
echo "=========================================="

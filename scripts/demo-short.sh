#!/bin/bash
# Demo: Generate a YouTube Short (~30 sec, 1080x1920 vertical)
# This script runs the complete video generation pipeline for vertical shorts
set -e

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROJECT_DIR="$REPO_DIR/test-projects/sample-short"

echo "=============================================="
echo "VWF Demo: Generate YouTube Short"
echo "=============================================="
echo
echo "Project: VWF in 30 Seconds"
echo "Format: 1080x1920 vertical (9:16)"
echo "Target: < 60 seconds"
echo "Location: $PROJECT_DIR"
echo

# Step 1: Generate scripts via workflow
echo "Step 1/4: Generating narration scripts..."
cargo run -p vwf-cli --manifest-path "$REPO_DIR/components/vwf-apps/Cargo.toml" --release -- run "$PROJECT_DIR/workflow.yaml" --workdir "$PROJECT_DIR" --resume
echo

# Step 2: Generate TTS audio
echo "Step 2/4: Generating voice-cloned TTS audio..."
"$PROJECT_DIR/scripts/generate-tts.sh"
echo

# Step 3: Build video
echo "Step 3/4: Building vertical video (1080x1920)..."
"$PROJECT_DIR/scripts/build-video.sh"
echo

# Step 4: Rename and open preview
echo "Step 4/4: Opening preview..."
SRC="$PROJECT_DIR/output/preview.mp4"
PREVIEW="$PROJECT_DIR/output/preview-sample-short.mp4"
if [[ -f "$SRC" ]]; then
    mv "$SRC" "$PREVIEW"
    dur=$(ffprobe -i "$PREVIEW" -show_entries format=duration -v quiet -of csv="p=0")
    dims=$(ffprobe -i "$PREVIEW" -v error -select_streams v:0 -show_entries stream=width,height -of csv=p=0)
    printf "Preview: %s\n" "$PREVIEW"
    printf "Duration: %.1f seconds\n" "$dur"
    echo "Dimensions: $dims (vertical)"
    open "$PREVIEW"
else
    echo "ERROR: Preview not found at $SRC"
    exit 1
fi

echo
echo "=============================================="
echo "YouTube Short demo complete!"
echo "=============================================="

#!/bin/bash
# Generate TTS audio for tt-rs explainer narration
# Uses VoxCPM for voice cloning
set -e

source "$(dirname "$0")/common.sh"

cd "$PROJECT_DIR"

mkdir -p "$AUDIO_DIR"

echo "=== Generating TTS Audio ==="
echo "Server: $TTS_SERVER"
echo ""

# Check if TTS server is available
if ! curl -s --connect-timeout 5 "$TTS_SERVER" > /dev/null 2>&1; then
    echo "WARNING: TTS server not available at $TTS_SERVER"
    echo "Skipping TTS generation. Run this script when server is up."
    exit 1
fi

# Generate audio for each narration section
for name in 01-hook 02-intro-demo 03-cta; do
    script_file="$SCRIPTS_DIR/${name}.txt"
    audio_file="$AUDIO_DIR/${name}.wav"

    if [ -f "$audio_file" ]; then
        echo "  Skipping $name (already exists)"
        continue
    fi

    echo "  Generating: $name..."

    # VoxCPM voice clone via Gradio API
    python3 - "$script_file" "$audio_file" << 'PYTHON'
import sys
import json
import requests

script_path = sys.argv[1]
output_path = sys.argv[2]

with open(script_path, 'r') as f:
    text = f.read().strip()

# VoxCPM Gradio API endpoint
server = "http://curiosity:7860"
ref_wav = "/Users/mike/github/softwarewrighter/video-publishing/reference/voice/mike-medium-ref-1.wav"
ref_text = "In this session, I'm going to write a small command line tool and explain the decision making process as I go."

payload = {
    "data": [ref_wav, ref_text, text],
    "fn_index": 0
}

response = requests.post(f"{server}/run/predict", json=payload)
result = response.json()

# Download the generated audio
audio_url = result["data"][0]["name"]
audio_response = requests.get(f"{server}/file={audio_url}")

with open(output_path, 'wb') as f:
    f.write(audio_response.content)

print(f"  Created: {output_path}")
PYTHON

done

echo ""
echo "=== TTS Generation Complete ==="
echo "Audio files in: $AUDIO_DIR/"
ls -la "$AUDIO_DIR/"

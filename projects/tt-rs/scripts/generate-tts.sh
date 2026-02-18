#!/bin/bash
# Generate TTS audio for all script sections using VoxCPM voice cloning
set -e

source "$(dirname "$0")/common.sh"

cd "$PROJECT_DIR"

# Activate venv with gradio_client
source .venv/bin/activate

mkdir -p "$AUDIO_DIR"

echo "Generating TTS audio..."
for script in "$SCRIPTS_DIR"/*.txt; do
    name=$(basename "$script" .txt)
    output="$AUDIO_DIR/${name}.wav"

    if [[ -f "$output" ]]; then
        echo "  $name: exists, skipping"
        continue
    fi

    echo "  $name: generating..."
    python tts_client.py \
        --server "$TTS_SERVER" \
        --reference "$REF_WAV" \
        --prompt-text "$REF_TEXT" \
        --text "$(cat "$script")" \
        --output "$output"
done

echo "Done. Audio files in $AUDIO_DIR"

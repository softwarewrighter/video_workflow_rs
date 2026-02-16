#!/bin/bash
# Generate TTS audio for all script sections using VoxCPM voice cloning
set -e

source "$(dirname "$0")/common.sh"

cd "$PROJECT_DIR"

# Activate venv with gradio_client
if [[ ! -f ".venv/bin/activate" ]]; then
    echo "Creating Python venv..."
    uv venv .venv
fi
source .venv/bin/activate
if ! python -c "import gradio_client" 2>/dev/null; then
    echo "Installing gradio_client..."
    uv pip install gradio_client
fi

mkdir -p "$AUDIO_DIR"

# Copy tts_client.py from sample-video if needed
if [[ ! -f "tts_client.py" ]]; then
    cp ../sample-video/tts_client.py .
fi

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

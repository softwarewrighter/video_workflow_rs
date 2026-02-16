#!/bin/bash
# Verify TTS output using whisper transcription
set -e

source "$(dirname "$0")/common.sh"

cd "$PROJECT_DIR"

echo "Verifying TTS audio with Whisper..."
echo

for audio in "$AUDIO_DIR"/*.wav; do
    name=$(basename "$audio" .wav)
    script="$SCRIPTS_DIR/${name}.txt"

    echo "=== $name ==="
    echo "SCRIPT: $(cat "$script")"
    echo "WHISPER:"

    # Convert to 16kHz mono for whisper
    ffmpeg -y -i "$audio" -ar 16000 -ac 1 -c:a pcm_s16le /tmp/whisper_temp.wav 2>/dev/null
    /opt/homebrew/bin/whisper-cli -m "$WHISPER_MODEL" -f /tmp/whisper_temp.wav -nt 2>/dev/null

    # Get duration
    dur=$(ffprobe -i "$audio" -show_entries format=duration -v quiet -of csv="p=0")
    printf "DURATION: %.1f sec\n" "$dur"
    echo
done

rm -f /tmp/whisper_temp.wav

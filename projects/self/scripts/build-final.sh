#!/bin/bash
# Final video assembly: concatenate normalized clips + mix background music
# Run AFTER normalize_volume steps have adjusted clip audio levels
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/common.sh"

cd "$PROJECT_DIR"

mkdir -p "$OUTPUT_DIR"

# Standard format
TARGET_AUDIO_RATE=44100
TARGET_AUDIO_CHANNELS=2

echo "=== Final Video Assembly ==="

# ============================================
# Step 1: Concatenate all normalized clips
# ============================================
echo "Step 1: Concatenating normalized clips..."

# Build concat list
cat > "$CLIPS_DIR/concat.txt" << EOF
file '00-hook.mp4'
file '01-problem.mp4'
file '02-solution.mp4'
file '03-project.mp4'
file '04-workflow.mp4'
file '05-steps.mp4'
file '06-running.mp4'
file '07-resume.mp4'
file '08-meta.mp4'
file '09-cta.mp4'
EOF

# Re-encode during concat to ensure format consistency
ffmpeg -y -f concat -safe 0 -i "$CLIPS_DIR/concat.txt" \
    -c:v libx264 -preset fast -crf 18 -pix_fmt yuv420p \
    -c:a aac -ar ${TARGET_AUDIO_RATE} -ac ${TARGET_AUDIO_CHANNELS} -b:a 192k \
    "$OUTPUT_DIR/preview.mp4" 2>/dev/null

echo "  Created: $OUTPUT_DIR/preview.mp4"

# ============================================
# Step 2: Mix in background music
# ============================================
echo "Step 2: Mixing background music..."

MUSIC_INTRO="$ASSETS_DIR/music-intro.wav"
MUSIC_OUTRO="$ASSETS_DIR/music-outro.wav"

if [ -f "$MUSIC_INTRO" ] && [ -f "$MUSIC_OUTRO" ]; then
    # Get video duration
    VIDEO_DUR=$(ffprobe -i "$OUTPUT_DIR/preview.mp4" -show_entries format=duration -v quiet -of csv="p=0")
    VIDEO_DUR_INT=${VIDEO_DUR%.*}

    # Music volume: -12dB (background level)
    # Intro music: fade out over 4 seconds starting at 10s
    # Outro music: start near end, fade in over 3 seconds
    OUTRO_START=$((VIDEO_DUR_INT - 18))

    ffmpeg -y -i "$OUTPUT_DIR/preview.mp4" \
        -i "$MUSIC_INTRO" \
        -i "$MUSIC_OUTRO" \
        -filter_complex "
            [1:a]volume=-12dB,afade=t=out:st=10:d=4[intro_music];
            [2:a]volume=-12dB,adelay=${OUTRO_START}000|${OUTRO_START}000,afade=t=in:st=${OUTRO_START}:d=3[outro_music];
            [0:a][intro_music][outro_music]amix=inputs=3:duration=first:normalize=0[aout]
        " \
        -map 0:v -map "[aout]" \
        -c:v copy -c:a aac -ar ${TARGET_AUDIO_RATE} -ac ${TARGET_AUDIO_CHANNELS} -b:a 192k \
        "$OUTPUT_DIR/final.mp4" 2>/dev/null

    echo "  Added intro music (fade out at 10s)"
    echo "  Added outro music (fade in at ${OUTRO_START}s)"
    FINAL_OUTPUT="$OUTPUT_DIR/final.mp4"
else
    echo "  Skipping music (files not found)"
    FINAL_OUTPUT="$OUTPUT_DIR/preview.mp4"
fi

echo ""
echo "=== Build Complete ==="
echo "Output: $FINAL_OUTPUT"

# Show duration and dimensions
dur=$(ffprobe -i "$FINAL_OUTPUT" -show_entries format=duration -v quiet -of csv="p=0")
dims=$(ffprobe -i "$FINAL_OUTPUT" -v error -select_streams v:0 -show_entries stream=width,height -of csv=p=0)
printf "Duration: %.1f seconds\n" "$dur"
echo "Dimensions: $dims (landscape 16:9)"

# Show audio levels to verify normalization worked
echo ""
echo "Final audio level:"
ffmpeg -i "$FINAL_OUTPUT" -af "volumedetect" -f null - 2>&1 | grep -E "(mean_volume|max_volume)"

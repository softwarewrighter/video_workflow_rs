#!/bin/bash
# Build the VWF self-explainer video (landscape 1920x1080)
# Structure:
#   00-intro (~4s) - Five things about VWF
#   01-workflows (~10s) - Workflows as data
#   02-dag (~10s) - Dependency graph
#   03-placeholders (~12s) - Placeholders
#   04-checkpoints (~13s) - Checkpoints
#   05-incremental (~12s) - Incremental builds
#   06-cta (~7s) - Call to action
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/common.sh"

cd "$PROJECT_DIR"

mkdir -p "$CLIPS_DIR" "$OUTPUT_DIR" "$IMAGES_DIR"

# Standard format for concatenation
TARGET_FPS=25
TARGET_AUDIO_RATE=44100
TARGET_AUDIO_CHANNELS=2

echo "=== Building VWF Self-Explainer Video ==="

# ============================================
# Step 1: Upscale Flux backgrounds to 1920x1080 and composite SVG overlays
# ============================================
echo "Step 1: Creating composite slides (Flux bg + SVG overlay)..."

create_composite_slide() {
    local name="$1"
    local bg="$IMAGES_DIR/${name}-bg.png"
    local svg="$SVG_DIR/${name}.svg"
    local output="$IMAGES_DIR/${name}-slide.png"

    if [ ! -f "$bg" ]; then
        echo "  WARNING: Background not found: $bg"
        # Fall back to solid color
        convert -size ${WIDTH}x${HEIGHT} xc:"$BG_COLOR" "$output"
        return
    fi

    # Upscale Flux image (1280x720) to 1920x1080
    local upscaled="$IMAGES_DIR/${name}-upscaled.png"
    ffmpeg -y -i "$bg" -vf "scale=${WIDTH}:${HEIGHT}:flags=lanczos" "$upscaled" 2>/dev/null

    if [ -f "$svg" ]; then
        # Render SVG to PNG with transparency
        local overlay="$IMAGES_DIR/${name}-overlay.png"
        rsvg-convert -w $WIDTH -h $HEIGHT "$svg" -o "$overlay"

        # Composite: background + SVG overlay
        ffmpeg -y -i "$upscaled" -i "$overlay" \
            -filter_complex "[0:v][1:v]overlay=0:0" \
            "$output" 2>/dev/null
        echo "  Created composite: $output"
    else
        # No SVG, just use upscaled background
        cp "$upscaled" "$output"
        echo "  Created (no overlay): $output"
    fi
}

for section in 00-intro 01-workflows 02-dag 03-placeholders 04-checkpoints 05-incremental 06-cta; do
    create_composite_slide "$section"
done

# ============================================
# Step 2: Create clips with audio
# ============================================
echo "Step 2: Creating video clips with audio..."

create_section_clip() {
    local name="$1"
    local slide="$IMAGES_DIR/${name}-slide.png"
    local audio="$AUDIO_DIR/${name}.wav"
    local output="$CLIPS_DIR/${name}.mp4"

    if [ ! -f "$slide" ]; then
        echo "  WARNING: Slide not found: $slide"
        return
    fi

    if [ -f "$audio" ]; then
        # Use TTS audio - duration matches audio length
        ffmpeg -y -loop 1 -i "$slide" \
            -i "$audio" \
            -c:v libx264 -pix_fmt yuv420p -c:a aac -ar ${TARGET_AUDIO_RATE} -ac ${TARGET_AUDIO_CHANNELS} -shortest \
            "$output" 2>/dev/null
        echo "  Created clip with audio: $output"
    else
        # Fallback to silent placeholder
        ffmpeg -y -loop 1 -i "$slide" \
            -f lavfi -i anullsrc=r=${TARGET_AUDIO_RATE}:cl=stereo \
            -t $PLACEHOLDER_DURATION \
            -c:v libx264 -pix_fmt yuv420p -c:a aac -ar ${TARGET_AUDIO_RATE} -ac ${TARGET_AUDIO_CHANNELS} -shortest \
            "$output" 2>/dev/null
        echo "  Created clip (silent): $output"
    fi
}

for section in 00-intro 01-workflows 02-dag 03-placeholders 04-checkpoints 05-incremental 06-cta; do
    create_section_clip "$section"
done

# ============================================
# Step 3: Concatenate all clips
# ============================================
echo "Step 3: Concatenating clips into preview..."

cat > "$CLIPS_DIR/concat.txt" << EOF
file '00-intro.mp4'
file '01-workflows.mp4'
file '02-dag.mp4'
file '03-placeholders.mp4'
file '04-checkpoints.mp4'
file '05-incremental.mp4'
file '06-cta.mp4'
EOF

# Re-encode during concat to ensure format consistency
ffmpeg -y -f concat -safe 0 -i "$CLIPS_DIR/concat.txt" \
    -c:v libx264 -preset fast -crf 18 -pix_fmt yuv420p \
    -c:a aac -ar ${TARGET_AUDIO_RATE} -ac ${TARGET_AUDIO_CHANNELS} -b:a 192k \
    "$OUTPUT_DIR/preview.mp4" 2>/dev/null

echo ""
echo "=== Build Complete ==="
echo "Output: $OUTPUT_DIR/preview.mp4"

# Show duration and dimensions
dur=$(ffprobe -i "$OUTPUT_DIR/preview.mp4" -show_entries format=duration -v quiet -of csv="p=0")
dims=$(ffprobe -i "$OUTPUT_DIR/preview.mp4" -v error -select_streams v:0 -show_entries stream=width,height -of csv=p=0)
printf "Duration: %.1f seconds\n" "$dur"
echo "Dimensions: $dims (landscape 16:9)"

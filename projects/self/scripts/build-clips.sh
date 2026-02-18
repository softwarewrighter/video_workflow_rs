#!/bin/bash
# Build individual video clips (slides + audio)
# Clips are NOT concatenated here - that's done in build-final.sh after normalization
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/common.sh"

cd "$PROJECT_DIR"

mkdir -p "$CLIPS_DIR" "$IMAGES_DIR"

# Slide sections (10 slides for detailed tutorial)
SECTIONS="00-hook 01-problem 02-solution 03-project 04-workflow 05-steps 06-running 07-resume 08-meta 09-cta"

echo "=== Building Video Clips (VWF Self-Explainer) ==="

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

for section in $SECTIONS; do
    create_composite_slide "$section"
done

# ============================================
# Step 2: Create clips with audio
# ============================================
echo "Step 2: Creating video clips with audio..."

# Standard format for concatenation
TARGET_FPS=25
TARGET_AUDIO_RATE=44100
TARGET_AUDIO_CHANNELS=2

create_section_clip() {
    local name="$1"
    local slide="$IMAGES_DIR/${name}-slide.png"
    local animated="$PROJECT_DIR/work/videos/${name}-animated.mp4"
    local audio="$AUDIO_DIR/${name}.wav"
    local output="$CLIPS_DIR/${name}.mp4"

    # Check for animated video background (SVD-generated)
    if [ -f "$animated" ] && [ -f "$audio" ]; then
        # Use animated video looped with audio overlay + SVG
        local svg="$SVG_DIR/${name}.svg"
        local overlay="$IMAGES_DIR/${name}-overlay.png"

        # Render SVG overlay
        if [ -f "$svg" ]; then
            rsvg-convert -w $WIDTH -h $HEIGHT "$svg" -o "$overlay" 2>/dev/null
        fi

        # Get audio duration
        local audio_dur=$(ffprobe -i "$audio" -show_entries format=duration -v quiet -of csv="p=0")

        # Scale animated video to 1920x1080, loop to audio duration, overlay SVG
        if [ -f "$overlay" ]; then
            ffmpeg -y -stream_loop -1 -i "$animated" -i "$overlay" -i "$audio" \
                -t "$audio_dur" \
                -filter_complex "[0:v]scale=${WIDTH}:${HEIGHT}:flags=lanczos[bg];[bg][1:v]overlay=0:0[v]" \
                -map "[v]" -map 2:a \
                -c:v libx264 -pix_fmt yuv420p -c:a aac -ar ${TARGET_AUDIO_RATE} -ac ${TARGET_AUDIO_CHANNELS} \
                "$output" 2>/dev/null
        else
            ffmpeg -y -stream_loop -1 -i "$animated" -i "$audio" \
                -t "$audio_dur" \
                -vf "scale=${WIDTH}:${HEIGHT}:flags=lanczos" \
                -c:v libx264 -pix_fmt yuv420p -c:a aac -ar ${TARGET_AUDIO_RATE} -ac ${TARGET_AUDIO_CHANNELS} \
                "$output" 2>/dev/null
        fi
        echo "  Created ANIMATED clip: $output"
        return
    fi

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

for section in $SECTIONS; do
    create_section_clip "$section"
done

echo ""
echo "=== Clips Created ==="
echo "Next: Run normalize_volume steps, then build-final.sh"
ls -la "$CLIPS_DIR"/*.mp4

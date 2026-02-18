#!/bin/bash
# Build the VWF self-explainer video (landscape 1920x1080)
# 10-slide detailed tutorial structure
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/common.sh"

cd "$PROJECT_DIR"

mkdir -p "$CLIPS_DIR" "$OUTPUT_DIR" "$IMAGES_DIR"

# Standard format for concatenation
TARGET_FPS=25
TARGET_AUDIO_RATE=44100
TARGET_AUDIO_CHANNELS=2

# Slide sections (10 slides for detailed tutorial)
SECTIONS="00-hook 01-problem 02-solution 03-project 04-workflow 05-steps 06-running 07-resume 08-meta 09-cta"

echo "=== Building VWF Self-Explainer Video (Detailed Tutorial) ==="

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

# ============================================
# Step 3: Concatenate all clips
# ============================================
echo "Step 3: Concatenating clips into preview..."

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

# ============================================
# Step 4: Mix in background music
# ============================================
echo "Step 4: Mixing background music..."

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

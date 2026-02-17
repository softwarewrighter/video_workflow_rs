#!/bin/bash
# Build the tt-rs explainer video (landscape 1920x1080)
# Structure:
#   00-title (5s) - robot video on white background with text + music
#   01-hook (~5s) - narration placeholder
#   02-intro-demo (~5s) - narration placeholder
#   03-demo (placeholder) - OBS capture to be inserted later
#   04-cta (~5s) - narration placeholder
#   99a-epilog (~12s) - standard epilog
#   99b-epilog-ext (7s) - epilog frame with music
set -e

source "$(dirname "$0")/common.sh"

cd "$PROJECT_DIR"

mkdir -p "$CLIPS_DIR" "$OUTPUT_DIR"

# Standard format for concatenation
TARGET_FPS=25
TARGET_AUDIO_RATE=44100
TARGET_AUDIO_CHANNELS=2

# Volume levels (in dB) - narration should be louder than music
NARRATION_VOLUME_DB=-18
MUSIC_VOLUME_DB=-28

# Normalize a clip to standard format (1920x1080, 25fps, 44100Hz stereo AAC)
normalize_clip() {
    local input="$1"
    local output="$2"

    # Get current format
    local audio_format=$(ffprobe -v error -select_streams a:0 -show_entries stream=sample_rate,channels -of csv=p=0 "$input" 2>/dev/null || echo "0,0")
    local video_res=$(ffprobe -v error -select_streams v:0 -show_entries stream=width,height -of csv=p=0 "$input" 2>/dev/null)
    local fps=$(ffprobe -v error -select_streams v:0 -show_entries stream=r_frame_rate -of csv=p=0 "$input" 2>/dev/null)

    local needs_normalize=false

    if [ "$audio_format" != "${TARGET_AUDIO_RATE},${TARGET_AUDIO_CHANNELS}" ]; then
        needs_normalize=true
    fi

    if [ "$video_res" != "${WIDTH},${HEIGHT}" ]; then
        needs_normalize=true
    fi

    if [ "$needs_normalize" = true ]; then
        ffmpeg -y -i "$input" \
            -vf "scale=${WIDTH}:${HEIGHT}:force_original_aspect_ratio=decrease,pad=${WIDTH}:${HEIGHT}:(ow-iw)/2:(oh-ih)/2,fps=${TARGET_FPS}" \
            -c:v libx264 -preset fast -crf 18 -pix_fmt yuv420p \
            -c:a aac -ar ${TARGET_AUDIO_RATE} -ac ${TARGET_AUDIO_CHANNELS} -b:a 192k \
            "$output" 2>/dev/null
        echo "    Normalized: $(basename "$input")"
    else
        cp "$input" "$output"
        echo "    Already normalized: $(basename "$input")"
    fi
}

echo "=== Building tt-rs Explainer Video ==="

# ============================================
# Step 1: Create title clip (robot on title background + music)
# ============================================
echo "Step 1: Creating title clip..."

# Convert title SVG to PNG
echo "  Converting title SVG to PNG..."
rsvg-convert -w $WIDTH -h $HEIGHT "$SVG_DIR/00-title.svg" -o "$IMAGES_DIR/title-background.png"

# Scale robot to 600px height (maintains 1:1 aspect = 600x600)
# Play forward then backward (boomerang) to double length
# Right-justify: x = 1920 - 600 = 1320, vertically centered: y = (1080 - 600) / 2 = 240
# Reduce music volume to not overpower narration
ffmpeg -y -loop 1 -i "$IMAGES_DIR/title-background.png" \
    -i "$ASSETS_DIR/robot.mp4" \
    -i "$ASSETS_DIR/music-upbeat.wav" \
    -filter_complex "[1:v]scale=600:600,split[fwd][rev];[rev]reverse[reversed];[fwd][reversed]concat=n=2:v=1:a=0[robot];[0:v][robot]overlay=1320:240:shortest=1[v];[2:a]volume=${MUSIC_VOLUME_DB}dB[a]" \
    -map "[v]" -map "[a]" \
    -t $TITLE_DURATION \
    -c:v libx264 -pix_fmt yuv420p -c:a aac -ar ${TARGET_AUDIO_RATE} -ac ${TARGET_AUDIO_CHANNELS} \
    "$CLIPS_DIR/00-title.mp4"

echo "  Created: 00-title.mp4"

# ============================================
# Step 2: Convert SVG slides to PNG and create clips
# ============================================
echo "Step 2: Converting SVG slides and creating clips..."

# Function to convert SVG to PNG using rsvg-convert
convert_svg_to_png() {
    local name="$1"
    local svg_path="$SVG_DIR/${name}.svg"
    local png_path="$IMAGES_DIR/${name}-slide.png"

    if [ -f "$svg_path" ]; then
        rsvg-convert -w $WIDTH -h $HEIGHT "$svg_path" -o "$png_path"
        echo "  Converted: $svg_path -> $png_path"
    else
        echo "  WARNING: SVG not found: $svg_path"
        return 1
    fi
}

create_section_clip() {
    local name="$1"
    local slide="$IMAGES_DIR/${name}-slide.png"
    local audio="$AUDIO_DIR/${name}.wav"
    local output="$CLIPS_DIR/${name}.mp4"

    if [ -f "$audio" ]; then
        # Use TTS audio - normalize volume and duration matches audio length
        ffmpeg -y -loop 1 -i "$slide" \
            -i "$audio" \
            -af "volume=${NARRATION_VOLUME_DB}dB" \
            -c:v libx264 -pix_fmt yuv420p -c:a aac -ar ${TARGET_AUDIO_RATE} -ac ${TARGET_AUDIO_CHANNELS} -shortest \
            "$output"
        echo "  Created clip with TTS audio: $output"
    else
        # Fallback to silent placeholder
        ffmpeg -y -loop 1 -i "$slide" \
            -f lavfi -i anullsrc=r=${TARGET_AUDIO_RATE}:cl=stereo \
            -t $PLACEHOLDER_DURATION \
            -c:v libx264 -pix_fmt yuv420p -c:a aac -ar ${TARGET_AUDIO_RATE} -ac ${TARGET_AUDIO_CHANNELS} -shortest \
            "$output"
        echo "  Created clip (silent placeholder): $output"
    fi
}

# Convert SVGs to PNGs and create clips for each narration section
for section in 01-hook 02-intro-demo 03-cta; do
    convert_svg_to_png "$section"
    create_section_clip "$section"
done

# ============================================
# Step 3: Create demo placeholder clip
# ============================================
echo "Step 3: Creating demo placeholder clip..."

ffmpeg -y -loop 1 -i "$IMAGES_DIR/demo-placeholder.png" \
    -f lavfi -i anullsrc=r=44100:cl=stereo \
    -t $PLACEHOLDER_DURATION \
    -c:v libx264 -pix_fmt yuv420p -c:a aac -shortest \
    "$CLIPS_DIR/03-demo.mp4"

echo "  Created: 03-demo.mp4"

# ============================================
# Step 4: Copy epilog
# ============================================
echo "Step 4: Copying epilog..."
cp "$ASSETS_DIR/epilog.mp4" "$CLIPS_DIR/99a-epilog.mp4"
echo "  Copied: 99a-epilog.mp4"

# ============================================
# Step 5: Create epilog extension (frame + music)
# ============================================
echo "Step 5: Creating epilog extension..."

# Reduce music volume to match narration levels
ffmpeg -y -loop 1 -i "$ASSETS_DIR/epilog-frame.png" \
    -i "$ASSETS_DIR/music-upbeat.wav" \
    -af "volume=${MUSIC_VOLUME_DB}dB" \
    -t $EPILOG_EXT_DURATION \
    -c:v libx264 -pix_fmt yuv420p -c:a aac -ar ${TARGET_AUDIO_RATE} -ac ${TARGET_AUDIO_CHANNELS} \
    "$CLIPS_DIR/99b-epilog-ext.mp4"

echo "  Created: 99b-epilog-ext.mp4"

# ============================================
# Step 6: Normalize all clips for concatenation
# ============================================
echo "Step 6: Normalizing clips for concatenation..."

NORMALIZED_DIR="$CLIPS_DIR/normalized"
mkdir -p "$NORMALIZED_DIR"

for clip in 00-title.mp4 01-hook.mp4 02-intro-demo.mp4 03-demo.mp4 03-cta.mp4 99a-epilog.mp4 99b-epilog-ext.mp4; do
    normalize_clip "$CLIPS_DIR/$clip" "$NORMALIZED_DIR/$clip"
done

# ============================================
# Step 7: Concatenate all normalized clips
# ============================================
echo "Step 7: Concatenating clips into preview..."

cat > "$NORMALIZED_DIR/concat.txt" << EOF
file '00-title.mp4'
file '01-hook.mp4'
file '02-intro-demo.mp4'
file '03-demo.mp4'
file '03-cta.mp4'
file '99a-epilog.mp4'
file '99b-epilog-ext.mp4'
EOF

# Re-encode during concat to ensure format consistency
ffmpeg -y -f concat -safe 0 -i "$NORMALIZED_DIR/concat.txt" \
    -c:v libx264 -preset fast -crf 18 -pix_fmt yuv420p \
    -c:a aac -ar ${TARGET_AUDIO_RATE} -ac ${TARGET_AUDIO_CHANNELS} -b:a 192k \
    "$OUTPUT_DIR/preview.mp4"

echo ""
echo "=== Build Complete ==="
echo "Output: $OUTPUT_DIR/preview.mp4"

# Show duration and dimensions
dur=$(ffprobe -i "$OUTPUT_DIR/preview.mp4" -show_entries format=duration -v quiet -of csv="p=0")
dims=$(ffprobe -i "$OUTPUT_DIR/preview.mp4" -v error -select_streams v:0 -show_entries stream=width,height -of csv=p=0)
printf "Duration: %.1f seconds\n" "$dur"
echo "Dimensions: $dims (landscape 16:9)"

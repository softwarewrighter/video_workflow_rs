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

echo "=== Building tt-rs Explainer Video ==="

# ============================================
# Step 1: Create title clip (robot on title background + music)
# ============================================
echo "Step 1: Creating title clip..."

# Get robot video dimensions
ROBOT_W=$(ffprobe -v error -select_streams v:0 -show_entries stream=width -of csv=p=0 "$ASSETS_DIR/robot.mp4")
ROBOT_H=$(ffprobe -v error -select_streams v:0 -show_entries stream=height -of csv=p=0 "$ASSETS_DIR/robot.mp4")

# Scale robot to fit right half (960px width max, maintain aspect)
# Position on right half of 1920x1080 canvas
ffmpeg -y -loop 1 -i "$IMAGES_DIR/title-background.png" \
    -i "$ASSETS_DIR/robot.mp4" \
    -i "$ASSETS_DIR/music-upbeat.wav" \
    -filter_complex "[1:v]scale=-1:600[robot];[0:v][robot]overlay=1100:240:shortest=1[v]" \
    -map "[v]" -map 2:a \
    -t $TITLE_DURATION \
    -c:v libx264 -pix_fmt yuv420p -c:a aac \
    "$CLIPS_DIR/00-title.mp4"

echo "  Created: 00-title.mp4"

# ============================================
# Step 2: Create narration section placeholders
# ============================================
echo "Step 2: Creating narration section placeholders..."

# Function to create a text slide from narration script
create_narration_slide() {
    local name="$1"
    local script_path="$SCRIPTS_DIR/${name}.txt"
    local output="$IMAGES_DIR/${name}-slide.png"

    # Read script content
    local text=$(cat "$script_path")

    # Create slide with text (dark background for contrast with title)
    magick -size ${WIDTH}x${HEIGHT} xc:"#1a1a2e" \
        -font Helvetica -pointsize 48 -fill "#ffffff" \
        -gravity Center -annotate +0+0 "$text" \
        "$output"

    echo "  Created slide: $output"
}

create_section_clip() {
    local name="$1"
    local slide="$IMAGES_DIR/${name}-slide.png"
    local output="$CLIPS_DIR/${name}.mp4"

    # Create video clip from slide (silent placeholder)
    ffmpeg -y -loop 1 -i "$slide" \
        -f lavfi -i anullsrc=r=44100:cl=stereo \
        -t $PLACEHOLDER_DURATION \
        -c:v libx264 -pix_fmt yuv420p -c:a aac -shortest \
        "$output"

    echo "  Created clip: $output"
}

# Create slides and clips for each narration section
for section in 01-hook 02-intro-demo 03-cta; do
    create_narration_slide "$section"
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

ffmpeg -y -loop 1 -i "$ASSETS_DIR/epilog-frame.png" \
    -i "$ASSETS_DIR/music-upbeat.wav" \
    -t $EPILOG_EXT_DURATION \
    -c:v libx264 -pix_fmt yuv420p -c:a aac \
    "$CLIPS_DIR/99b-epilog-ext.mp4"

echo "  Created: 99b-epilog-ext.mp4"

# ============================================
# Step 6: Concatenate all clips
# ============================================
echo "Step 6: Concatenating clips into preview..."

cat > "$CLIPS_DIR/concat.txt" << EOF
file '00-title.mp4'
file '01-hook.mp4'
file '02-intro-demo.mp4'
file '03-demo.mp4'
file '03-cta.mp4'
file '99a-epilog.mp4'
file '99b-epilog-ext.mp4'
EOF

ffmpeg -y -f concat -safe 0 -i "$CLIPS_DIR/concat.txt" \
    -c:v libx264 -c:a aac \
    "$OUTPUT_DIR/preview.mp4"

echo ""
echo "=== Build Complete ==="
echo "Output: $OUTPUT_DIR/preview.mp4"

# Show duration and dimensions
dur=$(ffprobe -i "$OUTPUT_DIR/preview.mp4" -show_entries format=duration -v quiet -of csv="p=0")
dims=$(ffprobe -i "$OUTPUT_DIR/preview.mp4" -v error -select_streams v:0 -show_entries stream=width,height -of csv=p=0)
printf "Duration: %.1f seconds\n" "$dur"
echo "Dimensions: $dims (landscape 16:9)"

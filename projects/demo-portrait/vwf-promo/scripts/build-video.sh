#!/bin/bash
# Build the final YouTube Short video (1080x1920 vertical)
set -e

source "$(dirname "$0")/common.sh"

cd "$PROJECT_DIR"

mkdir -p "$SLIDES_DIR" "$CLIPS_DIR" "$OUTPUT_DIR"

echo "Step 1: Generate vertical slide images (1080x1920)..."

# Function to create a vertical text slide using ImageMagick
create_slide() {
    local title="$1"
    local body="$2"
    local output="$3"

    convert -size ${WIDTH}x${HEIGHT} xc:"$BG_COLOR" \
        -font Helvetica-Bold -pointsize 64 -fill "$ACCENT_COLOR" \
        -gravity North -annotate +0+350 "$title" \
        -font Helvetica -pointsize 44 -fill "$TEXT_COLOR" \
        -interline-spacing 20 \
        -gravity Center -annotate +0+50 "$body" \
        -font Helvetica -pointsize 32 -fill "#888888" \
        -gravity South -annotate +0+100 "@softwarewrighter" \
        "$output"

    echo "  Created: $output"
}

# Create slides for each section
create_slide "VWF" "Making videos takes forever.\n\nWhat if you could\nautomate all of it?" "$SLIDES_DIR/01-hook.png"

create_slide "The Problem" "Manual video production\nis slow and error prone.\n\nMiss a step?\nStart over." "$SLIDES_DIR/02-problem.png"

create_slide "The Solution" "VWF is workflows as code.\n\nDefine steps in YAML.\nScripts. TTS. Assembly.\n\nAll automated." "$SLIDES_DIR/03-solution.png"

create_slide "Features" "Resume interrupted work.\nSkip completed steps.\nValidate outputs.\n\nBuilt in Rust." "$SLIDES_DIR/04-features.png"

create_slide "Try VWF" "Automate your\nvideo pipeline.\n\nGitHub: video_workflow_rs" "$SLIDES_DIR/05-cta.png"

echo "Step 2: Create video clips with audio..."
for name in 01-hook 02-problem 03-solution 04-features 05-cta; do
    echo "  $name..."
    $VID_IMAGE \
        --image "$SLIDES_DIR/${name}.png" \
        --audio "$AUDIO_DIR/${name}.wav" \
        --width $WIDTH \
        --height $HEIGHT \
        --output "$CLIPS_DIR/${name}.mp4"
done

echo "Step 3: Create concat list..."
cat > "$CLIPS_DIR/concat.txt" << EOF
$CLIPS_DIR/01-hook.mp4
$CLIPS_DIR/02-problem.mp4
$CLIPS_DIR/03-solution.mp4
$CLIPS_DIR/04-features.mp4
$CLIPS_DIR/05-cta.mp4
EOF

echo "Step 4: Concatenate into final video..."
$VID_CONCAT --list "$CLIPS_DIR/concat.txt" --output "$OUTPUT_DIR/preview.mp4" --reencode

echo "Done! Output: $OUTPUT_DIR/preview.mp4"

# Show duration and dimensions
dur=$(ffprobe -i "$OUTPUT_DIR/preview.mp4" -show_entries format=duration -v quiet -of csv="p=0")
dims=$(ffprobe -i "$OUTPUT_DIR/preview.mp4" -v error -select_streams v:0 -show_entries stream=width,height -of csv=p=0)
printf "Duration: %.1f seconds\n" "$dur"
echo "Dimensions: $dims (vertical 9:16)"

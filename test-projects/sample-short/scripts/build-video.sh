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
        -font Helvetica-Bold -pointsize 72 -fill "$ACCENT_COLOR" \
        -gravity North -annotate +0+300 "$title" \
        -font Helvetica -pointsize 48 -fill "$TEXT_COLOR" \
        -gravity Center -annotate +0+0 "$body" \
        -font Helvetica -pointsize 32 -fill "#888888" \
        -gravity South -annotate +0+100 "@softwarewrighter" \
        "$output"

    echo "  Created: $output"
}

# Create slides for each section
create_slide "VWF" "Still writing the same\nscripts by hand?\n\nWhat if your workflow\nwrote itself?" "$SLIDES_DIR/01-hook.png"

create_slide "Automate" "VWF turns your\nprocess into code.\n\nDefine steps in YAML.\nHit run.\nWatch it go." "$SLIDES_DIR/02-content.png"

create_slide "Try VWF" "Automate\neverything." "$SLIDES_DIR/03-cta.png"

echo "Step 2: Create video clips with audio..."
for name in 01-hook 02-content 03-cta; do
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
$CLIPS_DIR/02-content.mp4
$CLIPS_DIR/03-cta.mp4
EOF

echo "Step 4: Concatenate into final video..."
$VID_CONCAT --list "$CLIPS_DIR/concat.txt" --output "$OUTPUT_DIR/preview.mp4" --reencode

echo "Done! Output: $OUTPUT_DIR/preview.mp4"

# Show duration and dimensions
dur=$(ffprobe -i "$OUTPUT_DIR/preview.mp4" -show_entries format=duration -v quiet -of csv="p=0")
dims=$(ffprobe -i "$OUTPUT_DIR/preview.mp4" -v error -select_streams v:0 -show_entries stream=width,height -of csv=p=0)
printf "Duration: %.1f seconds\n" "$dur"
echo "Dimensions: $dims (vertical 9:16)"

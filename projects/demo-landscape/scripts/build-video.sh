#!/bin/bash
# Build the final video from slides + audio
set -e

source "$(dirname "$0")/common.sh"

cd "$PROJECT_DIR"

mkdir -p "$SLIDES_DIR" "$CLIPS_DIR" "$OUTPUT_DIR"

echo "Step 1: Generate slide images..."
$VID_SLIDE --title "What is a Workflow?" --subtitle "The Hook" \
    --body "Ever find yourself doing the same steps over and over?" \
    --body "There's a better way." \
    --image-only --output "$SLIDES_DIR/01-hook.png"

$VID_SLIDE --title "The Problem" \
    --body "Manual processes are slow, prone to errors, and tedious." \
    --body "One missed step can break everything." \
    --image-only --output "$SLIDES_DIR/02-problem.png"

$VID_SLIDE --title "The Solution" \
    --body "A workflow is a sequence of automated steps." \
    --body "Define it once, run it reliably every time." \
    --image-only --output "$SLIDES_DIR/03-solution.png"

$VID_SLIDE --title "The Benefit" \
    --body "Save hours of repetitive work." \
    --body "Reduce errors." \
    --body "Focus on what actually matters—your creative work." \
    --image-only --output "$SLIDES_DIR/04-benefit.png"

$VID_SLIDE --title "Try VWF" --subtitle "Workflows as Code" \
    --body "Your future self will thank you." \
    --image-only --output "$SLIDES_DIR/05-cta.png"

echo "Step 2: Create video clips with audio..."
for name in 01-hook 02-problem 03-solution 04-benefit 05-cta; do
    echo "  $name..."
    $VID_IMAGE \
        --image "$SLIDES_DIR/${name}.png" \
        --audio "$AUDIO_DIR/${name}.wav" \
        --output "$CLIPS_DIR/${name}.mp4"
done

echo "Step 3: Create concat list..."
cat > "$CLIPS_DIR/concat.txt" << EOF
$CLIPS_DIR/01-hook.mp4
$CLIPS_DIR/02-problem.mp4
$CLIPS_DIR/03-solution.mp4
$CLIPS_DIR/04-benefit.mp4
$CLIPS_DIR/05-cta.mp4
EOF

echo "Step 4: Concatenate into final video..."
$VID_CONCAT --list "$CLIPS_DIR/concat.txt" --output "$OUTPUT_DIR/preview.mp4" --reencode

echo "Done! Output: $OUTPUT_DIR/preview.mp4"

# Show duration
dur=$(ffprobe -i "$OUTPUT_DIR/preview.mp4" -show_entries format=duration -v quiet -of csv="p=0")
printf "Duration: %.1f seconds\n" "$dur"

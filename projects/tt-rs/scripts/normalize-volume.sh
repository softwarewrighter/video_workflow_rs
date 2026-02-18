#!/bin/bash
# Normalize audio volume AND FORMAT of a clip
# Usage: ./normalize-volume.sh <clip.mp4> [target_db]
#
# Default target: -25 dB (matches reference epilog with music)
# This script modifies the clip IN PLACE
#
# CRITICAL: Also enforces 44100 Hz stereo format to prevent
# audio loss during concatenation.

set -e
export PATH="/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:$PATH"

CLIP="$1"
TARGET_DB="${2:--25}"

if [ -z "$CLIP" ] || [ ! -f "$CLIP" ]; then
    echo "Usage: $0 <clip.mp4> [target_db]"
    echo "  target_db defaults to -25"
    exit 1
fi

# MANDATORY: Check and fix audio format first (44100 Hz stereo)
SAMPLE_RATE=$(ffprobe -v error -select_streams a:0 -show_entries stream=sample_rate -of csv=p=0 "$CLIP" 2>/dev/null || echo "0")
CHANNELS=$(ffprobe -v error -select_streams a:0 -show_entries stream=channels -of csv=p=0 "$CLIP" 2>/dev/null || echo "0")

if [ "$SAMPLE_RATE" != "44100" ] || [ "$CHANNELS" != "2" ]; then
    TEMP_FILE="/tmp/normalize-format-$$-$(basename "$CLIP")"
    ffmpeg -y -i "$CLIP" -c:v copy -c:a aac -ar 44100 -ac 2 "$TEMP_FILE" 2>/dev/null
    mv "$TEMP_FILE" "$CLIP"
    printf "%-40s FORMAT FIXED: %s Hz %s ch -> 44100 Hz stereo\n" "$(basename "$CLIP")" "$SAMPLE_RATE" "$CHANNELS"
fi

# Get current mean volume
CURRENT_DB=$(ffmpeg -i "$CLIP" -af "volumedetect" -f null - 2>&1 | grep mean_volume | sed 's/.*mean_volume: \([-0-9.]*\) dB/\1/')

if [ -z "$CURRENT_DB" ]; then
    echo "ERROR: Could not detect volume for $CLIP"
    exit 1
fi

# Calculate adjustment needed
ADJUST=$(echo "$TARGET_DB - $CURRENT_DB" | bc)

# Check if adjustment is needed (within 1 dB is acceptable)
ABS_ADJUST=$(echo "$ADJUST" | sed 's/-//')
if (( $(echo "$ABS_ADJUST < 1" | bc -l) )); then
    printf "%-40s %6.1f dB (OK)\n" "$(basename "$CLIP")" "$CURRENT_DB"
    exit 0
fi

# Apply volume adjustment
TEMP_FILE="/tmp/normalize-$$-$(basename "$CLIP")"

ffmpeg -y -i "$CLIP" \
    -af "volume=${ADJUST}dB" \
    -c:v copy -c:a aac -ar 44100 -ac 2 \
    "$TEMP_FILE" 2>/dev/null

# Verify new level
NEW_DB=$(ffmpeg -i "$TEMP_FILE" -af "volumedetect" -f null - 2>&1 | grep mean_volume | sed 's/.*mean_volume: \([-0-9.]*\) dB/\1/')

# Replace original
mv "$TEMP_FILE" "$CLIP"

printf "%-40s %6.1f dB -> %6.1f dB (adjusted %+.1f dB)\n" \
    "$(basename "$CLIP")" "$CURRENT_DB" "$NEW_DB" "$ADJUST"

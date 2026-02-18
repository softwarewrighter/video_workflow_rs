//! Normalize audio volume of video/audio clips.
//!
//! This step normalizes clips to a target dB level, ensuring consistent
//! audio levels across all clips before concatenation.
//!
//! # Volume Standards (based on epilog.mp4 reference)
//!
//! - **Narration/Speech**: -25 dB mean (matches reference with background music)
//! - **Background Music**: -32 dB mean (7 dB quieter than narration)
//!
//! # Why Post-Process Normalization?
//!
//! Applying volume adjustment during clip creation (via ffmpeg -af) often
//! produces inconsistent results due to AAC encoding variations. The proven
//! approach from explainer/shorts projects is to:
//!
//! 1. Create clips with native audio levels
//! 2. Normalize clips in a separate post-processing step
//! 3. Then concatenate the normalized clips
//!
//! This step implements that post-processing normalization.

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::process::Command;

use crate::context::StepCtx;

#[derive(Deserialize)]
struct Payload {
    /// Path to the clip to normalize (modified in place)
    clip_path: String,
    /// Target mean volume in dB (default: -25 for narration)
    #[serde(default = "default_target_db")]
    target_db: i32,
}

fn default_target_db() -> i32 {
    -25
}

pub fn execute(ctx: &mut StepCtx<'_>, payload: &serde_json::Value) -> Result<()> {
    let p: Payload = serde_json::from_value(payload.clone())
        .context("Failed to parse normalize_volume payload")?;

    // Resolve clip path (absolute or relative to workdir)
    let clip_path = if p.clip_path.starts_with('/') {
        p.clip_path.clone()
    } else {
        ctx.rt.workdir().join(&p.clip_path).to_string_lossy().to_string()
    };

    println!(
        "Normalizing volume: {} to {} dB",
        clip_path, p.target_db
    );

    // Check if clip exists
    if !std::path::Path::new(&clip_path).exists() {
        bail!("Clip not found: {}", clip_path);
    }

    // Step 1: Check and fix audio format (44100 Hz stereo)
    let sample_rate = get_audio_property(&clip_path, "sample_rate")?;
    let channels = get_audio_property(&clip_path, "channels")?;

    if sample_rate != "44100" || channels != "2" {
        println!(
            "  Fixing format: {}Hz {}ch -> 44100Hz stereo",
            sample_rate, channels
        );
        fix_audio_format(&clip_path)?;
    }

    // Step 2: Get current mean volume
    let current_db = get_mean_volume(&clip_path)?;
    println!("  Current volume: {:.1} dB", current_db);

    // Step 3: Calculate adjustment needed
    let target = p.target_db as f64;
    let adjust = target - current_db;

    // If within 1 dB, no adjustment needed
    if adjust.abs() < 1.0 {
        println!("  Volume OK (within 1 dB of target)");
        return Ok(());
    }

    // Step 4: Apply volume adjustment
    println!("  Adjusting by {:.1} dB", adjust);
    apply_volume_adjustment(&clip_path, adjust)?;

    // Step 5: Verify new level
    let new_db = get_mean_volume(&clip_path)?;
    println!(
        "  Normalized: {:.1} dB -> {:.1} dB",
        current_db, new_db
    );

    Ok(())
}

fn get_audio_property(clip_path: &str, property: &str) -> Result<String> {
    let output = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-select_streams", "a:0",
            "-show_entries", &format!("stream={}", property),
            "-of", "csv=p=0",
            clip_path,
        ])
        .output()
        .context("Failed to run ffprobe")?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn get_mean_volume(clip_path: &str) -> Result<f64> {
    let output = Command::new("ffmpeg")
        .args([
            "-i", clip_path,
            "-af", "volumedetect",
            "-f", "null",
            "-",
        ])
        .output()
        .context("Failed to run ffmpeg volumedetect")?;

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Parse "mean_volume: -25.1 dB"
    for line in stderr.lines() {
        if line.contains("mean_volume:") {
            if let Some(db_str) = line.split("mean_volume:").nth(1) {
                let db_str = db_str.trim().replace(" dB", "");
                return db_str.parse::<f64>()
                    .context("Failed to parse mean_volume");
            }
        }
    }

    bail!("Could not detect mean volume for {}", clip_path);
}

fn fix_audio_format(clip_path: &str) -> Result<()> {
    let temp_path = format!("{}.tmp.mp4", clip_path);

    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-i", clip_path,
            "-c:v", "copy",
            "-c:a", "aac",
            "-ar", "44100",
            "-ac", "2",
            &temp_path,
        ])
        .status()
        .context("Failed to run ffmpeg for format fix")?;

    if !status.success() {
        bail!("ffmpeg format fix failed");
    }

    std::fs::rename(&temp_path, clip_path)
        .context("Failed to replace original clip")?;

    Ok(())
}

fn apply_volume_adjustment(clip_path: &str, adjust_db: f64) -> Result<()> {
    let temp_path = format!("{}.tmp.mp4", clip_path);
    let volume_filter = format!("volume={}dB", adjust_db);

    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-i", clip_path,
            "-af", &volume_filter,
            "-c:v", "copy",
            "-c:a", "aac",
            "-ar", "44100",
            "-ac", "2",
            &temp_path,
        ])
        .status()
        .context("Failed to run ffmpeg for volume adjustment")?;

    if !status.success() {
        bail!("ffmpeg volume adjustment failed");
    }

    std::fs::rename(&temp_path, clip_path)
        .context("Failed to replace original clip")?;

    Ok(())
}

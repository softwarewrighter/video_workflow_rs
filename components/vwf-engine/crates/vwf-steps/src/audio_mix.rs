//! Mix audio overlay onto a video clip with volume control.
//!
//! This step takes a video with existing audio (e.g., narration) and mixes
//! in an overlay audio track (e.g., background music) at a specified volume.

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::process::Command;

use crate::context::StepCtx;

#[derive(Deserialize)]
struct Payload {
    /// Input video clip (with existing audio, e.g., narration)
    base_clip: String,
    /// Audio file to overlay (e.g., background music)
    overlay_audio: String,
    /// Volume level for overlay in dB (default: -32 for background music)
    #[serde(default = "default_overlay_volume")]
    overlay_volume: i32,
    /// Output path for the mixed video
    output_path: String,
    /// Whether to loop the overlay audio to match video length (default: true)
    #[serde(default = "default_loop")]
    loop_overlay: bool,
    /// Fade out duration in seconds for overlay audio (default: 2.0)
    #[serde(default = "default_fade_out")]
    fade_out: f64,
}

fn default_overlay_volume() -> i32 {
    -32 // Background music should be quieter than narration
}

fn default_loop() -> bool {
    true
}

fn default_fade_out() -> f64 {
    2.0
}

pub fn execute(ctx: &mut StepCtx<'_>, payload: &serde_json::Value) -> Result<()> {
    let p: Payload = serde_json::from_value(payload.clone())
        .with_context(|| ctx.error_context("payload decode audio_mix"))?;

    // Render paths with template variables
    let base_clip = ctx.render(&p.base_clip)?;
    let overlay_audio = ctx.render(&p.overlay_audio)?;
    let output_path = ctx.render(&p.output_path)?;

    // Resolve paths (relative to workdir)
    let workdir = ctx.rt.workdir();
    let resolved_base = if base_clip.starts_with('/') {
        base_clip.clone()
    } else {
        workdir.join(&base_clip).to_string_lossy().to_string()
    };

    let resolved_overlay = if overlay_audio.starts_with('/') {
        overlay_audio.clone()
    } else {
        workdir.join(&overlay_audio).to_string_lossy().to_string()
    };

    let resolved_output = if output_path.starts_with('/') {
        output_path.clone()
    } else {
        workdir.join(&output_path).to_string_lossy().to_string()
    };

    // Verify inputs exist
    if !std::path::Path::new(&resolved_base).exists() {
        bail!("Base clip not found: {}", resolved_base);
    }
    if !std::path::Path::new(&resolved_overlay).exists() {
        bail!("Overlay audio not found: {}", resolved_overlay);
    }

    println!(
        "Mixing audio: {} + {} ({}dB) -> {}",
        resolved_base, resolved_overlay, p.overlay_volume, resolved_output
    );

    // Ensure output directory exists
    if let Some(parent) = std::path::Path::new(&resolved_output).parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create output directory: {:?}", parent))?;
    }

    // Get video duration for fade-out calculation
    let duration = get_video_duration(&resolved_base)?;
    let fade_start = (duration - p.fade_out).max(0.0);

    // Build complex audio filter
    // [0:a] = base clip audio (narration)
    // [1:a] = overlay audio (music)
    let mut filter_parts = Vec::new();

    // Loop overlay audio if needed
    if p.loop_overlay {
        filter_parts
            .push("[1:a]aloop=loop=-1:size=2e+09,asetpts=PTS-STARTPTS[music_loop]".to_string());
    }

    let music_input = if p.loop_overlay {
        "[music_loop]"
    } else {
        "[1:a]"
    };

    // Apply volume and fade to overlay
    filter_parts.push(format!(
        "{}volume={}dB,afade=t=out:st={}:d={}[music_adj]",
        music_input, p.overlay_volume, fade_start, p.fade_out
    ));

    // Mix base audio with adjusted overlay
    filter_parts.push(
        "[0:a][music_adj]amix=inputs=2:duration=first:dropout_transition=2[aout]".to_string(),
    );

    let filter_complex = filter_parts.join(";");

    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            &resolved_base,
            "-stream_loop",
            "-1", // Loop the overlay audio file
            "-i",
            &resolved_overlay,
            "-filter_complex",
            &filter_complex,
            "-map",
            "0:v", // Video from base clip
            "-map",
            "[aout]", // Mixed audio
            "-c:v",
            "copy", // Don't re-encode video
            "-c:a",
            "aac",
            "-b:a",
            "192k",
            "-shortest", // Stop when shortest input ends
            &resolved_output,
        ])
        .status()
        .with_context(|| ctx.error_context("spawn ffmpeg audio_mix"))?;

    if !status.success() {
        bail!(
            "ffmpeg audio_mix failed with exit code: {:?}",
            status.code()
        );
    }

    println!("  Created: {}", resolved_output);
    Ok(())
}

/// Get video duration in seconds using ffprobe
fn get_video_duration(path: &str) -> Result<f64> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "csv=p=0",
            path,
        ])
        .output()
        .context("Failed to run ffprobe for duration")?;

    let duration_str = String::from_utf8_lossy(&output.stdout);
    duration_str
        .trim()
        .parse::<f64>()
        .context("Failed to parse video duration")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        assert_eq!(default_overlay_volume(), -32);
        assert!(default_loop());
        assert_eq!(default_fade_out(), 2.0);
    }
}

//! Concatenate multiple video clips into a single video.
//!
//! Uses ffmpeg's concat demuxer for efficient lossless concatenation
//! when clips have matching codecs and parameters.

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::io::Write;
use std::process::Command;

use crate::context::StepCtx;

#[derive(Deserialize)]
struct Payload {
    /// List of video clip paths to concatenate (in order)
    clips: Vec<String>,
    /// Output path for the concatenated video
    output_path: String,
    /// Whether to re-encode (default: false for copy mode)
    #[serde(default)]
    reencode: bool,
}

pub fn execute(ctx: &mut StepCtx<'_>, payload: &serde_json::Value) -> Result<()> {
    let p: Payload = serde_json::from_value(payload.clone())
        .with_context(|| ctx.error_context("payload decode video_concat"))?;

    if p.clips.is_empty() {
        bail!("video_concat requires at least one clip");
    }

    // Render all paths with template variables
    let clips: Vec<String> = p
        .clips
        .iter()
        .map(|c| ctx.render(c))
        .collect::<Result<Vec<_>>>()?;

    let output_path = ctx.render(&p.output_path)?;

    // Resolve paths (relative to workdir)
    let workdir = ctx.rt.workdir();
    let resolved_clips: Vec<String> = clips
        .iter()
        .map(|c| {
            if c.starts_with('/') {
                c.clone()
            } else {
                workdir.join(c).to_string_lossy().to_string()
            }
        })
        .collect();

    let resolved_output = if output_path.starts_with('/') {
        output_path.clone()
    } else {
        workdir.join(&output_path).to_string_lossy().to_string()
    };

    // Verify all input clips exist
    for clip in &resolved_clips {
        if !std::path::Path::new(clip).exists() {
            bail!("Input clip not found: {}", clip);
        }
    }

    println!(
        "Concatenating {} clips -> {}",
        resolved_clips.len(),
        resolved_output
    );

    // Create concat list file for ffmpeg
    let concat_list = create_concat_list(&resolved_clips)?;

    // Ensure output directory exists
    if let Some(parent) = std::path::Path::new(&resolved_output).parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create output directory: {:?}", parent))?;
    }

    // Run ffmpeg concat
    let mut args = vec![
        "-y".to_string(),
        "-f".to_string(),
        "concat".to_string(),
        "-safe".to_string(),
        "0".to_string(),
        "-i".to_string(),
        concat_list.clone(),
    ];

    if p.reencode {
        // Re-encode mode: useful when clips have different parameters
        args.extend([
            "-c:v".to_string(),
            "libx264".to_string(),
            "-preset".to_string(),
            "medium".to_string(),
            "-crf".to_string(),
            "23".to_string(),
            "-c:a".to_string(),
            "aac".to_string(),
            "-b:a".to_string(),
            "192k".to_string(),
        ]);
    } else {
        // Copy mode: fast, lossless (requires matching codecs)
        args.extend(["-c".to_string(), "copy".to_string()]);
    }

    args.push(resolved_output.clone());

    let status = Command::new("ffmpeg")
        .args(&args)
        .status()
        .with_context(|| ctx.error_context("spawn ffmpeg concat"))?;

    // Clean up temp concat list
    let _ = std::fs::remove_file(&concat_list);

    if !status.success() {
        bail!("ffmpeg concat failed with exit code: {:?}", status.code());
    }

    println!("  Created: {}", resolved_output);
    Ok(())
}

/// Create a temporary file listing all clips for ffmpeg concat demuxer
fn create_concat_list(clips: &[String]) -> Result<String> {
    let temp_path = std::env::temp_dir().join(format!("vwf_concat_{}.txt", std::process::id()));
    let mut file =
        std::fs::File::create(&temp_path).context("Failed to create concat list file")?;

    for clip in clips {
        // ffmpeg concat format: file 'path'
        // Escape single quotes in paths
        let escaped = clip.replace('\'', "'\\''");
        writeln!(file, "file '{}'", escaped)?;
    }

    Ok(temp_path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn creates_concat_list() {
        let clips = vec![
            "/path/to/clip1.mp4".to_string(),
            "/path/to/clip2.mp4".to_string(),
        ];

        let list_path = create_concat_list(&clips).unwrap();
        let mut contents = String::new();
        std::fs::File::open(&list_path)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();

        assert!(contents.contains("file '/path/to/clip1.mp4'"));
        assert!(contents.contains("file '/path/to/clip2.mp4'"));

        std::fs::remove_file(&list_path).unwrap();
    }
}

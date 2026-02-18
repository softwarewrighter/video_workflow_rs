//! Output validation for resume/skip logic.

use std::path::Path;
use std::process::Command;

/// Check if an output file exists and is valid (non-empty, valid media).
pub fn output_is_valid(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    match path.extension().and_then(|e| e.to_str()) {
        Some("wav") | Some("mp3") | Some("m4a") => audio_valid(path),
        Some("mp4") | Some("mkv") | Some("webm") => video_valid(path),
        _ => file_non_empty(path),
    }
}

fn file_non_empty(path: &Path) -> bool {
    path.metadata().map(|m| m.len() > 0).unwrap_or(false)
}

fn audio_valid(path: &Path) -> bool {
    media_duration(path).map(|d| d > 0.0).unwrap_or(false)
}

fn video_valid(path: &Path) -> bool {
    media_duration(path).map(|d| d > 0.0).unwrap_or(false)
}

fn media_duration(path: &Path) -> Option<f64> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "quiet",
            "-show_entries",
            "format=duration",
            "-of",
            "csv=p=0",
        ])
        .arg(path)
        .output()
        .ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.trim().parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_invalid() {
        assert!(!output_is_valid(Path::new("/nonexistent/file.wav")));
    }
}

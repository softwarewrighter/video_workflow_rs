//! Audit produced assets using vision-capable LLMs.
//!
//! This step uses a recursive approach to validate video/image quality:
//! 1. For videos, extract frames at regular intervals
//! 2. Send each frame to a vision model (llava, qwen2.5vl, llama3.2-vision)
//! 3. Aggregate feedback and report issues
//! 4. Optionally fail the workflow if critical issues found

use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::path::Path;
use std::process::Command;

use crate::context::StepCtx;

#[derive(Deserialize)]
struct Payload {
    /// Assets to audit (images or videos)
    assets: Vec<String>,
    /// Audit criteria/prompt for the vision model
    audit_prompt: String,
    /// Vision model to use (default: llava)
    #[serde(default = "default_model")]
    model: String,
    /// Ollama server URL
    #[serde(default = "default_server")]
    server: String,
    /// Number of frames to extract from videos (default: 5)
    #[serde(default = "default_frame_count")]
    frame_count: u32,
    /// Output path for audit report
    output_path: String,
    /// Fail workflow if issues found (default: false)
    #[serde(default)]
    fail_on_issues: bool,
    /// Keywords that indicate critical issues (default: ["error", "broken", "corrupt"])
    #[serde(default = "default_critical_keywords")]
    critical_keywords: Vec<String>,
}

fn default_model() -> String {
    "llava".to_string()
}

fn default_server() -> String {
    "http://localhost:11434".to_string()
}

fn default_frame_count() -> u32 {
    5
}

fn default_critical_keywords() -> Vec<String> {
    vec![
        "error".to_string(),
        "broken".to_string(),
        "corrupt".to_string(),
        "missing".to_string(),
        "blank".to_string(),
        "black screen".to_string(),
    ]
}

pub fn execute(ctx: &mut StepCtx<'_>, payload: &serde_json::Value) -> Result<()> {
    let p: Payload = serde_json::from_value(payload.clone())
        .with_context(|| ctx.error_context("payload decode llm_audit"))?;

    let workdir = ctx.rt.workdir();

    // Resolve paths
    let assets: Vec<String> = p
        .assets
        .iter()
        .map(|a| {
            let rendered = ctx.render(a)?;
            Ok(if rendered.starts_with('/') {
                rendered
            } else {
                workdir.join(&rendered).to_string_lossy().to_string()
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let output_path = ctx.render(&p.output_path)?;
    let output_path = if output_path.starts_with('/') {
        output_path
    } else {
        workdir.join(&output_path).to_string_lossy().to_string()
    };

    let server = ctx.render(&p.server)?;
    let model = ctx.render(&p.model)?;
    let audit_prompt = ctx.render(&p.audit_prompt)?;

    println!("Auditing {} assets with {}", assets.len(), model);

    let mut report = AuditReport {
        model: model.clone(),
        total_assets: assets.len(),
        total_frames_analyzed: 0,
        findings: Vec::new(),
        critical_issues: Vec::new(),
    };

    // Create temp directory for extracted frames
    let temp_dir = std::env::temp_dir().join(format!("vwf_audit_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir)?;

    for asset in &assets {
        if !Path::new(asset).exists() {
            report.findings.push(AssetFinding {
                asset: asset.clone(),
                frame: None,
                feedback: "Asset file not found".to_string(),
                is_critical: true,
            });
            continue;
        }

        let ext = Path::new(asset)
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        let frames = if ["mp4", "mov", "avi", "mkv", "webm"].contains(&ext.as_str()) {
            // Extract frames from video
            extract_video_frames(asset, &temp_dir, p.frame_count)?
        } else if ["png", "jpg", "jpeg", "webp", "gif"].contains(&ext.as_str()) {
            // Single image
            vec![asset.clone()]
        } else {
            println!("  Skipping unsupported format: {}", asset);
            continue;
        };

        println!("  Analyzing {} ({} frames)", asset, frames.len());

        for frame in &frames {
            report.total_frames_analyzed += 1;

            let feedback = analyze_frame(&server, &model, frame, &audit_prompt)?;

            // Check for critical issues
            let is_critical = p
                .critical_keywords
                .iter()
                .any(|kw| feedback.to_lowercase().contains(&kw.to_lowercase()));

            let finding = AssetFinding {
                asset: asset.clone(),
                frame: if frames.len() > 1 {
                    Some(frame.clone())
                } else {
                    None
                },
                feedback: feedback.clone(),
                is_critical,
            };

            if is_critical {
                report.critical_issues.push(finding.clone());
            }
            report.findings.push(finding);
        }
    }

    // Clean up temp directory
    let _ = std::fs::remove_dir_all(&temp_dir);

    // Write report
    if let Some(parent) = Path::new(&output_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let report_json = serde_json::to_string_pretty(&report)?;
    std::fs::write(&output_path, &report_json)?;

    println!(
        "Audit complete: {} frames analyzed, {} critical issues",
        report.total_frames_analyzed,
        report.critical_issues.len()
    );
    println!("Report saved to: {}", output_path);

    // Print critical issues
    if !report.critical_issues.is_empty() {
        println!("\nCritical issues found:");
        for issue in &report.critical_issues {
            println!("  - {}: {}", issue.asset, issue.feedback);
        }

        if p.fail_on_issues {
            bail!(
                "Audit failed: {} critical issues found",
                report.critical_issues.len()
            );
        }
    }

    Ok(())
}

/// Extract frames from a video at regular intervals
fn extract_video_frames(video_path: &str, temp_dir: &Path, frame_count: u32) -> Result<Vec<String>> {
    // Get video duration
    let duration = get_video_duration(video_path)?;
    let interval = duration / (frame_count as f64 + 1.0);

    let mut frames = Vec::new();
    let video_name = Path::new(video_path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "video".to_string());

    for i in 1..=frame_count {
        let timestamp = interval * i as f64;
        let output_frame = temp_dir.join(format!("{}_{:03}.png", video_name, i));

        let status = Command::new("ffmpeg")
            .args([
                "-y",
                "-ss",
                &format!("{:.2}", timestamp),
                "-i",
                video_path,
                "-frames:v",
                "1",
                "-q:v",
                "2",
                output_frame.to_str().unwrap(),
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .context("Failed to extract video frame")?;

        if status.success() && output_frame.exists() {
            frames.push(output_frame.to_string_lossy().to_string());
        }
    }

    Ok(frames)
}

/// Get video duration in seconds
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
        .context("Failed to run ffprobe")?;

    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<f64>()
        .context("Failed to parse duration")
}

/// Analyze a single frame using the vision model
fn analyze_frame(server: &str, model: &str, frame_path: &str, prompt: &str) -> Result<String> {
    // Read image and encode as base64
    let image_data = std::fs::read(frame_path).context("Failed to read frame")?;
    let base64_image = base64_encode(&image_data);

    // Call Ollama with vision model
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;

    let request_body = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "images": [base64_image],
        "stream": false
    });

    let response = client
        .post(format!("{}/api/generate", server))
        .json(&request_body)
        .send()
        .context("Failed to call Ollama")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        bail!("Ollama request failed ({}): {}", status, body);
    }

    let result: serde_json::Value = response.json().context("Failed to parse response")?;

    result["response"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("No response in Ollama output"))
}

fn base64_encode(data: &[u8]) -> String {
    use std::io::Write;
    let mut encoder = base64::write::EncoderStringWriter::new(&base64::engine::general_purpose::STANDARD);
    encoder.write_all(data).unwrap();
    encoder.into_inner()
}

#[derive(Debug, Clone, serde::Serialize)]
struct AuditReport {
    model: String,
    total_assets: usize,
    total_frames_analyzed: usize,
    findings: Vec<AssetFinding>,
    critical_issues: Vec<AssetFinding>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct AssetFinding {
    asset: String,
    frame: Option<String>,
    feedback: String,
    is_critical: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        assert_eq!(default_model(), "llava");
        assert_eq!(default_frame_count(), 5);
        assert!(!default_critical_keywords().is_empty());
    }
}

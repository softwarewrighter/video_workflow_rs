//! Generate title/text slides for video production.
//!
//! Creates static image slides with text for title screens, bullet points,
//! or quote cards. Uses ImageMagick for text rendering.

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::process::Command;

use crate::context::StepCtx;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SlideTemplate {
    Title,  // Large centered text with optional subtitle
    Bullet, // List of bullet points
    Quote,  // Quoted text with attribution
}

impl Default for SlideTemplate {
    fn default() -> Self {
        Self::Title
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Orientation {
    Landscape,
    Portrait,
}

impl Default for Orientation {
    fn default() -> Self {
        Self::Landscape
    }
}

#[derive(Deserialize)]
struct Payload {
    /// Slide template type
    #[serde(default)]
    template: SlideTemplate,
    /// Main text content
    text: String,
    /// Subtitle or attribution (optional)
    #[serde(default)]
    subtext: Option<String>,
    /// Background image path (optional, uses solid color if not provided)
    #[serde(default)]
    background: Option<String>,
    /// Background color when no image (default: #1a1a2e)
    #[serde(default = "default_bg_color")]
    background_color: String,
    /// Text color (default: #ffffff)
    #[serde(default = "default_text_color")]
    text_color: String,
    /// Output image path
    output_path: String,
    /// Slide orientation
    #[serde(default)]
    orientation: Orientation,
    /// Font name (default: DejaVu-Sans-Bold)
    #[serde(default = "default_font")]
    font: String,
}

fn default_bg_color() -> String {
    "#1a1a2e".to_string() // Dark blue-gray
}

fn default_text_color() -> String {
    "#ffffff".to_string() // White
}

fn default_font() -> String {
    "DejaVu-Sans-Bold".to_string()
}

pub fn execute(ctx: &mut StepCtx<'_>, payload: &serde_json::Value) -> Result<()> {
    let p: Payload = serde_json::from_value(payload.clone())
        .with_context(|| ctx.error_context("payload decode create_slide"))?;

    // Render text with template variables
    let text = ctx.render(&p.text)?;
    let subtext = p.subtext.as_ref().map(|s| ctx.render(s)).transpose()?;
    let output_path = ctx.render(&p.output_path)?;
    let background = p.background.as_ref().map(|b| ctx.render(b)).transpose()?;

    let workdir = ctx.rt.workdir();
    let resolved_output = if output_path.starts_with('/') {
        output_path.clone()
    } else {
        workdir.join(&output_path).to_string_lossy().to_string()
    };

    let resolved_bg = background.map(|b| {
        if b.starts_with('/') {
            b
        } else {
            workdir.join(&b).to_string_lossy().to_string()
        }
    });

    // Get dimensions based on orientation
    let (width, height) = match p.orientation {
        Orientation::Landscape => (1920, 1080),
        Orientation::Portrait => (1080, 1920),
    };

    println!(
        "Creating {:?} slide: {} ({}x{})",
        p.template, resolved_output, width, height
    );

    // Ensure output directory exists
    if let Some(parent) = std::path::Path::new(&resolved_output).parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create output directory: {:?}", parent))?;
    }

    // Build ImageMagick command with owned strings
    let size_str = format!("{}x{}", width, height);
    let resize_str = format!("{}x{}^", width, height);
    let bg_canvas = format!("xc:{}", p.background_color);
    let pointsize_main = (width / 16).to_string();
    let pointsize_sub = (width / 32).to_string();
    let pointsize_quote = (width / 24).to_string();
    let pointsize_attr = (width / 40).to_string();

    let mut cmd = Command::new("convert");

    // Start with background
    if let Some(ref bg_path) = resolved_bg {
        if !std::path::Path::new(bg_path).exists() {
            bail!("Background image not found: {}", bg_path);
        }
        cmd.args([
            bg_path.as_str(),
            "-resize",
            &resize_str,
            "-gravity",
            "center",
            "-extent",
            &size_str,
        ]);
    } else {
        cmd.args(["-size", &size_str, &bg_canvas]);
    }

    // Common text settings
    cmd.args(["-fill", &p.text_color, "-font", &p.font]);

    // Apply template-specific text rendering
    match p.template {
        SlideTemplate::Title => {
            cmd.args(["-pointsize", &pointsize_main, "-gravity", "center"]);
            if let Some(ref sub) = subtext {
                cmd.args(["-annotate", "+0-50", &text]);
                cmd.args(["-pointsize", &pointsize_sub, "-annotate", "+0+80", sub]);
            } else {
                cmd.args(["-annotate", "+0+0", &text]);
            }
        }
        SlideTemplate::Bullet => {
            // Split text by newlines, each line becomes a bullet
            let bullet_text: String = text
                .lines()
                .map(|line| format!("  \u{2022} {}", line.trim()))
                .collect::<Vec<_>>()
                .join("\n");
            cmd.args([
                "-pointsize",
                &pointsize_sub,
                "-gravity",
                "west",
                "-annotate",
                "+100+0",
                &bullet_text,
            ]);
        }
        SlideTemplate::Quote => {
            let quoted = format!("\u{201C}{}\u{201D}", text);
            cmd.args(["-pointsize", &pointsize_quote, "-gravity", "center"]);
            if let Some(ref attr) = subtext {
                let attribution = format!("\u{2014} {}", attr);
                cmd.args(["-annotate", "+0-30", &quoted]);
                cmd.args([
                    "-pointsize",
                    &pointsize_attr,
                    "-annotate",
                    "+0+80",
                    &attribution,
                ]);
            } else {
                cmd.args(["-annotate", "+0+0", &quoted]);
            }
        }
    }

    cmd.arg(&resolved_output);

    let status = cmd
        .status()
        .with_context(|| ctx.error_context("spawn ImageMagick convert"))?;

    if !status.success() {
        bail!(
            "ImageMagick convert failed with exit code: {:?}",
            status.code()
        );
    }

    println!("  Created: {}", resolved_output);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        assert_eq!(default_bg_color(), "#1a1a2e");
        assert_eq!(default_text_color(), "#ffffff");
        assert_eq!(default_font(), "DejaVu-Sans-Bold");
    }
}

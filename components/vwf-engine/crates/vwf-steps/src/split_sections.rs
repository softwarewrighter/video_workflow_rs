//! Handler for split_sections step kind.

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;

use super::context::StepCtx;

#[derive(Deserialize)]
struct Output {
    heading: String,
    path: String,
}

#[derive(Deserialize)]
struct Payload {
    input_path: String,
    outputs: Vec<Output>,
}

pub fn execute(ctx: &mut StepCtx<'_>, payload: &Value) -> Result<()> {
    let p: Payload = serde_json::from_value(payload.clone())
        .with_context(|| ctx.error_context("payload decode split_sections"))?;
    let input_path = ctx.render(&p.input_path)?;
    let text = ctx.rt.read_text(&input_path)?;
    for out in p.outputs {
        write_section(ctx, &text, &out)?;
    }
    Ok(())
}

fn write_section(ctx: &mut StepCtx<'_>, text: &str, out: &Output) -> Result<()> {
    let heading = ctx.render(&out.heading)?;
    let path = ctx.render(&out.path)?;
    let section = extract_section(text, &heading)
        .with_context(|| ctx.error_context(&format!("cannot extract heading `{heading}`")))?;
    ctx.rt.write_text(&path, section.trim())
}

fn extract_section<'a>(text: &'a str, heading: &str) -> Result<&'a str> {
    let start = text.find(heading).ok_or_else(|| anyhow::anyhow!("heading not found"))?;
    let after = &text[start + heading.len()..];
    let end_idx = find_next_heading(after);
    Ok(after[..end_idx].trim_matches(['\n', '\r'].as_ref()))
}

fn find_next_heading(text: &str) -> usize {
    for (i, line) in text.lines().enumerate() {
        if i == 0 {
            continue;
        }
        if is_heading(line.trim()) {
            return byte_offset_of_line(text, i).saturating_sub(1);
        }
    }
    text.len()
}

fn is_heading(line: &str) -> bool {
    if !line.ends_with(':') {
        return false;
    }
    let letters: String = line.chars().filter(|c| c.is_alphabetic()).collect();
    !letters.is_empty() && letters.chars().filter(|c| c.is_uppercase()).count() * 2 >= letters.len()
}

fn byte_offset_of_line(text: &str, line_num: usize) -> usize {
    text.lines().take(line_num).map(|l| l.len() + 1).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_section_between_headings() {
        let text = "DESCRIPTION:\nA\n\nNARRATION:\nB\n";
        assert_eq!(extract_section(text, "DESCRIPTION:").unwrap().trim(), "A");
        assert_eq!(extract_section(text, "NARRATION:").unwrap().trim(), "B");
    }

    #[test]
    fn is_heading_detects_caps() {
        assert!(is_heading("DESCRIPTION:"));
        assert!(is_heading("NARRATION:"));
        assert!(!is_heading("description:"));
        assert!(!is_heading("no colon"));
    }
}

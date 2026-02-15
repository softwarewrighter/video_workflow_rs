use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;

use crate::config::{StepConfig, StepKind};
use crate::render::render_template;
use crate::runtime::{LlmRequest, Runtime};

/// Execute a single step.
pub fn execute_step(
    rt: &mut dyn Runtime,
    vars: &BTreeMap<String, String>,
    step: &StepConfig,
) -> Result<()> {
    match step.kind {
        StepKind::EnsureDirs => {
            #[derive(Deserialize)]
            struct Payload {
                dirs: Vec<String>,
            }
            let p: Payload = serde_json::from_value(step.payload.clone())
                .with_context(|| format!("step `{}` payload decode ensure_dirs", step.id))?;
            for d in p.dirs {
                rt.ensure_dir(&render_template(&d, vars)?)?;
            }
            Ok(())
        }
        StepKind::WriteFile => {
            #[derive(Deserialize)]
            struct Payload {
                path: String,
                content: String,
            }
            let p: Payload = serde_json::from_value(step.payload.clone())
                .with_context(|| format!("step `{}` payload decode write_file", step.id))?;
            let path = render_template(&p.path, vars)?;
            let content = render_template(&p.content, vars)?;
            rt.write_text(&path, &content)?;
            Ok(())
        }
        StepKind::SplitSections => {
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
            let p: Payload = serde_json::from_value(step.payload.clone())
                .with_context(|| format!("step `{}` payload decode split_sections", step.id))?;
            let input_path = render_template(&p.input_path, vars)?;
            let text = rt.read_text(&input_path)?;
            for out in p.outputs {
                let heading = render_template(&out.heading, vars)?;
                let path = render_template(&out.path, vars)?;
                let section = extract_section(&text, &heading).with_context(|| {
                    format!("step `{}` cannot extract heading `{heading}`", step.id)
                })?;
                rt.write_text(&path, section.trim().to_string().as_str())?;
            }
            Ok(())
        }
        StepKind::RunCommand => {
            #[derive(Deserialize)]
            struct Payload {
                program: String,
                #[serde(default)]
                args: Vec<String>,
                #[serde(default)]
                cwd: Option<String>,
                #[serde(default)]
                capture_path: Option<String>,
            }
            let p: Payload = serde_json::from_value(step.payload.clone())
                .with_context(|| format!("step `{}` payload decode run_command", step.id))?;
            let program = render_template(&p.program, vars)?;
            let args: Vec<String> = p
                .args
                .iter()
                .map(|a| render_template(a, vars))
                .collect::<Result<_>>()?;
            let cwd = match &p.cwd {
                Some(c) => Some(render_template(c, vars)?),
                None => None,
            };
            let out = rt.run_command(&program, &args, cwd.as_deref())?;
            if let Some(cap) = p.capture_path {
                let cap = render_template(&cap, vars)?;
                let s = format!(
                    "status: {}\n\nstdout:\n{}\n\nstderr:\n{}\n",
                    out.status, out.stdout, out.stderr
                );
                rt.write_text(&cap, &s)?;
            }
            Ok(())
        }
        StepKind::LlmGenerate => {
            #[derive(Deserialize)]
            struct Payload {
                system: String,
                user_prompt_path: String,
                output_path: String,
                provider: String,
                #[serde(default)]
                mock_response: Option<String>,
            }
            let p: Payload = serde_json::from_value(step.payload.clone())
                .with_context(|| format!("step `{}` payload decode llm_generate", step.id))?;
            let system = render_template(&p.system, vars)?;
            let user_prompt_path = render_template(&p.user_prompt_path, vars)?;
            let user = rt.read_text(&user_prompt_path)?;
            let provider = render_template(&p.provider, vars)?;
            // If provider is mock and mock_response present, temporarily override the mock client by embedding content in request.
            // (Real adapters can ignore this field; tests can set canned responses.)
            let mut req = LlmRequest {
                system,
                user,
                provider,
            };
            if let Some(m) = p.mock_response {
                // Put mock response at top so MockLlmClient can pick it up if configured to echo.
                req.user = format!("[MOCK_RESPONSE]\n{}\n\n{}", m, req.user);
            }
            let resp = rt.llm().generate(req)?;
            let output_path = render_template(&p.output_path, vars)?;
            rt.write_text(&output_path, &resp)?;
            Ok(())
        }
    }
}

/// Extracts the text after `heading` until the next ALL-CAPS heading or EOF.
fn extract_section<'a>(text: &'a str, heading: &str) -> Result<&'a str> {
    let start = text
        .find(heading)
        .ok_or_else(|| anyhow::anyhow!("heading not found"))?;
    let after = &text[start + heading.len()..];
    // heuristic: stop at another heading line that ends with ':' and is mostly uppercase or a known label
    let mut end_idx = after.len();
    for (i, line) in after.lines().enumerate() {
        let line_trim = line.trim();
        if i == 0 {
            continue;
        }
        if line_trim.ends_with(':') && is_headingish(line_trim) {
            // compute byte index to start of this line
            let mut bytes = 0usize;
            for (j, l) in after.lines().enumerate() {
                if j == i {
                    break;
                }
                bytes += l.len() + 1; // + newline
            }
            end_idx = bytes.saturating_sub(1); // remove preceding newline
            break;
        }
    }
    Ok(after[..end_idx].trim_matches(['\n', '\r'].as_ref()))
}

fn is_headingish(line: &str) -> bool {
    let letters: String = line.chars().filter(|c| c.is_alphabetic()).collect();
    if letters.is_empty() {
        return false;
    }
    let upper = letters.chars().filter(|c| c.is_uppercase()).count();
    upper * 2 >= letters.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{FsRuntime, MockLlmClient};
    use tempfile::TempDir;

    #[test]
    fn split_sections_extracts() {
        let tmp = TempDir::new().unwrap();
        let mut rt = FsRuntime::new(tmp.path(), Box::new(MockLlmClient::echo()));
        rt.write_text("in.txt", "DESCRIPTION:\nA\n\nNARRATION:\nB\n")
            .unwrap();

        let cfg = StepConfig {
            id: "s".into(),
            kind: StepKind::SplitSections,
            payload: serde_json::json!({
                "input_path":"in.txt",
                "outputs":[
                  {"heading":"DESCRIPTION:","path":"a.txt"},
                  {"heading":"NARRATION:","path":"b.txt"}
                ]
            }),
        };
        let vars = BTreeMap::new();
        execute_step(&mut rt, &vars, &cfg).unwrap();
        assert_eq!(rt.read_text("a.txt").unwrap().trim(), "A");
        assert_eq!(rt.read_text("b.txt").unwrap().trim(), "B");
    }
}

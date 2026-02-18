//! LLM-driven workflow generation command.
//!
//! Reads a project brief and generates a complete workflow.yaml using local LLM.

use std::path::Path;

use anyhow::{Context, Result, bail};

use vwf_core::{LlmClient, LlmReq, OllamaClient, WorkflowConfig};

/// Generate a workflow from a project brief.
pub fn generate(project_dir: &Path, model: &str, context_dir: Option<&Path>) -> Result<()> {
    // Find the context documents directory
    let ctx_dir = match context_dir {
        Some(d) => d.to_path_buf(),
        None => find_llm_context_dir()?,
    };

    // Read the project brief
    let brief_path = project_dir.join("brief.txt");
    if !brief_path.exists() {
        bail!(
            "Project brief not found: {}\n\nCreate a brief.txt file describing your video.",
            brief_path.display()
        );
    }
    let brief = std::fs::read_to_string(&brief_path)
        .with_context(|| format!("read {}", brief_path.display()))?;

    println!("Generating workflow for: {}", project_dir.display());
    println!("  Brief: {} chars", brief.len());
    println!("  Model: {}", model);
    println!("  Context: {}", ctx_dir.display());

    // Load all context documents
    let context = load_context_documents(&ctx_dir)?;
    println!("  Loaded {} chars of context", context.len());

    // Build the LLM prompt
    let system_prompt = build_system_prompt(&context);
    let user_prompt = build_user_prompt(&brief, project_dir);

    // Call Ollama
    let mut llm = OllamaClient::new(model.to_string());
    let req = LlmReq {
        system: system_prompt,
        user: user_prompt,
        provider: format!("ollama:{}", model),
    };

    println!("\nGenerating workflow (this may take a minute)...");
    let response = llm.generate(req)?;

    // Extract YAML from response (may be wrapped in markdown code blocks)
    let yaml_content = extract_yaml(&response)?;

    // Validate the generated YAML
    println!("Validating generated workflow...");
    let _cfg =
        WorkflowConfig::from_yaml(&yaml_content).context("Generated workflow is not valid YAML")?;

    // Write the workflow
    let output_path = project_dir.join("workflow.yaml");
    std::fs::write(&output_path, &yaml_content)
        .with_context(|| format!("write {}", output_path.display()))?;

    println!("\nGenerated: {}", output_path.display());
    println!("\nNext steps:");
    println!("  1. Review the generated workflow.yaml");
    println!("  2. Customize variables as needed");
    println!(
        "  3. Run: vwf run workflow.yaml --workdir {} --dry-run",
        project_dir.display()
    );

    Ok(())
}

/// Find the llm-context directory relative to the binary or repo root.
fn find_llm_context_dir() -> Result<std::path::PathBuf> {
    // Try relative to current working directory first
    let cwd_path = std::env::current_dir()?.join("docs/llm-context");
    if cwd_path.exists() {
        return Ok(cwd_path);
    }

    // Try common locations
    let candidates = [
        "/Users/mike/github/softwarewrighter/video_workflow_rs/docs/llm-context",
        "./docs/llm-context",
        "../docs/llm-context",
        "../../docs/llm-context",
    ];

    for candidate in candidates {
        let path = Path::new(candidate);
        if path.exists() && path.join("step-types.md").exists() {
            return Ok(path.to_path_buf());
        }
    }

    bail!(
        "Could not find docs/llm-context directory.\n\
         Use --context-dir to specify the path to the LLM context documents."
    )
}

/// Load all markdown files from the context directory.
fn load_context_documents(ctx_dir: &Path) -> Result<String> {
    let mut content = String::new();

    // Priority order for context files
    let files = [
        "workflow-structure.md",
        "step-types.md",
        "service-catalog.md",
        "best-practices.md",
    ];

    for filename in files {
        let path = ctx_dir.join(filename);
        if path.exists() {
            let text = std::fs::read_to_string(&path)
                .with_context(|| format!("read {}", path.display()))?;
            content.push_str(&format!("\n\n# {} #\n\n{}", filename, text));
        }
    }

    // Also load example workflows
    let examples_dir = ctx_dir.join("examples");
    if examples_dir.exists() {
        for entry in std::fs::read_dir(&examples_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "yaml").unwrap_or(false) {
                let text = std::fs::read_to_string(&path)
                    .with_context(|| format!("read {}", path.display()))?;
                let filename = path.file_name().unwrap().to_string_lossy();
                content.push_str(&format!(
                    "\n\n# Example: {} #\n\n```yaml\n{}\n```",
                    filename, text
                ));
            }
        }
    }

    Ok(content)
}

fn build_system_prompt(context: &str) -> String {
    format!(
        r#"You are an expert video workflow engineer. Your task is to generate VWF (Video Workflow Framework) workflow.yaml files.

IMPORTANT RULES:
1. Output ONLY valid YAML - no explanations, no markdown except for the yaml code block
2. Every step must have a unique `id`
3. Use `resume_output` for expensive generation steps (TTS, image, video)
4. Follow the segment structure for proper audio handling:
   - music_only: No narration allowed (title/outro)
   - narration_only: No music allowed (main content)
   - mixed: Both allowed with proper volume mixing
5. Use phonetic spelling for acronyms in narration scripts:
   - VWF → "Vee Double-You Eff"
   - TTS → "text to speech"
   - API → "A-P-I"
6. Standard volume levels:
   - Narration: -25 dB
   - Background music: -32 dB

REFERENCE DOCUMENTATION:
{context}

Generate a complete, working workflow.yaml file."#
    )
}

fn build_user_prompt(brief: &str, project_dir: &Path) -> String {
    let project_name = project_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "MyProject".to_string());

    format!(
        r#"Generate a workflow.yaml for the following project:

PROJECT NAME: {project_name}
PROJECT DIRECTORY: {dir}

BRIEF:
{brief}

Generate a complete workflow.yaml file. Include:
1. Appropriate vars section with server URLs and voice reference
2. Segments for proper audio organization
3. All necessary steps: ensure_dirs, script writing, TTS, image generation, video assembly
4. Volume normalization for all clips
5. Final video concatenation

Output ONLY the workflow.yaml content wrapped in ```yaml code blocks."#,
        dir = project_dir.display()
    )
}

/// Extract YAML content from LLM response, handling markdown code blocks.
fn extract_yaml(response: &str) -> Result<String> {
    // Look for yaml code block
    if let Some(start) = response.find("```yaml") {
        let content_start = start + 7; // Skip "```yaml"
        if let Some(end) = response[content_start..].find("```") {
            let yaml = response[content_start..content_start + end].trim();
            return Ok(yaml.to_string());
        }
    }

    // Look for generic code block
    if let Some(start) = response.find("```") {
        let content_start = start + 3;
        // Skip language identifier if present
        let line_end = response[content_start..].find('\n').unwrap_or(0);
        let actual_start = content_start + line_end + 1;
        if let Some(end) = response[actual_start..].find("```") {
            let yaml = response[actual_start..actual_start + end].trim();
            return Ok(yaml.to_string());
        }
    }

    // Assume the whole response is YAML if no code blocks found
    if response.trim().starts_with("version:") {
        return Ok(response.trim().to_string());
    }

    bail!(
        "Could not extract YAML from LLM response. Response started with: {}...",
        &response.chars().take(100).collect::<String>()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_yaml_from_code_block() {
        let response = r#"Here's your workflow:

```yaml
version: 1
name: "Test"
steps: []
```

Hope this helps!"#;

        let yaml = extract_yaml(response).unwrap();
        assert!(yaml.starts_with("version: 1"));
        assert!(yaml.contains("name: \"Test\""));
    }

    #[test]
    fn extracts_yaml_without_code_block() {
        let response = "version: 1\nname: \"Test\"\nsteps: []";
        let yaml = extract_yaml(response).unwrap();
        assert!(yaml.starts_with("version: 1"));
    }
}

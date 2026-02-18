//! LLM-driven workflow generation command.
//!
//! Uses structured prompts to ask the LLM for individual decisions,
//! then constructs valid YAML programmatically. The LLM provides
//! creative content; the code ensures valid structure.

use std::path::Path;

use anyhow::{bail, Context, Result};

use vwf_core::{LlmClient, LlmReq, OllamaClient};

/// A segment identified from the brief.
#[derive(Debug, Clone)]
struct SegmentPlan {
    id: String,
    title: String,
    segment_type: String, // music_only, narration_only, mixed
    narration: Option<String>,
    visual_style: String, // diagram, animation, static, title_card
    duration_hint: String,
}

/// Generate a workflow from a project brief using structured LLM queries.
pub fn generate(project_dir: &Path, model: &str, _context_dir: Option<&Path>) -> Result<()> {
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

    let project_name = project_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "MyProject".to_string());

    println!("Generating workflow for: {}", project_dir.display());
    println!("  Brief: {} chars", brief.len());
    println!("  Model: {}", model);

    let mut llm = OllamaClient::new(model.to_string());

    // Step 1: Ask LLM to identify segments from the brief
    println!("\n[1/4] Identifying segments from brief...");
    let segments = identify_segments(&mut llm, &brief)?;
    println!("  Found {} segments", segments.len());

    // Step 2: For each segment, get the narration content
    println!("\n[2/4] Generating narration for each segment...");
    let segments = generate_narrations(&mut llm, &brief, segments)?;

    // Step 3: Determine visual style for each segment
    println!("\n[3/4] Determining visual styles...");
    let segments = determine_visuals(&mut llm, &brief, segments)?;

    // Step 4: Build the workflow YAML programmatically
    println!("\n[4/4] Building workflow YAML...");
    let yaml = build_workflow_yaml(&project_name, &segments);

    // Write the workflow
    let output_path = project_dir.join("workflow.yaml");
    std::fs::write(&output_path, &yaml)
        .with_context(|| format!("write {}", output_path.display()))?;

    println!("\nGenerated: {}", output_path.display());
    println!("  {} segments", segments.len());
    println!(
        "  {} narration segments",
        segments.iter().filter(|s| s.narration.is_some()).count()
    );

    println!("\nNext steps:");
    println!("  1. Review the generated workflow.yaml");
    println!("  2. Customize vars (voice_ref, server URLs) as needed");
    println!(
        "  3. Run: vwf run workflow.yaml --workdir {} --dry-run",
        project_dir.display()
    );

    Ok(())
}

/// Ask LLM to identify segments from the brief.
fn identify_segments(llm: &mut OllamaClient, brief: &str) -> Result<Vec<SegmentPlan>> {
    let prompt = format!(
        r#"Analyze this video brief and list each segment.

BRIEF:
{brief}

For each segment, output ONE LINE in this exact format:
SEGMENT|<id>|<title>|<type>|<duration>

Where:
- id: short lowercase identifier (e.g., "intro", "problem", "demo1")
- title: brief description (e.g., "Introduction and Hook")
- type: one of: music_only, narration_only, mixed
- duration: approximate duration (e.g., "30s", "2m")

Rules:
- Title/intro/outro segments should be "music_only"
- Main content segments should be "narration_only"
- Only use "mixed" if the brief explicitly calls for it

Output ONLY the segment lines, nothing else. Example:
SEGMENT|title|Title Screen|music_only|5s
SEGMENT|hook|Opening Hook|narration_only|30s
SEGMENT|outro|Closing|music_only|10s"#
    );

    let response = llm.generate(LlmReq {
        system: "You extract structured information from video briefs. Output only the requested format, no explanations.".to_string(),
        user: prompt,
        provider: format!("ollama:{}", llm.model),
    })?;

    let mut segments = Vec::new();
    for line in response.lines() {
        let line = line.trim();
        if !line.starts_with("SEGMENT|") {
            continue;
        }
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 5 {
            segments.push(SegmentPlan {
                id: parts[1].to_string(),
                title: parts[2].to_string(),
                segment_type: normalize_segment_type(parts[3]),
                narration: None,
                visual_style: "static".to_string(),
                duration_hint: parts[4].to_string(),
            });
        }
    }

    if segments.is_empty() {
        bail!("LLM did not identify any segments from the brief");
    }

    Ok(segments)
}

fn normalize_segment_type(s: &str) -> String {
    match s.trim().to_lowercase().as_str() {
        "music_only" | "music" => "music_only".to_string(),
        "narration_only" | "narration" => "narration_only".to_string(),
        "mixed" => "mixed".to_string(),
        _ => "narration_only".to_string(),
    }
}

/// Generate narration content for each narration segment.
fn generate_narrations(
    llm: &mut OllamaClient,
    brief: &str,
    mut segments: Vec<SegmentPlan>,
) -> Result<Vec<SegmentPlan>> {
    for segment in &mut segments {
        if segment.segment_type == "music_only" {
            println!("  [{}] Skipping (music only)", segment.id);
            continue;
        }

        println!("  [{}] Generating narration...", segment.id);

        let prompt = format!(
            r#"Write narration for segment "{}" of this video.

BRIEF:
{brief}

SEGMENT: {title}
DURATION HINT: {duration}

RULES:
1. Write natural, conversational speech
2. Spell out acronyms phonetically:
   - VWF → "Vee Double-You Eff"
   - API → "A-P-I"
   - CLI → "command line"
   - TTS → "text-to-speech"
   - LLM → "large language model"
   - YAML → "yaml"
   - ffmpeg → "eff eff em peg"
3. Keep sentences short and punchy
4. Match the duration hint (~150 words per minute)
5. Do NOT include stage directions or [brackets]

Output ONLY the narration text, nothing else."#,
            segment.id,
            title = segment.title,
            duration = segment.duration_hint,
        );

        let response = llm.generate(LlmReq {
            system: "You write professional narration scripts for explainer videos. Output only the narration text.".to_string(),
            user: prompt,
            provider: format!("ollama:{}", llm.model),
        })?;

        segment.narration = Some(response.trim().to_string());
    }

    Ok(segments)
}

/// Determine visual style for each segment.
fn determine_visuals(
    llm: &mut OllamaClient,
    brief: &str,
    mut segments: Vec<SegmentPlan>,
) -> Result<Vec<SegmentPlan>> {
    let segment_list: String = segments
        .iter()
        .map(|s| format!("- {}: {}", s.id, s.title))
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        r#"For each video segment, choose an appropriate visual style.

BRIEF:
{brief}

SEGMENTS:
{segment_list}

For each segment, output ONE LINE in this format:
VISUAL|<segment_id>|<style>

Where style is one of:
- title_card: For title screens, outros (use with music_only segments)
- diagram: For explaining architecture, concepts, structure
- animation: For dynamic content, demos, processes
- static: For narration over a simple background

Output ONLY the visual lines, nothing else."#
    );

    let response = llm.generate(LlmReq {
        system: "You are a video production expert. Output only the requested format.".to_string(),
        user: prompt,
        provider: format!("ollama:{}", llm.model),
    })?;

    // Parse visual assignments
    for line in response.lines() {
        let line = line.trim();
        if !line.starts_with("VISUAL|") {
            continue;
        }
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 3 {
            let segment_id = parts[1].trim();
            let style = parts[2].trim();
            if let Some(seg) = segments.iter_mut().find(|s| s.id == segment_id) {
                seg.visual_style = normalize_visual_style(style);
                println!("  [{}] {}", seg.id, seg.visual_style);
            }
        }
    }

    Ok(segments)
}

fn normalize_visual_style(s: &str) -> String {
    match s.trim().to_lowercase().as_str() {
        "title_card" | "title" => "title_card".to_string(),
        "diagram" | "architecture" => "diagram".to_string(),
        "animation" | "animated" | "dynamic" => "animation".to_string(),
        _ => "static".to_string(),
    }
}

/// Build the workflow YAML programmatically from segment plans.
fn build_workflow_yaml(project_name: &str, segments: &[SegmentPlan]) -> String {
    let mut yaml = String::new();

    // Header
    yaml.push_str(&format!(
        r#"version: 1
name: "{project_name}"

vars:
  project_name: "{project_name}"
  tts_server: "http://curiosity:7860"
  tti_server: "http://192.168.1.64:8570"
  itv_server: "http://192.168.1.64:8100"
  ttv_server: "http://192.168.1.64:6000"
  ollama_server: "http://localhost:11434"
  voice_ref: "assets/voice-reference.wav"
  voice_ref_text: "This is a sample of my voice for cloning purposes."
  music_file: "assets/background-music.wav"

segments:
"#
    ));

    // Segments section
    for seg in segments {
        let step_ref = if seg.segment_type == "music_only" {
            format!("img_{}", seg.id)
        } else {
            format!("script_{}", seg.id)
        };
        yaml.push_str(&format!(
            r#"  - id: {id}
    segment_type: {seg_type}
    description: "{title}"
    steps:
      - {step_ref}
"#,
            id = seg.id,
            seg_type = seg.segment_type,
            title = seg.title,
            step_ref = step_ref
        ));
    }

    yaml.push_str("\nsteps:\n");

    // Directory setup
    yaml.push_str(
        r#"  # ========== Directory Setup ==========
  - id: setup_dirs
    kind: ensure_dirs
    dirs:
      - "work/scripts"
      - "work/audio"
      - "work/images"
      - "work/videos"
      - "work/clips"
      - "work/reports"
      - "output"

"#,
    );

    // Script writing steps
    yaml.push_str("  # ========== Scripts ==========\n");
    for (i, seg) in segments.iter().enumerate() {
        if let Some(ref narration) = seg.narration {
            yaml.push_str(&format!(
                "  - id: script_{id}\n    kind: write_file\n    path: \"work/scripts/{i:02}-{id}.txt\"\n    content: |\n{content}\n\n",
                id = seg.id,
                i = i,
                content = indent_content(narration, 6)
            ));
        }
    }

    // TTS steps for narration segments
    yaml.push_str("  # ========== TTS Generation ==========\n");
    for (i, seg) in segments.iter().enumerate() {
        if seg.narration.is_some() {
            yaml.push_str(&format!(
                r#"  - id: tts_{id}
    kind: tts_generate
    resume_output: "work/audio/{i:02}-{id}.wav"
    script_path: "work/scripts/{i:02}-{id}.txt"
    output_path: "work/audio/{i:02}-{id}.wav"
    server: "{{{{tts_server}}}}"
    reference_audio: "{{{{voice_ref}}}}"
    reference_text: "{{{{voice_ref_text}}}}"

"#,
                id = seg.id,
                i = i
            ));
        }
    }

    // Visual generation based on style
    yaml.push_str("  # ========== Visual Generation ==========\n");
    for (i, seg) in segments.iter().enumerate() {
        match seg.visual_style.as_str() {
            "title_card" => {
                yaml.push_str(&format!(
                    r#"  - id: img_{id}
    kind: create_slide
    resume_output: "work/images/{i:02}-{id}.png"
    template: title
    text: "{title}"
    output_path: "work/images/{i:02}-{id}.png"
    orientation: landscape

"#,
                    id = seg.id,
                    i = i,
                    title = seg.title
                ));
            }
            "diagram" | "animation" => {
                yaml.push_str(&format!(
                    r#"  - id: img_{id}
    kind: text_to_image
    resume_output: "work/images/{i:02}-{id}.png"
    prompt: "Professional diagram showing {title_lower}, dark theme, clean modern design, high contrast"
    output_path: "work/images/{i:02}-{id}.png"
    orientation: landscape
    seed: {seed}
    server: "{{{{tti_server}}}}"

"#,
                    id = seg.id,
                    i = i,
                    title_lower = seg.title.to_lowercase(),
                    seed = 2026 + i
                ));
            }
            _ => {
                // Static - simple slide
                yaml.push_str(&format!(
                    r##"  - id: img_{id}
    kind: create_slide
    resume_output: "work/images/{i:02}-{id}.png"
    template: title
    text: "{title}"
    background_color: "#1a1a2e"
    output_path: "work/images/{i:02}-{id}.png"
    orientation: landscape

"##,
                    id = seg.id,
                    i = i,
                    title = seg.title
                ));
            }
        }
    }

    // Video clip creation
    yaml.push_str("  # ========== Clip Assembly ==========\n");
    for (i, seg) in segments.iter().enumerate() {
        if seg.segment_type == "music_only" {
            // Music-only clip
            yaml.push_str(&format!(
                r#"  - id: clip_{id}
    kind: run_command
    program: ffmpeg
    args:
      - "-y"
      - "-loop"
      - "1"
      - "-i"
      - "work/images/{i:02}-{id}.png"
      - "-i"
      - "{{{{music_file}}}}"
      - "-filter_complex"
      - "[1:a]volume=-28dB,afade=t=out:st=4:d=1[aout]"
      - "-map"
      - "0:v"
      - "-map"
      - "[aout]"
      - "-c:v"
      - "libx264"
      - "-tune"
      - "stillimage"
      - "-c:a"
      - "aac"
      - "-pix_fmt"
      - "yuv420p"
      - "-t"
      - "5"
      - "work/clips/{i:02}-{id}.mp4"

"#,
                id = seg.id,
                i = i
            ));
        } else {
            // Narration clip
            yaml.push_str(&format!(
                r#"  - id: clip_{id}
    kind: run_command
    program: ffmpeg
    args:
      - "-y"
      - "-loop"
      - "1"
      - "-i"
      - "work/images/{i:02}-{id}.png"
      - "-i"
      - "work/audio/{i:02}-{id}.wav"
      - "-c:v"
      - "libx264"
      - "-tune"
      - "stillimage"
      - "-c:a"
      - "aac"
      - "-b:a"
      - "192k"
      - "-pix_fmt"
      - "yuv420p"
      - "-shortest"
      - "work/clips/{i:02}-{id}.mp4"

"#,
                id = seg.id,
                i = i
            ));
        }
    }

    // Final concatenation
    yaml.push_str("  # ========== Final Assembly ==========\n");
    yaml.push_str("  - id: final_concat\n    kind: video_concat\n    clips:\n");
    for (i, seg) in segments.iter().enumerate() {
        yaml.push_str(&format!(
            "      - \"work/clips/{i:02}-{id}.mp4\"\n",
            i = i,
            id = seg.id
        ));
    }
    yaml.push_str("    output_path: \"output/final.mp4\"\n    reencode: true\n\n");

    // Audit step
    yaml.push_str(
        r#"  # ========== Quality Audit ==========
  - id: audit_output
    kind: llm_audit
    assets:
      - "output/final.mp4"
    audit_prompt: |
      Analyze this frame for quality issues:
      - Is the image clear and not corrupted?
      - Are there any visual artifacts or glitches?
      Describe any issues found. Say "OK" if the frame looks good.
    output_path: "work/reports/audit.json"
    model: llava
    frame_count: 5
    fail_on_issues: false
    server: "{{ollama_server}}"
"#,
    );

    yaml
}

fn indent_content(content: &str, spaces: usize) -> String {
    let indent = " ".repeat(spaces);
    content
        .lines()
        .map(|line| format!("{}{}", indent, line))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_segment_type() {
        assert_eq!(normalize_segment_type("music_only"), "music_only");
        assert_eq!(normalize_segment_type("MUSIC"), "music_only");
        assert_eq!(normalize_segment_type("narration"), "narration_only");
        assert_eq!(normalize_segment_type("unknown"), "narration_only");
    }

    #[test]
    fn test_normalize_visual_style() {
        assert_eq!(normalize_visual_style("title_card"), "title_card");
        assert_eq!(normalize_visual_style("DIAGRAM"), "diagram");
        assert_eq!(normalize_visual_style("animated"), "animation");
        assert_eq!(normalize_visual_style("unknown"), "static");
    }

    #[test]
    fn test_indent_content() {
        let content = "line 1\nline 2";
        let indented = indent_content(content, 4);
        assert_eq!(indented, "    line 1\n    line 2");
    }
}

//! Step handlers for workflow execution.

mod audio_mix;
mod context;
mod create_slide;
mod ensure_dirs;
mod image_to_video;
mod llm_audit;
mod llm_generate;
mod normalize_volume;
mod run_command;
mod split_sections;
mod text_to_image;
mod text_to_video;
mod tts_generate;
mod video_concat;
mod whisper_transcribe;
mod write_file;

use anyhow::Result;
use std::collections::BTreeMap;

use context::StepCtx;
use vwf_config::{StepConfig, StepKind};
use vwf_runtime::Runtime;

/// Execute a single step with dependency injection via Runtime trait.
pub fn execute_step(
    rt: &mut dyn Runtime,
    vars: &BTreeMap<String, String>,
    step: &StepConfig,
) -> Result<()> {
    let mut ctx = StepCtx::new(rt, vars, &step.id);
    dispatch(&mut ctx, &step.kind, &step.payload)
}

fn dispatch(ctx: &mut StepCtx<'_>, kind: &StepKind, payload: &serde_json::Value) -> Result<()> {
    match kind {
        StepKind::EnsureDirs => ensure_dirs::execute(ctx, payload),
        StepKind::WriteFile => write_file::execute(ctx, payload),
        StepKind::SplitSections => split_sections::execute(ctx, payload),
        StepKind::RunCommand => run_command::execute(ctx, payload),
        StepKind::LlmGenerate => llm_generate::execute(ctx, payload),
        StepKind::TtsGenerate => tts_generate::execute(ctx, payload),
        StepKind::TextToImage => text_to_image::execute(ctx, payload),
        StepKind::ImageToVideo => image_to_video::execute(ctx, payload),
        StepKind::TextToVideo => text_to_video::execute(ctx, payload),
        StepKind::NormalizeVolume => normalize_volume::execute(ctx, payload),
        StepKind::WhisperTranscribe => whisper_transcribe::execute(ctx, payload),
        StepKind::VideoConcat => video_concat::execute(ctx, payload),
        StepKind::AudioMix => audio_mix::execute(ctx, payload),
        StepKind::CreateSlide => create_slide::execute(ctx, payload),
        StepKind::LlmAudit => llm_audit::execute(ctx, payload),
    }
}

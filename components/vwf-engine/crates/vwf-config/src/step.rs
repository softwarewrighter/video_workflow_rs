//! Step configuration types.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepConfig {
    pub id: String,
    pub kind: StepKind,
    /// Output path for resume checking (optional).
    /// Use a separate field name to avoid conflicts with step payload fields.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resume_output: Option<String>,
    #[serde(flatten)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepKind {
    EnsureDirs,
    WriteFile,
    SplitSections,
    RunCommand,
    LlmGenerate,
    TtsGenerate,
    TextToImage,
    ImageToVideo,
    TextToVideo,
    NormalizeVolume,
    WhisperTranscribe,
    VideoConcat,
    AudioMix,
    CreateSlide,
}

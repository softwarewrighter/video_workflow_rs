//! Step configuration types.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepConfig {
    pub id: String,
    pub kind: StepKind,
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
}

//! Report types for deserialization (WASM-compatible).
//!
//! These mirror the types in vwf-core but without the runtime dependencies
//! that don't work on WASM.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunReport {
    pub run_id: String,
    pub workflow_name: String,
    pub started_at: String,
    pub finished_at: String,
    pub steps: Vec<StepReport>,
    pub vars: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepReport {
    pub id: String,
    pub kind: String,
    pub status: StepStatus,
    pub started_at: String,
    pub finished_at: String,
    pub error: Option<String>,
    pub duration_ms: u128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Ok,
    Skipped,
    Failed,
    Blocked,
}

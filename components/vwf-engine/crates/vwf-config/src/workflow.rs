//! Workflow configuration.

use std::collections::{BTreeMap, HashSet};

use anyhow::bail;
use serde::{Deserialize, Serialize};

use super::StepConfig;

/// Segment type determines what audio content is allowed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SegmentType {
    /// Music only - no narration allowed (e.g., intro/outro)
    MusicOnly,
    /// Narration only - no music allowed (main content)
    NarrationOnly,
    /// Both music and narration allowed
    Mixed,
}

impl Default for SegmentType {
    fn default() -> Self {
        Self::Mixed
    }
}

/// A semantic grouping of workflow steps.
///
/// Segments help organize video production by preventing invalid
/// audio combinations (e.g., music overlapping narration where
/// it shouldn't).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    /// Unique identifier for this segment
    pub id: String,
    /// Segment type determines allowed audio content
    #[serde(default)]
    pub segment_type: SegmentType,
    /// Description of this segment's purpose
    #[serde(default)]
    pub description: Option<String>,
    /// Step IDs that belong to this segment
    pub steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    pub version: u32,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub vars: BTreeMap<String, String>,
    pub steps: Vec<StepConfig>,
    /// Optional semantic segments for organizing steps
    #[serde(default)]
    pub segments: Vec<Segment>,
}

impl WorkflowConfig {
    pub fn from_yaml(yaml: &str) -> anyhow::Result<Self> {
        let cfg: Self = serde_yaml::from_str(yaml)
            .map_err(|e| anyhow::anyhow!("Failed to parse workflow YAML: {e}"))?;
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        // Validate step IDs
        let mut seen_ids = HashSet::new();
        for step in &self.steps {
            if step.id.is_empty() {
                bail!("Step id cannot be empty");
            }
            if !seen_ids.insert(&step.id) {
                bail!("Duplicate step id: `{}`", step.id);
            }
        }

        // Validate segments if present
        let step_ids: HashSet<_> = self.steps.iter().map(|s| s.id.as_str()).collect();
        let mut seen_segment_ids = HashSet::new();

        for segment in &self.segments {
            if segment.id.is_empty() {
                bail!("Segment id cannot be empty");
            }
            if !seen_segment_ids.insert(&segment.id) {
                bail!("Duplicate segment id: `{}`", segment.id);
            }
            // Validate that all referenced steps exist
            for step_ref in &segment.steps {
                if !step_ids.contains(step_ref.as_str()) {
                    bail!(
                        "Segment `{}` references unknown step: `{}`",
                        segment.id,
                        step_ref
                    );
                }
            }
        }

        Ok(())
    }

    /// Get the segment containing a given step ID, if any.
    pub fn segment_for_step(&self, step_id: &str) -> Option<&Segment> {
        self.segments
            .iter()
            .find(|s| s.steps.iter().any(|id| id == step_id))
    }
}

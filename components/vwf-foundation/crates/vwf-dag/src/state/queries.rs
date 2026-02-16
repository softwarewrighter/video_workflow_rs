//! Workflow state query methods.

use anyhow::{Context, Result};
use chrono::Utc;

use crate::{Task, TaskStatus};

use super::WorkflowState;

impl WorkflowState {
    pub fn ready_tasks(&self) -> Vec<&Task> {
        self.tasks.values().filter(|t| t.is_ready()).collect()
    }

    pub fn blocked_tasks(&self) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|t| matches!(t.status, TaskStatus::Blocked { .. }))
            .collect()
    }

    pub fn checkpoint_pending(&self, name: &str) -> bool {
        self.checkpoints.get(name).is_some_and(|c| !c.approved)
    }

    pub fn approve_checkpoint(&mut self, name: &str) -> Result<()> {
        let cp = self.checkpoints.get_mut(name).context(format!("Checkpoint not found: {name}"))?;
        cp.approved = true;
        cp.approved_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn progress(&self) -> f64 {
        if self.tasks.is_empty() {
            return 100.0;
        }
        let completed = self.tasks.values().filter(|t| t.is_complete()).count();
        (completed as f64 / self.tasks.len() as f64) * 100.0
    }
}

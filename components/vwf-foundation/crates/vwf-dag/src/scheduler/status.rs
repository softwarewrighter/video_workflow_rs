//! Task status update functions.

use std::collections::{HashMap, HashSet};

use crate::{ArtifactStatus, InputSpec, Task, TaskId, TaskStatus, WorkflowState};

/// Build set of available artifact IDs (Ready or Placeholder).
pub fn build_available_set(state: &WorkflowState) -> HashSet<String> {
    state
        .artifacts
        .values()
        .filter(|a| matches!(a.status, ArtifactStatus::Ready | ArtifactStatus::Placeholder))
        .map(|a| a.id.clone())
        .collect()
}

/// Build map from artifact ID to producing task ID.
pub fn build_producer_map(state: &WorkflowState) -> HashMap<String, TaskId> {
    state
        .tasks
        .values()
        .flat_map(|task| {
            task.outputs
                .iter()
                .map(|o| (o.artifact.clone(), task.id.clone()))
        })
        .collect()
}

/// Check a task's inputs for availability.
pub fn check_task_inputs(
    task: &Task,
    available: &HashSet<String>,
    producers: &HashMap<String, TaskId>,
) -> (bool, Vec<String>) {
    let mut waiting_on = Vec::new();
    let mut can_run = true;

    for input in &task.inputs {
        if let InputSpec::Required { artifact } = input {
            if !available.contains(artifact) {
                can_run = false;
                add_waiting_on(&mut waiting_on, artifact, producers);
            }
        }
    }
    (can_run, waiting_on)
}

fn add_waiting_on(
    waiting_on: &mut Vec<String>,
    artifact: &str,
    producers: &HashMap<String, TaskId>,
) {
    if let Some(producer) = producers.get(artifact) {
        waiting_on.push(producer.clone());
    } else {
        waiting_on.push(format!("artifact:{artifact}"));
    }
}

/// Check if task should be skipped during status update.
pub fn should_skip(task: &Task) -> bool {
    task.is_complete()
        || matches!(
            task.status,
            TaskStatus::Running | TaskStatus::Skipped { .. } | TaskStatus::Failed { .. }
        )
}

//! Artifact invalidation cascade functions.

use std::collections::HashSet;

use crate::{ArtifactStatus, InputSpec, Task, TaskId, TaskStatus, WorkflowState};

/// Collect all artifacts and tasks affected by invalidation.
pub fn collect_targets(
    state: &WorkflowState,
    changed_artifact: &str,
) -> (HashSet<String>, Vec<TaskId>) {
    let mut to_invalidate = vec![changed_artifact.to_string()];
    let mut invalidated = HashSet::new();
    let mut tasks_to_reset = Vec::new();

    while let Some(artifact_id) = to_invalidate.pop() {
        if invalidated.contains(&artifact_id) {
            continue;
        }
        invalidated.insert(artifact_id.clone());
        let (outputs, tasks) = find_downstream(state, &artifact_id, &invalidated);
        to_invalidate.extend(outputs);
        tasks_to_reset.extend(tasks);
    }
    (invalidated, tasks_to_reset)
}

/// Check if a task consumes a specific artifact.
fn task_consumes(task: &Task, artifact_id: &str) -> bool {
    task.inputs.iter().any(|input| match input {
        InputSpec::Required { artifact } => artifact == artifact_id,
        InputSpec::Optional { artifact, .. } => artifact == artifact_id,
        InputSpec::Placeholder { artifact, .. } => artifact == artifact_id,
    })
}

/// Find downstream artifacts and completed tasks for an artifact.
fn find_downstream(
    state: &WorkflowState,
    artifact_id: &str,
    invalidated: &HashSet<String>,
) -> (Vec<String>, Vec<TaskId>) {
    let mut outputs = Vec::new();
    let mut tasks_to_reset = Vec::new();

    for task in state.tasks.values() {
        if task_consumes(task, artifact_id) {
            collect_task_outputs(task, invalidated, &mut outputs);
            if task.is_complete() {
                tasks_to_reset.push(task.id.clone());
            }
        }
    }
    (outputs, tasks_to_reset)
}

fn collect_task_outputs(task: &Task, invalidated: &HashSet<String>, outputs: &mut Vec<String>) {
    for output in &task.outputs {
        if !invalidated.contains(&output.artifact) {
            outputs.push(output.artifact.clone());
        }
    }
}

/// Reset task statuses to Blocked.
pub fn reset_tasks(state: &mut WorkflowState, task_ids: Vec<TaskId>) {
    for task_id in task_ids {
        if let Some(task) = state.tasks.get_mut(&task_id) {
            task.status = TaskStatus::Blocked { waiting_on: vec![] };
        }
    }
}

/// Mark artifacts as invalidated.
pub fn invalidate_artifacts(state: &mut WorkflowState, artifact_ids: HashSet<String>) {
    for artifact_id in artifact_ids {
        if let Some(artifact) = state.artifacts.get_mut(&artifact_id)
            && matches!(artifact.status, ArtifactStatus::Ready)
        {
            artifact.invalidate();
        }
    }
}

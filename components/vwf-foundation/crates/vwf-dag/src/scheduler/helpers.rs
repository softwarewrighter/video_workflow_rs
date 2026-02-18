//! Scheduler helper functions.

use std::collections::{HashMap, HashSet};

use crate::{ArtifactStatus, InputSpec, Task, TaskStatus, WorkflowState};

pub fn update_all_task_statuses(state: &mut WorkflowState) {
    let available: HashSet<_> = state
        .artifacts
        .values()
        .filter(|a| {
            matches!(
                a.status,
                ArtifactStatus::Ready | ArtifactStatus::Placeholder
            )
        })
        .map(|a| a.id.clone())
        .collect();
    let producers: HashMap<_, _> = state
        .tasks
        .values()
        .flat_map(|t| t.outputs.iter().map(|o| (o.artifact.clone(), t.id.clone())))
        .collect();
    for task in state.tasks.values_mut() {
        if task.is_complete()
            || matches!(
                task.status,
                TaskStatus::Running | TaskStatus::Skipped { .. } | TaskStatus::Failed { .. }
            )
        {
            continue;
        }
        let waiting: Vec<_> = task
            .inputs
            .iter()
            .filter_map(|i| {
                if let InputSpec::Required { artifact } = i {
                    if !available.contains(artifact) {
                        return Some(
                            producers
                                .get(artifact)
                                .cloned()
                                .unwrap_or_else(|| format!("artifact:{artifact}")),
                        );
                    }
                }
                None
            })
            .collect();
        task.status = if waiting.is_empty() {
            TaskStatus::Ready
        } else {
            TaskStatus::Blocked {
                waiting_on: waiting,
            }
        };
    }
}

pub fn collect_invalidation_targets(state: &WorkflowState, changed: &str) -> HashSet<String> {
    let mut to_process = vec![changed.to_string()];
    let mut invalidated = HashSet::new();
    while let Some(artifact_id) = to_process.pop() {
        if invalidated.insert(artifact_id.clone()) {
            for task in state.tasks.values() {
                if task.is_complete() && task_consumes(&task, &artifact_id) {
                    for o in &task.outputs {
                        if !invalidated.contains(&o.artifact) {
                            to_process.push(o.artifact.clone());
                        }
                    }
                }
            }
        }
    }
    invalidated
}

pub fn apply_invalidations(state: &mut WorkflowState, invalidated: HashSet<String>) {
    for id in &invalidated {
        if let Some(a) = state.artifacts.get_mut(id) {
            if matches!(a.status, ArtifactStatus::Ready) {
                a.invalidate();
            }
        }
    }
    for task in state.tasks.values_mut() {
        if task.is_complete()
            && task.inputs.iter().any(|i| match i {
                InputSpec::Required { artifact }
                | InputSpec::Optional { artifact, .. }
                | InputSpec::Placeholder { artifact, .. } => invalidated.contains(artifact),
            })
        {
            task.status = TaskStatus::Blocked { waiting_on: vec![] };
        }
    }
}

fn task_consumes(task: &Task, artifact_id: &str) -> bool {
    task.inputs.iter().any(|i| match i {
        InputSpec::Required { artifact }
        | InputSpec::Optional { artifact, .. }
        | InputSpec::Placeholder { artifact, .. } => artifact == artifact_id,
    })
}

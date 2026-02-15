//! DAG scheduler with constraint enforcement.

use std::collections::{BTreeSet, HashSet};

use super::{ArtifactStatus, InputSpec, Task, TaskId, TaskStatus, WorkflowState};

/// Events emitted by the scheduler.
#[derive(Debug, Clone)]
pub enum SchedulerEvent {
    /// Task is now ready to run.
    TaskReady { task_id: TaskId },
    /// Task started running.
    TaskStarted { task_id: TaskId },
    /// Task completed successfully.
    TaskComplete { task_id: TaskId },
    /// Task failed.
    TaskFailed { task_id: TaskId, error: String },
    /// Checkpoint reached, waiting for approval.
    CheckpointReached { name: String },
    /// All tasks complete.
    WorkflowComplete,
    /// No tasks can run (blocked or waiting for checkpoint).
    WorkflowBlocked { reason: String },
}

/// Scheduler for DAG-based workflow execution.
pub struct Scheduler {
    /// Tasks currently running.
    running: BTreeSet<TaskId>,

    /// Sequential groups currently occupied.
    occupied_groups: BTreeSet<String>,

    /// Resources currently in use.
    occupied_resources: BTreeSet<String>,
}

impl Scheduler {
    /// Create a new scheduler.
    pub fn new() -> Self {
        Self {
            running: BTreeSet::new(),
            occupied_groups: BTreeSet::new(),
            occupied_resources: BTreeSet::new(),
        }
    }

    /// Update task statuses based on artifact availability.
    pub fn update_task_statuses(&self, state: &mut WorkflowState) {
        // Build artifact availability map
        let available: HashSet<String> = state
            .artifacts
            .values()
            .filter(|a| matches!(a.status, ArtifactStatus::Ready | ArtifactStatus::Placeholder))
            .map(|a| a.id.clone())
            .collect();

        // Build producer map: artifact -> producing task
        let producers: std::collections::HashMap<String, TaskId> = state
            .tasks
            .values()
            .flat_map(|task| {
                task.outputs
                    .iter()
                    .map(|o| (o.artifact.clone(), task.id.clone()))
            })
            .collect();

        // Update each task
        for task in state.tasks.values_mut() {
            if task.is_complete()
                || matches!(
                    task.status,
                    TaskStatus::Running | TaskStatus::Skipped { .. } | TaskStatus::Failed { .. }
                )
            {
                continue;
            }

            let mut waiting_on = Vec::new();
            let mut can_run = true;

            for input in &task.inputs {
                match input {
                    InputSpec::Required { artifact } => {
                        if !available.contains(artifact) {
                            can_run = false;
                            // Find which task produces this artifact
                            if let Some(producer) = producers.get(artifact) {
                                waiting_on.push(producer.clone());
                            } else {
                                waiting_on.push(format!("artifact:{artifact}"));
                            }
                        }
                    }
                    InputSpec::Optional { .. } => {
                        // Optional inputs don't block
                    }
                    InputSpec::Placeholder { artifact, .. } => {
                        // Placeholder inputs: we can run even if missing (with placeholder)
                        // but record we're waiting if not available
                        if !available.contains(artifact) {
                            // Create placeholder if needed - handled elsewhere
                        }
                    }
                }
            }

            if can_run && waiting_on.is_empty() {
                task.status = TaskStatus::Ready;
            } else {
                task.status = TaskStatus::Blocked { waiting_on };
            }
        }
    }

    /// Get tasks that can be started now, respecting constraints.
    pub fn get_runnable_tasks<'a>(&self, state: &'a WorkflowState) -> Vec<&'a Task> {
        state
            .tasks
            .values()
            .filter(|task| {
                if !task.is_ready() {
                    return false;
                }

                // Check sequential group constraint
                if let Some(group) = &task.constraints.sequential_group
                    && self.occupied_groups.contains(group)
                {
                    return false;
                }

                // Check resource constraint
                if let Some(resource) = &task.constraints.resource
                    && self.occupied_resources.contains(resource)
                {
                    return false;
                }

                true
            })
            .collect()
    }

    /// Mark a task as started.
    pub fn start_task(&mut self, task: &Task) {
        self.running.insert(task.id.clone());

        if let Some(group) = &task.constraints.sequential_group {
            self.occupied_groups.insert(group.clone());
        }

        if let Some(resource) = &task.constraints.resource {
            self.occupied_resources.insert(resource.clone());
        }
    }

    /// Mark a task as finished (success or failure).
    pub fn finish_task(&mut self, task: &Task) {
        self.running.remove(&task.id);

        if let Some(group) = &task.constraints.sequential_group {
            self.occupied_groups.remove(group);
        }

        if let Some(resource) = &task.constraints.resource {
            self.occupied_resources.remove(resource);
        }
    }

    /// Check if any tasks are running.
    pub fn has_running_tasks(&self) -> bool {
        !self.running.is_empty()
    }

    /// Get count of running tasks.
    pub fn running_count(&self) -> usize {
        self.running.len()
    }

    /// Invalidate artifacts downstream of a changed artifact.
    pub fn invalidate_downstream(state: &mut WorkflowState, changed_artifact: &str) {
        let mut to_invalidate: Vec<String> = vec![changed_artifact.to_string()];
        let mut invalidated: HashSet<String> = HashSet::new();
        let mut tasks_to_reset: Vec<TaskId> = Vec::new();

        while let Some(artifact_id) = to_invalidate.pop() {
            if invalidated.contains(&artifact_id) {
                continue;
            }
            invalidated.insert(artifact_id.clone());

            // Find tasks that consume this artifact and collect their IDs
            for task in state.tasks.values() {
                let consumes = task.inputs.iter().any(|input| match input {
                    InputSpec::Required { artifact } => artifact == &artifact_id,
                    InputSpec::Optional { artifact, .. } => artifact == &artifact_id,
                    InputSpec::Placeholder { artifact, .. } => artifact == &artifact_id,
                });

                if consumes {
                    // Invalidate this task's outputs
                    for output in &task.outputs {
                        if !invalidated.contains(&output.artifact) {
                            to_invalidate.push(output.artifact.clone());
                        }
                    }

                    // Collect task ID for later update
                    if task.is_complete() {
                        tasks_to_reset.push(task.id.clone());
                    }
                }
            }
        }

        // Mark tasks as needing re-run
        for task_id in tasks_to_reset {
            if let Some(task) = state.tasks.get_mut(&task_id) {
                task.status = TaskStatus::Blocked { waiting_on: vec![] };
            }
        }

        // Mark artifacts as invalidated
        for artifact_id in invalidated {
            if let Some(artifact) = state.artifacts.get_mut(&artifact_id)
                && matches!(artifact.status, ArtifactStatus::Ready)
            {
                artifact.invalidate();
            }
        }
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dag::{Artifact, OutputSpec};

    fn create_test_state() -> WorkflowState {
        let mut state = WorkflowState::new("test", 1);

        // Task A produces artifact_a
        let mut task_a = Task::new("task_a", "test");
        task_a.outputs.push(OutputSpec {
            artifact: "artifact_a".to_string(),
            primary: true,
        });
        state.add_task(task_a);

        // Task B consumes artifact_a, produces artifact_b
        let mut task_b = Task::new("task_b", "test");
        task_b.inputs.push(InputSpec::Required {
            artifact: "artifact_a".to_string(),
        });
        task_b.outputs.push(OutputSpec {
            artifact: "artifact_b".to_string(),
            primary: true,
        });
        state.add_task(task_b);

        // Task C consumes artifact_b
        let mut task_c = Task::new("task_c", "test");
        task_c.inputs.push(InputSpec::Required {
            artifact: "artifact_b".to_string(),
        });
        state.add_task(task_c);

        state
    }

    #[test]
    fn task_status_updates() {
        let mut state = create_test_state();
        let scheduler = Scheduler::new();

        // Initially, only task_a should be ready (no inputs)
        scheduler.update_task_statuses(&mut state);

        let task_a = state.get_task("task_a").unwrap();
        assert!(task_a.is_ready(), "task_a should be ready");

        let task_b = state.get_task("task_b").unwrap();
        assert!(!task_b.is_ready(), "task_b should be blocked");

        // Mark artifact_a as ready
        let mut artifact_a = Artifact::missing("artifact_a");
        artifact_a.mark_ready("abc".to_string(), Some("task_a".to_string()));
        state.add_artifact(artifact_a);

        // Now task_b should be ready
        scheduler.update_task_statuses(&mut state);

        let task_b = state.get_task("task_b").unwrap();
        assert!(task_b.is_ready(), "task_b should be ready after artifact_a");
    }

    #[test]
    fn sequential_constraint() {
        let mut state = WorkflowState::new("test", 1);

        // Two TTS tasks in same sequential group
        let task1 = Task::new("tts_1", "tts").sequential("tts");
        let task2 = Task::new("tts_2", "tts").sequential("tts");

        state.add_task(task1);
        state.add_task(task2);

        // Mark both as ready
        state.get_task_mut("tts_1").unwrap().status = TaskStatus::Ready;
        state.get_task_mut("tts_2").unwrap().status = TaskStatus::Ready;

        let mut scheduler = Scheduler::new();

        // Both are ready and runnable initially
        let runnable = scheduler.get_runnable_tasks(&state);
        assert_eq!(runnable.len(), 2, "Both should be runnable initially");

        // Start task1 - this occupies the sequential group
        let task1 = state.get_task("tts_1").unwrap();
        scheduler.start_task(task1);
        // Mark task1 as Running (scheduler tracks constraints, caller tracks status)
        state.get_task_mut("tts_1").unwrap().status = TaskStatus::Running;

        // Now only task2 is Ready, but it's blocked by sequential constraint
        let runnable = scheduler.get_runnable_tasks(&state);
        assert_eq!(runnable.len(), 0, "task2 blocked by sequential constraint");

        // Finish task1 - this releases the sequential group
        let task1 = state.get_task("tts_1").unwrap();
        scheduler.finish_task(task1);
        state.get_task_mut("tts_1").unwrap().status = TaskStatus::Complete;

        // Now task2 can run
        let runnable = scheduler.get_runnable_tasks(&state);
        assert_eq!(runnable.len(), 1, "task2 runnable after task1 finishes");
        assert_eq!(runnable[0].id, "tts_2");
    }

    #[test]
    fn invalidation_cascade() {
        let mut state = create_test_state();

        // Mark all artifacts and tasks as complete
        let mut artifact_a = Artifact::missing("artifact_a");
        artifact_a.mark_ready("v1".to_string(), Some("task_a".to_string()));
        state.add_artifact(artifact_a);

        let mut artifact_b = Artifact::missing("artifact_b");
        artifact_b.mark_ready("v1".to_string(), Some("task_b".to_string()));
        state.add_artifact(artifact_b);

        state.get_task_mut("task_a").unwrap().status = TaskStatus::Complete;
        state.get_task_mut("task_b").unwrap().status = TaskStatus::Complete;
        state.get_task_mut("task_c").unwrap().status = TaskStatus::Complete;

        // Invalidate artifact_a (simulating a change)
        Scheduler::invalidate_downstream(&mut state, "artifact_a");

        // artifact_b should be invalidated (produced by task that consumes artifact_a)
        let artifact_b = state.get_artifact("artifact_b").unwrap();
        assert_eq!(artifact_b.status, ArtifactStatus::Invalidated);

        // task_b and task_c should need re-run
        let task_b = state.get_task("task_b").unwrap();
        assert!(!task_b.is_complete(), "task_b should need re-run");

        let task_c = state.get_task("task_c").unwrap();
        assert!(!task_c.is_complete(), "task_c should need re-run");
    }
}

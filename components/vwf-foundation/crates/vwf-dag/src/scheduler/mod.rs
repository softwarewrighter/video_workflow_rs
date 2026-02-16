//! DAG scheduler with constraint enforcement.

mod helpers;

use std::collections::{BTreeSet, HashSet};

use crate::{InputSpec, Task, TaskId, TaskStatus, WorkflowState};

/// Events emitted by the scheduler.
#[derive(Debug, Clone)]
pub enum SchedulerEvent {
    TaskReady { task_id: TaskId },
    TaskStarted { task_id: TaskId },
    TaskComplete { task_id: TaskId },
    TaskFailed { task_id: TaskId, error: String },
    CheckpointReached { name: String },
    WorkflowComplete,
    WorkflowBlocked { reason: String },
}

/// Scheduler for DAG-based workflow execution.
#[derive(Default)]
pub struct Scheduler {
    running: BTreeSet<TaskId>,
    occupied_groups: BTreeSet<String>,
    occupied_resources: BTreeSet<String>,
}

impl Scheduler {
    /// Update task statuses based on artifact availability.
    pub fn update_task_statuses(&self, state: &mut WorkflowState) {
        helpers::update_all_task_statuses(state);
    }

    /// Get tasks that can be started now, respecting constraints.
    pub fn get_runnable_tasks<'a>(&self, state: &'a WorkflowState) -> Vec<&'a Task> {
        state.tasks.values().filter(|t| t.is_ready() && !self.is_blocked(t)).collect()
    }

    /// Mark a task as started and reserve its resources.
    pub fn start_task(&mut self, task: &Task) {
        self.running.insert(task.id.clone());
        if let Some(g) = &task.constraints.sequential_group { self.occupied_groups.insert(g.clone()); }
        if let Some(r) = &task.constraints.resource { self.occupied_resources.insert(r.clone()); }
    }

    /// Mark a task as finished and release its resources.
    pub fn finish_task(&mut self, task: &Task) {
        self.running.remove(&task.id);
        if let Some(g) = &task.constraints.sequential_group { self.occupied_groups.remove(g); }
        if let Some(r) = &task.constraints.resource { self.occupied_resources.remove(r); }
    }

    /// Get number of currently running tasks.
    pub fn running_count(&self) -> usize { self.running.len() }

    /// Invalidate artifacts downstream of a changed artifact.
    pub fn invalidate_downstream(state: &mut WorkflowState, changed: &str) {
        let invalidated = helpers::collect_invalidation_targets(state, changed);
        helpers::apply_invalidations(state, invalidated);
    }

    fn is_blocked(&self, task: &Task) -> bool {
        task.constraints.sequential_group.as_ref().is_some_and(|g| self.occupied_groups.contains(g))
            || task.constraints.resource.as_ref().is_some_and(|r| self.occupied_resources.contains(r))
    }
}

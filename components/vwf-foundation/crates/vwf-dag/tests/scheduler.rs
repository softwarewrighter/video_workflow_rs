//! Scheduler integration tests.

use vwf_dag::{Artifact, InputSpec, OutputSpec, Scheduler, Task, TaskStatus, WorkflowState};

fn create_test_state() -> WorkflowState {
    let mut state = WorkflowState::new("test", 1);
    let mut task_a = Task::new("task_a", "test");
    task_a.outputs.push(OutputSpec { artifact: "artifact_a".to_string(), primary: true });
    state.add_task(task_a);

    let mut task_b = Task::new("task_b", "test");
    task_b.inputs.push(InputSpec::Required { artifact: "artifact_a".to_string() });
    task_b.outputs.push(OutputSpec { artifact: "artifact_b".to_string(), primary: true });
    state.add_task(task_b);

    let mut task_c = Task::new("task_c", "test");
    task_c.inputs.push(InputSpec::Required { artifact: "artifact_b".to_string() });
    state.add_task(task_c);
    state
}

#[test]
fn task_status_updates() {
    let mut state = create_test_state();
    let scheduler = Scheduler::default();
    scheduler.update_task_statuses(&mut state);
    assert!(state.get_task("task_a").unwrap().is_ready());
    assert!(!state.get_task("task_b").unwrap().is_ready());

    let mut artifact_a = Artifact::missing("artifact_a");
    artifact_a.mark_ready("abc".to_string(), Some("task_a".to_string()));
    state.add_artifact(artifact_a);
    scheduler.update_task_statuses(&mut state);
    assert!(state.get_task("task_b").unwrap().is_ready());
}

#[test]
fn sequential_constraint() {
    let mut state = WorkflowState::new("test", 1);
    let mut task1 = Task::new("tts_1", "tts");
    task1.constraints.sequential_group = Some("tts".to_string());
    state.add_task(task1);
    let mut task2 = Task::new("tts_2", "tts");
    task2.constraints.sequential_group = Some("tts".to_string());
    state.add_task(task2);
    state.get_task_mut("tts_1").unwrap().status = TaskStatus::Ready;
    state.get_task_mut("tts_2").unwrap().status = TaskStatus::Ready;

    let mut scheduler = Scheduler::default();
    assert_eq!(scheduler.get_runnable_tasks(&state).len(), 2);

    scheduler.start_task(state.get_task("tts_1").unwrap());
    state.get_task_mut("tts_1").unwrap().status = TaskStatus::Running;
    assert_eq!(scheduler.get_runnable_tasks(&state).len(), 0);

    scheduler.finish_task(state.get_task("tts_1").unwrap());
    state.get_task_mut("tts_1").unwrap().status = TaskStatus::Complete;
    let runnable = scheduler.get_runnable_tasks(&state);
    assert_eq!(runnable.len(), 1);
    assert_eq!(runnable[0].id, "tts_2");
}

#[test]
fn invalidation_cascade() {
    let mut state = create_test_state();
    let mut artifact_a = Artifact::missing("artifact_a");
    artifact_a.mark_ready("v1".to_string(), Some("task_a".to_string()));
    state.add_artifact(artifact_a);
    let mut artifact_b = Artifact::missing("artifact_b");
    artifact_b.mark_ready("v1".to_string(), Some("task_b".to_string()));
    state.add_artifact(artifact_b);

    state.get_task_mut("task_a").unwrap().status = TaskStatus::Complete;
    state.get_task_mut("task_b").unwrap().status = TaskStatus::Complete;
    state.get_task_mut("task_c").unwrap().status = TaskStatus::Complete;

    Scheduler::invalidate_downstream(&mut state, "artifact_a");
    assert_eq!(state.get_artifact("artifact_b").unwrap().status, vwf_dag::ArtifactStatus::Invalidated);
    assert!(!state.get_task("task_b").unwrap().is_complete());
    assert!(!state.get_task("task_c").unwrap().is_complete());
}

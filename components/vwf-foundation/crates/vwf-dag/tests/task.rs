//! Task unit tests.

use vwf_dag::{Task, TaskStatus};

#[test]
fn task_builder() {
    let mut task = Task::new("generate_tts", "tts_generate")
        .with_input("work/scripts/01.txt")
        .with_output("work/audio/01.wav");
    task.constraints.sequential_group = Some("tts".to_string());

    assert_eq!(task.id, "generate_tts");
    assert_eq!(task.kind, "tts_generate");
    assert_eq!(task.inputs.len(), 1);
    assert_eq!(task.outputs.len(), 1);
    assert_eq!(task.constraints.sequential_group, Some("tts".to_string()));
}

#[test]
fn task_status_transitions() {
    let mut task = Task::new("test", "test_kind");
    assert!(matches!(task.status, TaskStatus::Blocked { .. }));

    task.status = TaskStatus::Ready;
    assert!(task.is_ready());

    task.status = TaskStatus::Running;
    assert!(!task.is_ready());
    assert!(!task.is_complete());

    task.status = TaskStatus::Complete;
    assert!(task.is_complete());
}

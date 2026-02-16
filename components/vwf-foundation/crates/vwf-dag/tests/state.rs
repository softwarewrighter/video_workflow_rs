//! State persistence tests.

use chrono::Utc;
use tempfile::TempDir;
use vwf_dag::{StateStore, Task, WorkflowState};

#[test]
fn state_persistence() {
    let tmp = TempDir::new().unwrap();
    let store = StateStore::new(tmp.path());

    assert!(!store.exists());
    assert!(store.load().unwrap().is_none());

    let mut state = WorkflowState::new("test_workflow", 1);
    state.inputs.insert("episode".to_string(), serde_json::json!(17));
    state.add_task(Task::new("generate_text", "llm_generate"));

    store.save(&state).unwrap();
    assert!(store.exists());

    let loaded = store.load().unwrap().unwrap();
    assert_eq!(loaded.workflow_name, "test_workflow");
    assert_eq!(loaded.version, 1);
    assert!(loaded.tasks.contains_key("generate_text"));
}

#[test]
fn checkpoint_approval() {
    let mut state = WorkflowState::new("test", 1);

    state.checkpoints.insert(
        "review_text".to_string(),
        vwf_dag::CheckpointStatus {
            name: "review_text".to_string(),
            message: "Review the text files".to_string(),
            reached_at: Utc::now(),
            approved: false,
            approved_at: None,
        },
    );

    assert!(state.checkpoint_pending("review_text"));
    state.approve_checkpoint("review_text").unwrap();
    assert!(!state.checkpoint_pending("review_text"));
}

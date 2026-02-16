//! Artifact unit tests.

use vwf_dag::{Artifact, ArtifactStatus};

#[test]
fn artifact_lifecycle() {
    let mut artifact = Artifact::missing("work/output.txt");
    assert!(artifact.needs_rebuild());
    assert_eq!(artifact.status, ArtifactStatus::Missing);
    artifact.mark_ready("abc123".to_string(), Some("my_task".to_string()));
    assert!(!artifact.needs_rebuild());
    assert_eq!(artifact.status, ArtifactStatus::Ready);
    artifact.invalidate();
    assert!(artifact.needs_rebuild());
    assert_eq!(artifact.status, ArtifactStatus::Invalidated);
}

#[test]
fn placeholder_artifact() {
    let artifact = Artifact::placeholder("assets/background.jpg");
    assert!(artifact.is_placeholder);
    assert_eq!(artifact.status, ArtifactStatus::Placeholder);
    assert!(artifact.needs_rebuild());
}

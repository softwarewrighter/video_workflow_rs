//! Artifact tracking with checksums for incremental builds.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Unique identifier for an artifact.
pub type ArtifactId = String;

/// Status of an artifact.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactStatus {
    /// Artifact does not exist yet.
    Missing,
    /// Artifact is a placeholder (will be replaced when real arrives).
    Placeholder,
    /// Artifact exists and is valid.
    Ready,
    /// Artifact is invalidated (upstream changed, needs rebuild).
    Invalidated,
}

/// An artifact is a file produced or consumed by tasks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// Unique ID (typically the relative path).
    pub id: ArtifactId,

    /// Path relative to workdir.
    pub path: PathBuf,

    /// SHA256 checksum of content (None if missing).
    pub checksum: Option<String>,

    /// When the artifact was created/updated.
    pub created_at: Option<DateTime<Utc>>,

    /// Which task produced this artifact (None if external input).
    pub produced_by: Option<String>,

    /// Current status.
    pub status: ArtifactStatus,

    /// Is this a placeholder for a missing optional input?
    pub is_placeholder: bool,
}

impl Artifact {
    /// Create a new missing artifact.
    pub fn missing(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let id = path.to_string_lossy().to_string();
        Self {
            id,
            path,
            checksum: None,
            created_at: None,
            produced_by: None,
            status: ArtifactStatus::Missing,
            is_placeholder: false,
        }
    }

    /// Create a placeholder artifact.
    pub fn placeholder(path: impl Into<PathBuf>) -> Self {
        let mut artifact = Self::missing(path);
        artifact.status = ArtifactStatus::Placeholder;
        artifact.is_placeholder = true;
        artifact
    }

    /// Mark artifact as ready with checksum.
    pub fn mark_ready(&mut self, checksum: String, produced_by: Option<String>) {
        self.checksum = Some(checksum);
        self.created_at = Some(Utc::now());
        self.produced_by = produced_by;
        self.status = ArtifactStatus::Ready;
        self.is_placeholder = false;
    }

    /// Mark artifact as invalidated.
    pub fn invalidate(&mut self) {
        self.status = ArtifactStatus::Invalidated;
    }

    /// Check if artifact needs rebuild.
    pub fn needs_rebuild(&self) -> bool {
        matches!(
            self.status,
            ArtifactStatus::Missing | ArtifactStatus::Invalidated | ArtifactStatus::Placeholder
        )
    }
}

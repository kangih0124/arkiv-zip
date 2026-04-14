use std::collections::BTreeMap;

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use super::artifact::ArtifactBundle;
use super::file_entry::FileEntry;
use super::history_entry::HistoryEntry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileState {
    pub files: BTreeMap<String, FileEntry>,
}

impl FileState {
    pub fn empty() -> Self {
        Self {
            files: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectIndex {
    pub project: String,
    pub last_run: Option<DateTime<FixedOffset>>,
    pub current_state: FileState,
    pub history: Vec<HistoryEntry>,
    pub artifacts: Vec<ArtifactBundle>,
}

impl ProjectIndex {
    pub fn new(project_name: &str) -> Self {
        Self {
            project: project_name.to_string(),
            last_run: None,
            current_state: FileState::empty(),
            history: Vec::new(),
            artifacts: Vec::new(),
        }
    }
}

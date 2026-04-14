use std::path::Path;

use chrono::Local;

use crate::error::Result;
use crate::fs::atomic;
use crate::model::artifact::ArtifactBundle;
use crate::model::diff_result::DiffResult;
use crate::model::history_entry::HistoryEntry;
use crate::model::index::{FileState, ProjectIndex};

/// index.json을 소스 폴더에서 로드한다.
pub fn load_index(source_dir: &Path) -> Result<ProjectIndex> {
    let path = source_dir.join("index.json");
    let content = std::fs::read_to_string(&path)?;
    let index: ProjectIndex = serde_json::from_str(&content)?;
    Ok(index)
}

/// index.json을 소스 폴더에 원자적으로 저장한다.
pub fn save_index(source_dir: &Path, index: &ProjectIndex) -> Result<()> {
    let path = source_dir.join("index.json");
    let json = serde_json::to_string_pretty(index)?;
    atomic::atomic_write(&path, json.as_bytes())
}

/// 실행 결과를 index에 기록한다.
pub fn record_run(
    index: &mut ProjectIndex,
    diff: &DiffResult,
    artifact_files: &[String],
    bundle_id: &str,
    current_state: FileState,
) {
    let now = Local::now().fixed_offset();

    index.history.push(HistoryEntry {
        time: now,
        added: diff.added.clone(),
        modified: diff.modified.clone(),
        deleted: diff.deleted.clone(),
    });

    index.artifacts.push(ArtifactBundle {
        bundle: bundle_id.to_string(),
        files: artifact_files.to_vec(),
    });

    index.current_state = current_state;
    index.last_run = Some(now);
}

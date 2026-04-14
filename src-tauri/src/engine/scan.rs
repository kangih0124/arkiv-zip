use chrono::{DateTime, FixedOffset, Local};

use crate::error::Result;
use crate::fs::hasher;
use crate::fs::pattern::compile_patterns;
use crate::fs::walker;
use crate::model::config::ProjectConfig;
use crate::model::file_entry::FileEntry;
use crate::model::index::FileState;

/// 프로젝트 소스 디렉토리를 스캔하여 현재 파일 상태를 반환한다.
pub fn scan_project(config: &ProjectConfig) -> Result<FileState> {
    let patterns = compile_patterns(&config.exclude_patterns);
    let entries = walker::walk_directory(&config.source_dir, &patterns)?;

    let mut files = std::collections::BTreeMap::new();

    for (rel_path, metadata) in entries {
        let hash = hasher::hash_file(&config.source_dir.join(&rel_path))?;
        let size = metadata.len();
        let mtime: DateTime<FixedOffset> = metadata
            .modified()
            .map(|t| {
                let dt: DateTime<Local> = t.into();
                dt.fixed_offset()
            })
            .unwrap_or_else(|_| Local::now().fixed_offset());

        files.insert(
            rel_path,
            FileEntry { hash, size, mtime },
        );
    }

    Ok(FileState { files })
}

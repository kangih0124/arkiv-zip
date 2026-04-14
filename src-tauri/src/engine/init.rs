use std::fs;

use crate::error::{ArkivError, Result};
use crate::model::config::ProjectConfig;
use crate::model::index::ProjectIndex;

/// 프로젝트를 초기화한다: source_dir에 index.json을 생성한다.
pub fn init_project(config: &ProjectConfig) -> Result<()> {
    // 소스 디렉토리 존재 확인
    if !config.source_dir.exists() {
        return Err(ArkivError::SourceDirNotFound(config.source_dir.clone()));
    }

    // 아카이브 디렉토리 생성
    fs::create_dir_all(&config.archive_dir)?;

    // index.json은 소스 폴더 안에 생성
    let index_path = config.source_dir.join("index.json");
    if !index_path.exists() {
        let index = ProjectIndex::new(&config.name);
        let json = serde_json::to_string_pretty(&index)?;
        fs::write(&index_path, json)?;
    }

    Ok(())
}

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use crate::engine;
use crate::model::config::{ArchiveFormat, ProjectConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub source_dir: String,
    pub archive_dir: String,
    pub exclude_patterns: Vec<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            source_dir: String::new(),
            archive_dir: String::new(),
            exclude_patterns: vec![
                ".git".into(),
                "node_modules".into(),
                "__pycache__".into(),
                "*.tmp".into(),
                "*.log".into(),
                "dist".into(),
                "bak".into(),
            ],
        }
    }
}

fn config_path() -> PathBuf {
    // OS 표준 설정 경로: Windows=%APPDATA%/arkiv, Linux/Mac=~/.config/arkiv
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("arkiv");
    let _ = std::fs::create_dir_all(&config_dir);
    config_dir.join("config.json")
}

#[tauri::command]
pub fn load_settings() -> Result<AppSettings, String> {
    let path = config_path();
    if !path.exists() {
        return Ok(AppSettings::default());
    }
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_settings(settings: AppSettings) -> Result<(), String> {
    let path = config_path();
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())
}

#[derive(Debug, Serialize)]
pub struct ProjectResult {
    pub name: String,
    pub status: String, // "archived", "skipped", "error"
    pub added: usize,
    pub modified: usize,
    pub deleted: usize,
    pub archive_path: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ArchiveAllResult {
    pub total: usize,
    pub archived: usize,
    pub skipped: usize,
    pub errors: usize,
    pub projects: Vec<ProjectResult>,
}

#[derive(Debug, Deserialize)]
pub struct ArchiveAllRequest {
    pub source_dir: String,
    pub archive_dir: String,
    pub exclude_patterns: Vec<String>,
}

#[tauri::command]
pub fn archive_all(request: ArchiveAllRequest) -> Result<ArchiveAllResult, String> {
    run_archive_all(&request, false).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn archive_all_dry_run(request: ArchiveAllRequest) -> Result<ArchiveAllResult, String> {
    run_archive_all(&request, true).map_err(|e| e.to_string())
}

fn run_archive_all(request: &ArchiveAllRequest, dry_run: bool) -> crate::error::Result<ArchiveAllResult> {
    let source = PathBuf::from(&request.source_dir);
    let archive_base = PathBuf::from(&request.archive_dir);

    if !source.exists() {
        return Err(crate::error::ArkivError::SourceDirNotFound(source));
    }

    let mut entries: Vec<_> = std::fs::read_dir(&source)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    let mut projects = Vec::new();
    let mut archived = 0;
    let mut skipped = 0;
    let mut errors = 0;

    for entry in &entries {
        let folder_name = entry.file_name().to_string_lossy().to_string();
        let folder_path = entry.path();

        // bak 폴더 자체는 스킵
        let archive_canonical = std::fs::canonicalize(&archive_base).unwrap_or_default();
        let folder_canonical = std::fs::canonicalize(&folder_path).unwrap_or_default();
        if folder_canonical == archive_canonical {
            continue;
        }

        let result = process_single_project(
            &folder_name,
            &folder_path,
            &archive_base,
            &request.exclude_patterns,
            dry_run,
        );

        match result {
            Ok(pr) => {
                match pr.status.as_str() {
                    "archived" => archived += 1,
                    "skipped" => skipped += 1,
                    _ => {}
                }
                projects.push(pr);
            }
            Err(e) => {
                errors += 1;
                projects.push(ProjectResult {
                    name: folder_name,
                    status: "error".to_string(),
                    added: 0,
                    modified: 0,
                    deleted: 0,
                    archive_path: None,
                    error: Some(e.to_string()),
                });
            }
        }
    }

    Ok(ArchiveAllResult {
        total: projects.len(),
        archived,
        skipped,
        errors,
        projects,
    })
}

fn process_single_project(
    name: &str,
    folder_path: &PathBuf,
    archive_base: &PathBuf,
    exclude: &[String],
    dry_run: bool,
) -> crate::error::Result<ProjectResult> {
    let mut patterns = exclude.to_vec();
    for auto in &["index.json", ".arkiv.lock"] {
        if !patterns.iter().any(|p| p == *auto) {
            patterns.push(auto.to_string());
        }
    }

    let config = ProjectConfig {
        name: name.to_string(),
        source_dir: folder_path.clone(),
        meta_dir: folder_path.clone(),
        archive_dir: archive_base.clone(),
        exclude_patterns: patterns,
        compression_level: 6,
        max_retention_days: 30,
        archive_formats: vec![ArchiveFormat::Zip],
    };

    // 자동 init
    if !config.source_dir.join("index.json").exists() {
        engine::init::init_project(&config)?;
    }

    // 락
    let lock_path = config.source_dir.join(".arkiv.lock");
    let _lock = crate::fs::lock::acquire_lock(&lock_path)?;

    // index 로드
    let mut index = engine::history::load_index(&config.source_dir)?;

    // 스캔
    let current_state = engine::scan::scan_project(&config)?;

    // diff
    let diff = engine::diff::compute_diff(&index.current_state, &current_state);

    if !diff.has_changes() {
        return Ok(ProjectResult {
            name: name.to_string(),
            status: "skipped".to_string(),
            added: 0,
            modified: 0,
            deleted: 0,
            archive_path: None,
            error: None,
        });
    }

    if dry_run {
        return Ok(ProjectResult {
            name: name.to_string(),
            status: "dry_run".to_string(),
            added: diff.added.len(),
            modified: diff.modified.len(),
            deleted: diff.deleted.len(),
            archive_path: None,
            error: None,
        });
    }

    let bundle_id = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();

    // 이전 ZIP 삭제 (같은 프로젝트명으로 시작하는 ZIP)
    let prefix = format!("{}_", name);
    if let Ok(entries) = std::fs::read_dir(&config.archive_dir) {
        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name.starts_with(&prefix) && file_name.ends_with(".zip") {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }

    let created = engine::archive::create_archive(&config, &current_state, &bundle_id)?;

    let artifact_names: Vec<String> = created
        .iter()
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .collect();

    engine::history::record_run(&mut index, &diff, &artifact_names, &bundle_id, current_state);
    engine::history::save_index(&config.source_dir, &index)?;

    let archive_path = created.first().map(|p| p.display().to_string());

    Ok(ProjectResult {
        name: name.to_string(),
        status: "archived".to_string(),
        added: diff.added.len(),
        modified: diff.modified.len(),
        deleted: diff.deleted.len(),
        archive_path,
        error: None,
    })
}

#[derive(Debug, Serialize)]
pub struct StatusResult {
    pub project: String,
    pub last_run: Option<String>,
    pub file_count: usize,
    pub history_count: usize,
    pub artifact_count: usize,
}

#[tauri::command]
pub fn get_project_status(source_dir: String) -> Result<Vec<StatusResult>, String> {
    let source = PathBuf::from(&source_dir);
    if !source.exists() {
        return Err("소스 디렉토리가 존재하지 않습니다".to_string());
    }

    let mut results = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(&source)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let folder_path = entry.path();
        let index_path = folder_path.join("index.json");
        if !index_path.exists() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();
        match engine::history::load_index(&folder_path) {
            Ok(index) => {
                results.push(StatusResult {
                    project: name,
                    last_run: index.last_run.map(|t| t.to_string()),
                    file_count: index.current_state.files.len(),
                    history_count: index.history.len(),
                    artifact_count: index.artifacts.len(),
                });
            }
            Err(_) => continue,
        }
    }

    Ok(results)
}

#[derive(Debug, Serialize)]
pub struct VerifyResult {
    pub total_files: usize,
    pub valid_files: usize,
    pub corrupted_files: Vec<String>,
}

#[tauri::command]
pub fn verify_archive(archive_path: String) -> Result<VerifyResult, String> {
    let path = PathBuf::from(&archive_path);
    let report = engine::verify::verify_archive(&path).map_err(|e| e.to_string())?;
    Ok(VerifyResult {
        total_files: report.total_files,
        valid_files: report.valid_files,
        corrupted_files: report.corrupted_files,
    })
}

#[tauri::command]
pub fn restore_archive(
    archive_path: String,
    target_dir: String,
    policy: String,
) -> Result<String, String> {
    let path = PathBuf::from(&archive_path);
    let target = PathBuf::from(&target_dir);
    let restore_policy = engine::restore::RestorePolicy::from_str(&policy).map_err(|e| e.to_string())?;
    let report = engine::restore::restore_archive(&path, &target, restore_policy).map_err(|e| e.to_string())?;
    Ok(format!(
        "복원: {}개, 스킵: {}개, 오류: {}개",
        report.restored_files, report.skipped_files, report.errors.len()
    ))
}

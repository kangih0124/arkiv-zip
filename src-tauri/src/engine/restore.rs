use std::fs::{self, File};
use std::io;
use std::path::Path;

use crate::engine::verify;
use crate::error::{ArkivError, Result};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RestorePolicy {
    Overwrite,
    SkipExisting,
    EmptyOnly,
}

impl RestorePolicy {
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "overwrite" => Ok(Self::Overwrite),
            "skip" | "skip_existing" => Ok(Self::SkipExisting),
            "empty" | "empty_only" => Ok(Self::EmptyOnly),
            _ => Err(ArkivError::ArchiveCreationFailed(format!(
                "알 수 없는 복원 정책: {s}"
            ))),
        }
    }
}

#[derive(Debug)]
pub struct RestoreReport {
    pub restored_files: usize,
    pub skipped_files: usize,
    pub errors: Vec<String>,
}

/// 아카이브를 지정 경로에 복원한다.
pub fn restore_archive(
    archive_path: &Path,
    target_dir: &Path,
    policy: RestorePolicy,
) -> Result<RestoreReport> {
    // 1. 무결성 검증
    let verify_result = verify::verify_archive(archive_path)?;
    if !verify_result.corrupted_files.is_empty() {
        return Err(ArkivError::ArchiveCreationFailed(format!(
            "아카이브 손상: {:?}",
            verify_result.corrupted_files
        )));
    }

    // 2. EmptyOnly 정책: 대상 디렉토리가 비어있는지 확인
    if policy == RestorePolicy::EmptyOnly && target_dir.exists() {
        let has_entries = fs::read_dir(target_dir)?.next().is_some();
        if has_entries {
            return Err(ArkivError::ArchiveCreationFailed(format!(
                "대상 디렉토리가 비어있지 않습니다: {}",
                target_dir.display()
            )));
        }
    }

    // 3. 대상 디렉토리 생성
    fs::create_dir_all(target_dir)?;

    // 4. 쓰기 권한 확인
    let test_file = target_dir.join(".arkiv_write_test");
    match File::create(&test_file) {
        Ok(_) => {
            let _ = fs::remove_file(&test_file);
        }
        Err(_) => {
            return Err(ArkivError::PermissionDenied(format!(
                "쓰기 권한 없음: {}",
                target_dir.display()
            )));
        }
    }

    // 5. ZIP 추출
    let file = File::open(archive_path)?;
    let mut zip = zip::ZipArchive::new(file)?;

    let mut restored = 0;
    let mut skipped = 0;
    let mut errors = Vec::new();

    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let name = entry.name().to_string();
        let out_path = target_dir.join(&name);

        if entry.is_dir() {
            let _ = fs::create_dir_all(&out_path);
            continue;
        }

        // SkipExisting 정책: 기존 파일이 있으면 스킵
        if policy == RestorePolicy::SkipExisting && out_path.exists() {
            skipped += 1;
            continue;
        }

        // 부모 디렉토리 생성
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }

        match File::create(&out_path) {
            Ok(mut out_file) => match io::copy(&mut entry, &mut out_file) {
                Ok(_) => restored += 1,
                Err(e) => errors.push(format!("{name}: {e}")),
            },
            Err(e) => errors.push(format!("{name}: {e}")),
        }
    }

    Ok(RestoreReport {
        restored_files: restored,
        skipped_files: skipped,
        errors,
    })
}

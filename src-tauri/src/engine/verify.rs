use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::error::{ArkivError, Result};

#[derive(Debug)]
pub struct VerifyReport {
    pub total_files: usize,
    pub valid_files: usize,
    pub corrupted_files: Vec<String>,
}

/// ZIP 파일의 무결성을 검증한다 (CRC 체크).
pub fn verify_archive(archive_path: &Path) -> Result<VerifyReport> {
    if !archive_path.exists() {
        return Err(ArkivError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("파일을 찾을 수 없습니다: {}", archive_path.display()),
        )));
    }

    let file = File::open(archive_path)?;
    let mut zip = zip::ZipArchive::new(file)?;

    let total_files = zip.len();
    let mut valid_files = 0;
    let mut corrupted_files = Vec::new();

    for i in 0..total_files {
        let mut entry = zip.by_index(i)?;
        let name = entry.name().to_string();

        // 디렉토리는 스킵
        if entry.is_dir() {
            valid_files += 1;
            continue;
        }

        // 전체 내용을 읽어서 CRC 검증 (zip crate가 자동으로 CRC 체크)
        let mut buf = Vec::new();
        match entry.read_to_end(&mut buf) {
            Ok(_) => valid_files += 1,
            Err(_) => corrupted_files.push(name),
        }
    }

    Ok(VerifyReport {
        total_files,
        valid_files,
        corrupted_files,
    })
}

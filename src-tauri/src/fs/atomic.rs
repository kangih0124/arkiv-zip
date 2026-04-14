use std::fs;
use std::io::Write;
use std::path::Path;

use tempfile::NamedTempFile;

use crate::error::Result;

/// 원자적 파일 쓰기: 임시 파일에 쓴 후 rename으로 교체한다.
pub fn atomic_write(target: &Path, content: &[u8]) -> Result<()> {
    let parent = target.parent().unwrap_or(Path::new("."));
    let mut tmp = NamedTempFile::new_in(parent)?;
    tmp.write_all(content)?;
    tmp.flush()?;

    let tmp_path = tmp.into_temp_path();
    // persist가 실패하면 임시 파일은 자동 삭제됨
    tmp_path.persist(target).map_err(|e| e.error)?;

    // Windows에서는 rename이 기존 파일을 덮어쓰지 못할 수 있으므로
    // persist가 이를 처리해줌
    let _ = fs::metadata(target)?; // 검증
    Ok(())
}

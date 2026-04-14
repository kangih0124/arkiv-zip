use std::fs;
use std::path::Path;

use crate::error::Result;
use crate::fs::pattern::Pattern;

/// 디렉토리를 재귀 탐색하여 (상대경로, 메타데이터) 목록을 반환한다.
/// 상대 경로는 `/` 구분자로 통일된다.
pub fn walk_directory(
    root: &Path,
    exclude: &[Pattern],
) -> Result<Vec<(String, fs::Metadata)>> {
    let mut results = Vec::new();
    walk_recursive(root, root, exclude, &mut results)?;
    Ok(results)
}

fn walk_recursive(
    root: &Path,
    current: &Path,
    exclude: &[Pattern],
    results: &mut Vec<(String, fs::Metadata)>,
) -> Result<()> {
    let entries = fs::read_dir(current)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");

        // 제외 패턴 확인
        if exclude.iter().any(|p| p.matches(&rel)) {
            continue;
        }

        if path.is_dir() {
            walk_recursive(root, &path, exclude, results)?;
        } else if path.is_file() {
            let metadata = fs::metadata(&path)?;
            results.push((rel, metadata));
        }
    }

    Ok(())
}

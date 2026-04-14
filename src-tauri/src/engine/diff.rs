use crate::model::diff_result::DiffResult;
use crate::model::index::FileState;

/// 이전 상태와 현재 상태를 비교하여 변경 사항을 반환한다.
/// 순수 함수 — 부수효과 없음.
pub fn compute_diff(previous: &FileState, current: &FileState) -> DiffResult {
    let mut added = Vec::new();
    let mut modified = Vec::new();
    let mut deleted = Vec::new();

    // current에만 있거나 hash가 다른 파일
    for (path, entry) in &current.files {
        match previous.files.get(path) {
            None => added.push(path.clone()),
            Some(prev_entry) => {
                if prev_entry.hash != entry.hash {
                    modified.push(path.clone());
                }
            }
        }
    }

    // previous에만 있는 파일
    for path in previous.files.keys() {
        if !current.files.contains_key(path) {
            deleted.push(path.clone());
        }
    }

    DiffResult {
        added,
        modified,
        deleted,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::file_entry::FileEntry;
    use chrono::Local;
    use std::collections::BTreeMap;

    fn entry(hash: &str) -> FileEntry {
        FileEntry {
            hash: hash.to_string(),
            size: 100,
            mtime: Local::now().fixed_offset(),
        }
    }

    #[test]
    fn no_changes() {
        let mut files = BTreeMap::new();
        files.insert("a.txt".to_string(), entry("aaa"));
        let state = FileState { files };
        let diff = compute_diff(&state, &state);
        assert!(!diff.has_changes());
    }

    #[test]
    fn detect_added() {
        let prev = FileState { files: BTreeMap::new() };
        let mut cur_files = BTreeMap::new();
        cur_files.insert("new.txt".to_string(), entry("nnn"));
        let cur = FileState { files: cur_files };
        let diff = compute_diff(&prev, &cur);
        assert_eq!(diff.added, vec!["new.txt"]);
        assert!(diff.modified.is_empty());
        assert!(diff.deleted.is_empty());
    }

    #[test]
    fn detect_modified() {
        let mut files_a = BTreeMap::new();
        files_a.insert("a.txt".to_string(), entry("old"));
        let mut files_b = BTreeMap::new();
        files_b.insert("a.txt".to_string(), entry("new"));
        let diff = compute_diff(
            &FileState { files: files_a },
            &FileState { files: files_b },
        );
        assert_eq!(diff.modified, vec!["a.txt"]);
    }

    #[test]
    fn detect_deleted() {
        let mut files = BTreeMap::new();
        files.insert("old.txt".to_string(), entry("ooo"));
        let diff = compute_diff(
            &FileState { files },
            &FileState { files: BTreeMap::new() },
        );
        assert_eq!(diff.deleted, vec!["old.txt"]);
    }
}

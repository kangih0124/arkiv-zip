use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error::{ArkivError, Result};

/// 락 파일 가드. Drop 시 자동으로 락 파일을 삭제한다.
pub struct LockGuard {
    path: PathBuf,
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// 락 파일을 획득한다. 이미 존재하면 stale 여부를 확인한다.
pub fn acquire_lock(lock_path: &Path) -> Result<LockGuard> {
    if lock_path.exists() {
        // stale lock 확인: PID를 읽어서 프로세스 존재 여부 확인
        if let Ok(content) = fs::read_to_string(lock_path) {
            if let Ok(pid) = content.trim().parse::<u32>() {
                if !is_process_alive(pid) {
                    // stale lock — 삭제하고 계속 진행
                    let _ = fs::remove_file(lock_path);
                } else {
                    return Err(ArkivError::LockAcquisitionFailed);
                }
            } else {
                // PID 파싱 실패 — 손상된 락 파일, 삭제
                let _ = fs::remove_file(lock_path);
            }
        }
    }

    // 락 파일 생성
    if let Some(parent) = lock_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut f = fs::File::create(lock_path)?;
    write!(f, "{}", std::process::id())?;

    Ok(LockGuard {
        path: lock_path.to_path_buf(),
    })
}

#[cfg(target_os = "windows")]
fn is_process_alive(pid: u32) -> bool {
    use std::process::Command;
    Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}"), "/NH"])
        .output()
        .map(|o| {
            let out = String::from_utf8_lossy(&o.stdout);
            out.contains(&pid.to_string())
        })
        .unwrap_or(false)
}

#[cfg(not(target_os = "windows"))]
fn is_process_alive(pid: u32) -> bool {
    use std::path::PathBuf;
    PathBuf::from(format!("/proc/{pid}")).exists()
}

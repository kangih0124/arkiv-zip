use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::error::Result;

const CHUNK_SIZE: usize = 8192;

/// 파일의 BLAKE3 해시를 계산한다 (8KB 청크 단위).
pub fn hash_file(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = [0u8; CHUNK_SIZE];

    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}

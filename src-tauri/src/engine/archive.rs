use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

use zip::write::SimpleFileOptions;
use zip::CompressionMethod;

use crate::error::{ArkivError, Result};
use crate::model::config::{ArchiveFormat, ProjectConfig};
use crate::model::index::FileState;

/// 아카이브를 생성한다. bak 폴더에 플랫하게 ZIP을 저장한다.
pub fn create_archive(
    config: &ProjectConfig,
    current_state: &FileState,
    bundle_id: &str,
) -> Result<Vec<PathBuf>> {
    fs::create_dir_all(&config.archive_dir)?;

    let mut created_files = Vec::new();

    for format in &config.archive_formats {
        match format {
            ArchiveFormat::Zip => {
                let path = create_zip(config, current_state, bundle_id)?;
                created_files.push(path);
            }
        }
    }

    Ok(created_files)
}

fn create_zip(
    config: &ProjectConfig,
    current_state: &FileState,
    bundle_id: &str,
) -> Result<PathBuf> {
    let zip_name = format!("{}_{}.zip", config.name, bundle_id);
    let final_path = config.archive_dir.join(&zip_name);
    let tmp_path = config.archive_dir.join(format!(".{}.tmp", zip_name));

    let result = (|| -> Result<()> {
        let file = File::create(&tmp_path)?;
        let mut zip_writer = zip::ZipWriter::new(file);

        let level = config.compression_level.min(9) as i64;
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .compression_level(Some(level));

        for (rel_path, _entry) in &current_state.files {
            let full_path = config.source_dir.join(rel_path);
            if !full_path.exists() {
                continue;
            }

            zip_writer.start_file(rel_path, options)?;
            let mut f = File::open(&full_path)?;
            let mut buf = [0u8; 8192];
            loop {
                let n = f.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                zip_writer.write_all(&buf[..n])?;
            }
        }

        zip_writer.finish()?;
        Ok(())
    })();

    match result {
        Ok(()) => {
            fs::rename(&tmp_path, &final_path)?;
            Ok(final_path)
        }
        Err(e) => {
            let _ = fs::remove_file(&tmp_path);
            Err(ArkivError::ArchiveCreationFailed(e.to_string()))
        }
    }
}

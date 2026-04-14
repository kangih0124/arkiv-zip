use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum ArchiveFormat {
    Zip,
}

#[derive(Debug, Clone)]
pub struct ProjectConfig {
    pub name: String,
    pub source_dir: PathBuf,
    pub meta_dir: PathBuf,
    pub archive_dir: PathBuf,
    pub exclude_patterns: Vec<String>,
    pub compression_level: u32,
    pub max_retention_days: u32,
    pub archive_formats: Vec<ArchiveFormat>,
}

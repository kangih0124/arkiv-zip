use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ArkivError {
    #[error("프로젝트를 찾을 수 없습니다: {0}")]
    ProjectNotFound(String),

    #[error("소스 디렉토리가 존재하지 않습니다: {0}")]
    SourceDirNotFound(PathBuf),

    #[error("순환 경로가 감지되었습니다: archive_dir가 source_dir 내부입니다")]
    CircularPath,

    #[error("아카이브 생성 실패: {0}")]
    ArchiveCreationFailed(String),

    #[error("락 획득 실패: 다른 프로세스가 실행 중입니다")]
    LockAcquisitionFailed,

    #[error("IO 오류: {0}")]
    Io(#[from] std::io::Error),

    #[error("권한 오류: {0}")]
    PermissionDenied(String),

    #[error("JSON 파싱 오류: {0}")]
    Json(#[from] serde_json::Error),

    #[error("ZIP 오류: {0}")]
    Zip(#[from] zip::result::ZipError),
}

pub type Result<T> = std::result::Result<T, ArkivError>;

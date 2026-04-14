use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactBundle {
    pub bundle: String,
    pub files: Vec<String>,
}

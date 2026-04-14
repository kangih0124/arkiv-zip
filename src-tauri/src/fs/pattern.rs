use glob::Pattern as GlobPattern;

#[derive(Debug, Clone)]
pub struct Pattern {
    raw: String,
    glob: Option<GlobPattern>,
}

impl Pattern {
    pub fn new(raw: &str) -> Self {
        let glob = GlobPattern::new(raw).ok();
        Self {
            raw: raw.to_string(),
            glob,
        }
    }

    /// 상대 경로가 이 패턴에 매칭되는지 확인한다.
    /// 경로는 `/` 구분자 기준으로 전달되어야 한다.
    pub fn matches(&self, relative_path: &str) -> bool {
        // 1. glob 패턴 매칭 (파일명)
        if let Some(ref glob) = self.glob {
            let file_name = relative_path.rsplit('/').next().unwrap_or(relative_path);
            if glob.matches(file_name) {
                return true;
            }
            if glob.matches(relative_path) {
                return true;
            }
        }

        // 2. 폴더명 매칭 (경로 파트에 포함)
        let clean = self.raw.trim_end_matches('/');
        for part in relative_path.split('/') {
            if part == clean {
                return true;
            }
        }

        false
    }
}

pub fn compile_patterns(raw_patterns: &[String]) -> Vec<Pattern> {
    raw_patterns.iter().map(|p| Pattern::new(p)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wildcard_extension() {
        let p = Pattern::new("*.tmp");
        assert!(p.matches("data.tmp"));
        assert!(p.matches("sub/dir/data.tmp"));
        assert!(!p.matches("data.txt"));
    }

    #[test]
    fn folder_name() {
        let p = Pattern::new(".git");
        assert!(p.matches(".git/config"));
        assert!(p.matches("sub/.git/HEAD"));
        assert!(!p.matches("gitconfig"));
    }

    #[test]
    fn folder_with_slash() {
        let p = Pattern::new("node_modules");
        assert!(p.matches("node_modules/pkg/index.js"));
    }
}

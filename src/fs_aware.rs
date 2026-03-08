use std::path::Path;

/// Find the git repository root for the given path.
/// Walks up from the path looking for a `.git` directory or file.
/// Returns the repo root directory name (not the full path).
pub fn find_git_root(path: &str) -> Option<String> {
    let p = Path::new(path);
    let start = if p.is_file() {
        p.parent()?
    } else if p.exists() {
        p
    } else {
        // Path doesn't exist on disk, try parent dirs
        p.parent()?
    };
    let mut current = start;
    loop {
        if current.join(".git").exists() {
            return current.file_name()?.to_str().map(|s| s.to_string());
        }
        current = current.parent()?;
    }
}

/// Find the shortest unique prefix for a directory name among its filesystem siblings.
/// Only compares against sibling directories (not files).
/// Returns the full name if no shorter prefix is unique or if the directory can't be read.
pub fn disambiguate_segment(parent_path: &Path, segment: &str) -> String {
    let siblings: Vec<String> = match std::fs::read_dir(parent_path) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .filter_map(|e| e.file_name().into_string().ok())
            .filter(|name| name != segment)
            .collect(),
        Err(_) => return segment.to_string(),
    };

    if siblings.is_empty() {
        // No siblings — 1 char is enough
        if segment.is_empty() {
            return String::new();
        }
        return segment.chars().next().unwrap().to_string();
    }

    for len in 1..=segment.len() {
        let prefix: String = segment.chars().take(len).collect();
        let is_unique = siblings.iter().all(|s| {
            let s_prefix: String = s.chars().take(len).collect();
            s_prefix != prefix
        });
        if is_unique {
            return prefix;
        }
    }
    segment.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("shrinkpath_test_{}_{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn find_git_root_found() {
        let dir = temp_dir("git_found");
        fs::create_dir_all(dir.join("src/deep/nested")).unwrap();
        fs::create_dir(dir.join(".git")).unwrap();
        fs::write(dir.join("src/deep/nested/main.rs"), "").unwrap();

        let root = find_git_root(dir.join("src/deep/nested/main.rs").to_str().unwrap());
        assert_eq!(root.as_deref(), dir.file_name().unwrap().to_str());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn find_git_root_not_found() {
        let dir = temp_dir("git_notfound");
        fs::create_dir_all(&dir).unwrap();
        // No .git directory
        let root = find_git_root(dir.join("file.txt").to_str().unwrap());
        // Should be None or find an ancestor .git (we can't control that)
        // Just verify it doesn't panic
        let _ = root;
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn find_git_root_at_root() {
        let dir = temp_dir("git_at_root");
        fs::create_dir_all(&dir).unwrap();
        fs::create_dir(dir.join(".git")).unwrap();
        fs::write(dir.join("file.txt"), "").unwrap();

        let root = find_git_root(dir.join("file.txt").to_str().unwrap());
        assert_eq!(root.as_deref(), dir.file_name().unwrap().to_str());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn disambiguate_with_siblings() {
        let dir = temp_dir("disambig_siblings");
        fs::create_dir_all(dir.join("documents")).unwrap();
        fs::create_dir_all(dir.join("downloads")).unwrap();
        fs::create_dir_all(dir.join("desktop")).unwrap();

        assert_eq!(disambiguate_segment(&dir, "documents"), "doc");
        assert_eq!(disambiguate_segment(&dir, "downloads"), "dow");
        assert_eq!(disambiguate_segment(&dir, "desktop"), "de");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn disambiguate_no_siblings() {
        let dir = temp_dir("disambig_alone");
        fs::create_dir_all(dir.join("only_child")).unwrap();

        let result = disambiguate_segment(&dir, "only_child");
        assert_eq!(result, "o");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn disambiguate_identical_prefix() {
        let dir = temp_dir("disambig_identical");
        fs::create_dir_all(dir.join("app")).unwrap();
        fs::create_dir_all(dir.join("application")).unwrap();

        // "app" can't be shortened further since "application" starts with "app"
        assert_eq!(disambiguate_segment(&dir, "app"), "app");
        // "application" needs "appl" to distinguish from "app"
        assert_eq!(disambiguate_segment(&dir, "application"), "appl");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn disambiguate_unreadable_dir() {
        // Non-existent parent should return full segment
        let result = disambiguate_segment(Path::new("/nonexistent_dir_12345"), "test");
        assert_eq!(result, "test");
    }
}

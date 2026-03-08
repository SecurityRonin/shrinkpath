/// Path style: Unix forward-slash or Windows backslash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathStyle {
    /// Forward-slash paths: `/home/user/file.txt`
    Unix,
    /// Backslash paths: `C:\Users\user\file.txt`, `\\server\share\...`
    Windows,
}

/// Auto-detect path style from the input string.
pub fn detect_style(path: &str) -> PathStyle {
    let bytes = path.as_bytes();

    // UNC path: \\server\share
    if bytes.starts_with(b"\\\\") {
        return PathStyle::Windows;
    }

    // Drive letter: C:\ or C:/
    if bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'\\' || bytes[2] == b'/')
    {
        return PathStyle::Windows;
    }

    // Dot-backslash prefix: .\Users\...
    if bytes.starts_with(b".\\") {
        return PathStyle::Windows;
    }

    // Heuristic: contains backslash but no forward slash
    if path.contains('\\') && !path.contains('/') {
        return PathStyle::Windows;
    }

    PathStyle::Unix
}

/// Segments immediately after these are identity (username) segments.
pub const UNIX_HOME_ROOTS: &[&str] = &["home", "Users"];
pub const WIN_HOME_ROOTS: &[&str] = &["Users"];

/// Returns the separator character for a path style.
pub fn separator(style: PathStyle) -> char {
    match style {
        PathStyle::Unix => '/',
        PathStyle::Windows => '\\',
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_unix() {
        assert_eq!(detect_style("/home/user/file.txt"), PathStyle::Unix);
        assert_eq!(detect_style("/Users/john/Documents"), PathStyle::Unix);
        assert_eq!(detect_style("~/file.txt"), PathStyle::Unix);
        assert_eq!(detect_style("relative/path/file"), PathStyle::Unix);
    }

    #[test]
    fn detect_windows_drive() {
        assert_eq!(
            detect_style("C:\\Users\\john\\file.txt"),
            PathStyle::Windows
        );
        assert_eq!(detect_style("D:\\Data\\file.txt"), PathStyle::Windows);
        assert_eq!(detect_style("C:/Users/john/file.txt"), PathStyle::Windows);
    }

    #[test]
    fn detect_windows_unc() {
        assert_eq!(
            detect_style("\\\\server\\share\\file.txt"),
            PathStyle::Windows
        );
    }

    #[test]
    fn detect_windows_dot_backslash() {
        assert_eq!(
            detect_style(".\\Users\\Admin\\file.txt"),
            PathStyle::Windows
        );
    }

    #[test]
    fn detect_windows_backslash_only() {
        assert_eq!(detect_style("Users\\Admin\\file.txt"), PathStyle::Windows);
    }
}

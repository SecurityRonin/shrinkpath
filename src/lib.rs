//! # shrinkpath
//!
//! Smart, cross-platform path shortening for Rust.
//!
//! Intelligently shortens file paths while preserving the information that matters
//! most: the **filename** (never truncated) and the **user identity** (preserved
//! when possible).
//!
//! ## Quick Start
//!
//! ```
//! use shrinkpath::{shrink_to, shrink_fish};
//!
//! // Hybrid strategy with target length
//! let short = shrink_to("/home/john/projects/rust/myapp/src/lib.rs", 30);
//! assert!(short.len() <= 30);
//! assert!(short.ends_with("lib.rs"));
//!
//! // Fish-style abbreviation (no length target)
//! let fish = shrink_fish("/home/john/projects/rust/myapp/src/lib.rs");
//! assert_eq!(fish, "/h/j/p/r/m/s/lib.rs");
//! ```
//!
//! ## Strategies
//!
//! - **Hybrid** (default): Graduated approach — abbreviates expendable segments first,
//!   then context segments, then collapses runs into `...`, then abbreviates identity.
//! - **Fish**: Abbreviates every directory segment to its first character.
//! - **Ellipsis**: Replaces middle segments with `...`, keeping identity and tail.
//!
//! ## Platform Support
//!
//! Handles Windows (`C:\Users\...`, UNC `\\server\share\...`, `.\...`),
//! macOS (`/Users/...`), and Linux (`/home/...`) paths. Auto-detects the style
//! from the input string, or you can force it with [`PathStyle`].

pub mod path_info;
pub mod platform;
pub mod strategy;

pub use platform::PathStyle;
pub use strategy::Strategy;

/// Configuration for path shortening.
#[derive(Debug, Clone)]
pub struct ShrinkOptions {
    /// Target maximum length. The output will be at most this long, unless the
    /// filename itself exceeds it (filenames are never truncated).
    pub max_len: usize,
    /// Shortening strategy.
    pub strategy: Strategy,
    /// Force a specific path style instead of auto-detecting.
    pub path_style: Option<PathStyle>,
    /// Custom ellipsis string. Default: `"..."`.
    pub ellipsis: String,
}

impl ShrinkOptions {
    /// Create options with sensible defaults: Hybrid strategy, max_len as specified.
    pub fn new(max_len: usize) -> Self {
        ShrinkOptions {
            max_len,
            strategy: Strategy::Hybrid,
            path_style: None,
            ellipsis: "...".to_string(),
        }
    }

    /// Set the shortening strategy.
    pub fn strategy(mut self, s: Strategy) -> Self {
        self.strategy = s;
        self
    }

    /// Force a specific path style.
    pub fn path_style(mut self, s: PathStyle) -> Self {
        self.path_style = Some(s);
        self
    }

    /// Set a custom ellipsis string.
    pub fn ellipsis(mut self, e: impl Into<String>) -> Self {
        self.ellipsis = e.into();
        self
    }
}

/// Result of a shrink operation with metadata.
#[derive(Debug, Clone)]
pub struct ShrinkResult {
    /// The shortened path string.
    pub shortened: String,
    /// Length of the original path.
    pub original_len: usize,
    /// Length of the shortened path.
    pub shortened_len: usize,
    /// Whether the path was actually truncated.
    pub was_truncated: bool,
    /// Detected (or forced) path style.
    pub detected_style: PathStyle,
}

/// Shorten a path using the given options.
pub fn shrink(path: &str, opts: &ShrinkOptions) -> String {
    if path.is_empty() {
        return String::new();
    }

    let info = path_info::PathInfo::parse(path, opts.path_style);

    match opts.strategy {
        Strategy::Fish => strategy::fish::shrink_fish(&info),
        Strategy::Ellipsis => {
            strategy::ellipsis::shrink_ellipsis(&info, opts.max_len, &opts.ellipsis)
        }
        Strategy::Hybrid => strategy::hybrid::shrink_hybrid(&info, opts.max_len, &opts.ellipsis),
    }
}

/// Shorten a path with detailed result metadata.
pub fn shrink_detailed(path: &str, opts: &ShrinkOptions) -> ShrinkResult {
    let info = path_info::PathInfo::parse(path, opts.path_style);
    let shortened = shrink(path, opts);
    ShrinkResult {
        original_len: path.len(),
        shortened_len: shortened.len(),
        was_truncated: shortened != path,
        detected_style: info.style,
        shortened,
    }
}

// ── Convenience functions ────────────────────────────────────────────────────

/// Shorten a path using the Hybrid strategy with a target max length.
pub fn shrink_to(path: &str, max_len: usize) -> String {
    shrink(path, &ShrinkOptions::new(max_len))
}

/// Shorten a path using Fish-style abbreviation (no length target).
pub fn shrink_fish(path: &str) -> String {
    let info = path_info::PathInfo::parse(path, None);
    strategy::fish::shrink_fish(&info)
}

/// Shorten a path using ellipsis with a target max length.
pub fn shrink_ellipsis(path: &str, max_len: usize) -> String {
    shrink(
        path,
        &ShrinkOptions::new(max_len).strategy(Strategy::Ellipsis),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convenience_shrink_to() {
        let result = shrink_to("/home/john/projects/rust/myapp/src/lib.rs", 30);
        assert!(result.len() <= 30);
        assert!(result.ends_with("lib.rs"));
    }

    #[test]
    fn convenience_shrink_fish() {
        let result = shrink_fish("/home/john/projects/rust/myapp/src/lib.rs");
        assert_eq!(result, "/h/j/p/r/m/s/lib.rs");
    }

    #[test]
    fn convenience_shrink_ellipsis() {
        let result = shrink_ellipsis("/home/john/projects/rust/myapp/src/lib.rs", 30);
        assert!(result.len() <= 30);
        assert!(result.ends_with("lib.rs"));
        assert!(result.contains("..."));
    }

    #[test]
    fn empty_path() {
        assert_eq!(shrink_to("", 30), "");
    }

    #[test]
    fn root_only() {
        assert_eq!(shrink_to("/", 5), "/");
    }

    #[test]
    fn filename_only() {
        assert_eq!(shrink_to("file.txt", 5), "file.txt");
    }

    #[test]
    fn detailed_result() {
        let result = shrink_detailed(
            "/home/john/projects/rust/myapp/src/lib.rs",
            &ShrinkOptions::new(30),
        );
        assert!(result.was_truncated);
        assert!(result.shortened_len <= 30);
        assert_eq!(result.detected_style, PathStyle::Unix);
    }

    #[test]
    fn detailed_no_truncation() {
        let result = shrink_detailed("/home/user/file.txt", &ShrinkOptions::new(50));
        assert!(!result.was_truncated);
        assert_eq!(result.shortened, "/home/user/file.txt");
    }

    #[test]
    fn custom_ellipsis() {
        let opts = ShrinkOptions::new(25)
            .strategy(Strategy::Ellipsis)
            .ellipsis("..");
        let result = shrink("/home/john/deep/nested/path/to/file.rs", &opts);
        assert!(result.len() <= 25, "got len {}: {}", result.len(), result);
        assert!(
            result.contains(".."),
            "should contain custom ellipsis: {result}"
        );
        assert!(
            !result.contains("..."),
            "should use '..' not '...': {result}"
        );
    }

    #[test]
    fn force_windows_style() {
        let result = shrink_to("C:\\Users\\Admin\\AppData\\Local\\Temp\\file.txt", 30);
        assert!(result.len() <= 30);
        assert!(result.ends_with("file.txt"));
    }

    #[test]
    fn force_path_style() {
        let opts = ShrinkOptions::new(30).path_style(PathStyle::Windows);
        let result = shrink("C:/Users/Admin/AppData/Local/Temp/file.txt", &opts);
        assert!(
            result.contains('\\'),
            "should use windows separators: {result}"
        );
    }

    #[test]
    fn unc_path() {
        let result = shrink_to("\\\\server\\share\\dept\\project\\reports\\q4.xlsx", 35);
        assert!(result.len() <= 35);
        assert!(result.ends_with("q4.xlsx"));
    }

    #[test]
    fn idempotent_on_short_paths() {
        let path = "/home/user/file.txt";
        let result = shrink_to(path, 50);
        assert_eq!(result, path);
    }

    #[test]
    fn cross_platform_windows_on_any_host() {
        // Windows path should be detected and handled regardless of host OS
        let result = shrink_to(
            ".\\Users\\Admin\\AppData\\Local\\Packages\\Microsoft\\file.txt",
            40,
        );
        assert!(result.len() <= 40, "got len {}: {}", result.len(), result);
        assert!(result.ends_with("file.txt"));
        assert!(result.contains('\\'));
    }
}

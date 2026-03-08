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

#[cfg(feature = "fs")]
pub mod fs_aware;

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
    /// Number of characters to keep per abbreviated directory segment. Default: 1.
    pub dir_length: usize,
    /// Number of trailing directory segments to keep unabbreviated. Default: 0.
    pub full_length_dirs: usize,
    /// Custom path prefix substitutions applied before shortening.
    /// Each tuple is `(from, to)`: if the path starts with `from`, replace it with `to`.
    /// Sorted by longest match first at application time.
    pub mapped_locations: Vec<(String, String)>,
    /// Segment names that should never be abbreviated, regardless of strategy.
    pub anchors: Vec<String>,
}

impl ShrinkOptions {
    /// Create options with sensible defaults: Hybrid strategy, max_len as specified.
    pub fn new(max_len: usize) -> Self {
        ShrinkOptions {
            max_len,
            strategy: Strategy::Hybrid,
            path_style: None,
            ellipsis: "...".to_string(),
            dir_length: 1,
            full_length_dirs: 0,
            mapped_locations: Vec::new(),
            anchors: Vec::new(),
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

    /// Set the number of characters to keep per abbreviated directory segment.
    pub fn dir_length(mut self, n: usize) -> Self {
        self.dir_length = n;
        self
    }

    /// Set the number of trailing directory segments to keep unabbreviated.
    pub fn full_length_dirs(mut self, n: usize) -> Self {
        self.full_length_dirs = n;
        self
    }

    /// Add a mapped location: if the path starts with `from`, replace it with `to`.
    pub fn map_location(mut self, from: impl Into<String>, to: impl Into<String>) -> Self {
        self.mapped_locations.push((from.into(), to.into()));
        self
    }

    /// Add an anchor segment name that should never be abbreviated.
    pub fn anchor(mut self, name: impl Into<String>) -> Self {
        self.anchors.push(name.into());
        self
    }
}

/// Metadata about a single component in the shortened path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentInfo {
    /// The original full text.
    pub original: String,
    /// The shortened text (may equal original if not abbreviated).
    pub shortened: String,
    /// Whether this component was abbreviated.
    pub was_abbreviated: bool,
    /// Whether this is the filename (sacred, always last).
    pub is_filename: bool,
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
    /// Per-segment metadata for the shortened path.
    pub segments: Vec<SegmentInfo>,
}

/// Apply mapped location substitutions to a path, returning the transformed path.
/// Sorts by longest `from` prefix first so more specific mappings win.
fn apply_mapped_locations(path: &str, mapped_locations: &[(String, String)]) -> String {
    if mapped_locations.is_empty() {
        return path.to_string();
    }

    // Sort by longest from-prefix first
    let mut sorted: Vec<&(String, String)> = mapped_locations.iter().collect();
    sorted.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

    for (from, to) in sorted {
        if path.starts_with(from.as_str()) {
            let remainder = &path[from.len()..];
            return format!("{to}{remainder}");
        }
    }

    path.to_string()
}

/// Shorten a path using the given options.
pub fn shrink(path: &str, opts: &ShrinkOptions) -> String {
    if path.is_empty() {
        return String::new();
    }

    // Apply mapped locations before parsing
    let path = apply_mapped_locations(path, &opts.mapped_locations);

    let info = path_info::PathInfo::parse(&path, opts.path_style);

    match opts.strategy {
        Strategy::Fish => strategy::fish::shrink_fish(
            &info,
            opts.dir_length,
            opts.full_length_dirs,
            &opts.anchors,
        ),
        Strategy::Ellipsis => {
            strategy::ellipsis::shrink_ellipsis(&info, opts.max_len, &opts.ellipsis)
        }
        Strategy::Hybrid => {
            strategy::hybrid::shrink_hybrid(&info, opts.max_len, &opts.ellipsis, &opts.anchors)
        }
        Strategy::Unique => strategy::unique::shrink_unique(&info, &opts.anchors),
    }
}

/// Shorten a path with detailed result metadata.
pub fn shrink_detailed(path: &str, opts: &ShrinkOptions) -> ShrinkResult {
    if path.is_empty() {
        return ShrinkResult {
            shortened: String::new(),
            original_len: 0,
            shortened_len: 0,
            was_truncated: false,
            detected_style: PathStyle::Unix,
            segments: Vec::new(),
        };
    }

    // Parse the original path (after mapped-location substitution, same as shrink())
    let mapped_path = apply_mapped_locations(path, &opts.mapped_locations);
    let original_info = path_info::PathInfo::parse(&mapped_path, opts.path_style);

    let shortened = shrink(path, opts);

    // Parse the shortened string to extract its segments
    let shortened_info = path_info::PathInfo::parse(&shortened, opts.path_style);

    let segments = build_segment_metadata(&original_info, &shortened_info);

    ShrinkResult {
        original_len: path.len(),
        shortened_len: shortened.len(),
        was_truncated: shortened != path,
        detected_style: original_info.style,
        shortened,
        segments,
    }
}

/// Build per-segment metadata by comparing original and shortened PathInfo.
fn build_segment_metadata(
    original: &path_info::PathInfo,
    shortened: &path_info::PathInfo,
) -> Vec<SegmentInfo> {
    let mut result = Vec::new();

    let orig_seg_count = original.segments.len();
    let short_seg_count = shortened.segments.len();

    if orig_seg_count == short_seg_count {
        // 1-to-1 mapping: Fish, Unique, Hybrid (when no collapse), or no truncation
        for (orig, short) in original.segments.iter().zip(shortened.segments.iter()) {
            result.push(SegmentInfo {
                original: orig.text.clone(),
                shortened: short.text.clone(),
                was_abbreviated: orig.text != short.text,
                is_filename: false,
            });
        }
    } else {
        // Shortened has fewer segments (ellipsis collapse or hybrid collapse).
        // Walk through shortened segments, mapping them back to originals.
        let short_texts: Vec<&str> = shortened.segments.iter().map(|s| s.text.as_str()).collect();

        // Find the ellipsis marker position in shortened segments
        let ellipsis_pos = short_texts
            .iter()
            .position(|t| t.contains("...") || t.contains(".."));

        match ellipsis_pos {
            Some(eidx) => {
                // Segments before ellipsis map to the first eidx original segments
                for i in 0..eidx {
                    if i < orig_seg_count {
                        result.push(SegmentInfo {
                            original: original.segments[i].text.clone(),
                            shortened: shortened.segments[i].text.clone(),
                            was_abbreviated: original.segments[i].text
                                != shortened.segments[i].text,
                            is_filename: false,
                        });
                    }
                }

                // The ellipsis itself represents collapsed segments
                let tail_count = short_seg_count - eidx - 1;
                let collapsed_start = eidx;
                let collapsed_end = orig_seg_count.saturating_sub(tail_count);
                for i in collapsed_start..collapsed_end {
                    result.push(SegmentInfo {
                        original: original.segments[i].text.clone(),
                        shortened: "...".to_string(),
                        was_abbreviated: true,
                        is_filename: false,
                    });
                }

                // Tail segments after ellipsis
                for i in (eidx + 1)..short_seg_count {
                    let orig_idx = orig_seg_count - (short_seg_count - i);
                    if orig_idx < orig_seg_count {
                        result.push(SegmentInfo {
                            original: original.segments[orig_idx].text.clone(),
                            shortened: shortened.segments[i].text.clone(),
                            was_abbreviated: original.segments[orig_idx].text
                                != shortened.segments[i].text,
                            is_filename: false,
                        });
                    }
                }
            }
            None => {
                // No ellipsis marker but different counts — fallback: use shortened as-is
                for seg in &shortened.segments {
                    result.push(SegmentInfo {
                        original: seg.text.clone(),
                        shortened: seg.text.clone(),
                        was_abbreviated: false,
                        is_filename: false,
                    });
                }
            }
        }
    }

    // Add filename segment
    if !original.filename.is_empty() {
        result.push(SegmentInfo {
            original: original.filename.clone(),
            shortened: shortened.filename.clone(),
            was_abbreviated: original.filename != shortened.filename,
            is_filename: true,
        });
    }

    result
}

// ── Convenience functions ────────────────────────────────────────────────────

/// Shorten a path using the Hybrid strategy with a target max length.
pub fn shrink_to(path: &str, max_len: usize) -> String {
    shrink(path, &ShrinkOptions::new(max_len))
}

/// Shorten a path using Fish-style abbreviation (no length target).
pub fn shrink_fish(path: &str) -> String {
    let info = path_info::PathInfo::parse(path, None);
    strategy::fish::shrink_fish(&info, 1, 0, &[])
}

/// Shorten a path using ellipsis with a target max length.
pub fn shrink_ellipsis(path: &str, max_len: usize) -> String {
    shrink(
        path,
        &ShrinkOptions::new(max_len).strategy(Strategy::Ellipsis),
    )
}

/// Shorten a path using unique-prefix disambiguation (no length target).
pub fn shrink_unique(path: &str) -> String {
    shrink(
        path,
        &ShrinkOptions::new(usize::MAX).strategy(Strategy::Unique),
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

    #[test]
    fn dir_length_two() {
        let opts = ShrinkOptions::new(50)
            .strategy(Strategy::Fish)
            .dir_length(2);
        let result = shrink("/home/john/projects/rust/myapp/src/lib.rs", &opts);
        assert_eq!(result, "/ho/jo/pr/ru/my/sr/lib.rs");
    }

    #[test]
    fn full_length_dirs_one() {
        let opts = ShrinkOptions::new(50)
            .strategy(Strategy::Fish)
            .full_length_dirs(1);
        let result = shrink("/home/john/projects/rust/myapp/src/lib.rs", &opts);
        assert_eq!(result, "/h/j/p/r/m/src/lib.rs");
    }

    #[test]
    fn full_length_dirs_two() {
        let opts = ShrinkOptions::new(50)
            .strategy(Strategy::Fish)
            .full_length_dirs(2);
        let result = shrink("/home/john/projects/rust/myapp/src/lib.rs", &opts);
        assert_eq!(result, "/h/j/p/r/myapp/src/lib.rs");
    }

    // ── Mapped locations tests ───────────────────────────────────────────

    #[test]
    fn mapped_location_tilde() {
        let opts = ShrinkOptions::new(50).map_location("/home/john", "~");
        let result = shrink("/home/john/projects/rust/lib.rs", &opts);
        assert_eq!(result, "~/projects/rust/lib.rs");
    }

    #[test]
    fn mapped_location_custom() {
        let opts = ShrinkOptions::new(50).map_location("/home/john/projects", "PROJ:");
        let result = shrink("/home/john/projects/rust/myapp/src/lib.rs", &opts);
        assert!(result.starts_with("PROJ:"), "got: {result}");
        assert!(result.ends_with("lib.rs"));
    }

    #[test]
    fn mapped_location_longest_match() {
        let opts = ShrinkOptions::new(50)
            .map_location("/home/john", "~")
            .map_location("/home/john/projects", "PROJ:");
        let result = shrink("/home/john/projects/rust/lib.rs", &opts);
        assert!(
            result.starts_with("PROJ:"),
            "longer match should win: {result}"
        );
    }

    #[test]
    fn mapped_location_no_match() {
        let opts = ShrinkOptions::new(50).map_location("/opt/data", "DATA:");
        let result = shrink("/home/john/file.rs", &opts);
        assert_eq!(result, "/home/john/file.rs");
    }

    #[test]
    fn mapped_location_windows() {
        let opts = ShrinkOptions::new(50).map_location("C:\\Users\\Admin", "~");
        let result = shrink("C:\\Users\\Admin\\Documents\\file.txt", &opts);
        assert!(result.starts_with("~"), "got: {result}");
        assert!(result.ends_with("file.txt"));
    }

    // ── Anchor segments tests ────────────────────────────────────────────

    #[test]
    fn anchor_preserves_segment_fish() {
        let opts = ShrinkOptions::new(50)
            .strategy(Strategy::Fish)
            .anchor("src");
        let result = shrink("/home/john/projects/rust/myapp/src/lib.rs", &opts);
        assert!(
            result.contains("/src/"),
            "should preserve anchored segment: {result}"
        );
        assert_eq!(result, "/h/j/p/r/m/src/lib.rs");
    }

    #[test]
    fn anchor_multiple() {
        let opts = ShrinkOptions::new(50)
            .strategy(Strategy::Fish)
            .anchor("src")
            .anchor("myapp");
        let result = shrink("/home/john/projects/rust/myapp/src/lib.rs", &opts);
        assert!(result.contains("myapp"), "got: {result}");
        assert!(result.contains("src"), "got: {result}");
    }

    #[test]
    fn anchor_in_hybrid() {
        let opts = ShrinkOptions::new(35).anchor("src");
        let result = shrink("/home/john/projects/rust/myapp/src/lib.rs", &opts);
        assert!(
            result.contains("src"),
            "anchor should survive hybrid: {result}"
        );
        assert!(result.len() <= 35, "got len {}: {result}", result.len());
    }

    #[test]
    fn anchor_no_match() {
        let opts = ShrinkOptions::new(50)
            .strategy(Strategy::Fish)
            .anchor("nonexistent");
        let result = shrink("/home/john/projects/lib.rs", &opts);
        assert_eq!(result, "/h/j/p/lib.rs");
    }

    #[test]
    fn convenience_shrink_unique() {
        let result = shrink_unique("/home/john/projects/rust/lib.rs");
        assert_eq!(result, "/h/j/p/r/lib.rs");
    }

    // ── Segment metadata tests ──────────────────────────────────────────

    #[test]
    fn segment_metadata_fish() {
        let opts = ShrinkOptions::new(usize::MAX).strategy(Strategy::Fish);
        let result = shrink_detailed("/home/john/projects/lib.rs", &opts);
        assert_eq!(result.segments.len(), 4); // home, john, projects, lib.rs

        assert_eq!(result.segments[0].original, "home");
        assert_eq!(result.segments[0].shortened, "h");
        assert!(result.segments[0].was_abbreviated);
        assert!(!result.segments[0].is_filename);

        assert_eq!(result.segments[1].original, "john");
        assert_eq!(result.segments[1].shortened, "j");

        assert_eq!(result.segments[3].original, "lib.rs");
        assert_eq!(result.segments[3].shortened, "lib.rs");
        assert!(!result.segments[3].was_abbreviated);
        assert!(result.segments[3].is_filename);
    }

    #[test]
    fn segment_metadata_no_truncation() {
        let result = shrink_detailed("/home/user/file.txt", &ShrinkOptions::new(50));
        assert_eq!(result.segments.len(), 3);
        assert!(!result.segments[0].was_abbreviated);
        assert!(!result.segments[1].was_abbreviated);
        assert!(result.segments[2].is_filename);
        assert_eq!(result.segments[2].original, "file.txt");
    }

    #[test]
    fn segment_metadata_filename_only() {
        let result = shrink_detailed("file.txt", &ShrinkOptions::new(50));
        assert_eq!(result.segments.len(), 1);
        assert!(result.segments[0].is_filename);
        assert_eq!(result.segments[0].original, "file.txt");
        assert!(!result.segments[0].was_abbreviated);
    }

    #[test]
    fn segment_metadata_empty() {
        let result = shrink_detailed("", &ShrinkOptions::new(50));
        assert!(result.segments.is_empty());
    }
}

use crate::path_info::{PathInfo, SegmentPriority};
use crate::platform;
use crate::strategy::fish::abbreviate_segment;

/// Apply hybrid shortening: graduated approach.
///
/// Phase 1: Fish-abbreviate expendable segments.
/// Phase 2: Fish-abbreviate context segments.
/// Phase 3: Collapse consecutive abbreviated segments into ellipsis.
/// Phase 4: Fish-abbreviate identity segments (last resort).
///
/// Never touches filename or prefix.
pub fn shrink_hybrid(
    info: &PathInfo,
    max_len: usize,
    ellipsis: &str,
    anchors: &[String],
) -> String {
    if info.segments.is_empty() {
        return info.reassemble(&[]);
    }

    // Full reassembly check
    let texts: Vec<String> = info.segments.iter().map(|s| s.text.clone()).collect();
    let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
    let full = info.reassemble(&text_refs);
    if full.len() <= max_len {
        return full;
    }

    // If filename alone exceeds max_len, return filename (sacred)
    if info.filename.len() >= max_len {
        return info.filename.clone();
    }

    // Working copy of segment texts
    let mut working: Vec<String> = texts;
    let priorities: Vec<SegmentPriority> = info.segments.iter().map(|s| s.priority).collect();

    // Phase 1: Fish expendable segments
    for (i, priority) in priorities.iter().enumerate() {
        if *priority == SegmentPriority::Expendable {
            working[i] = abbreviate_segment(&info.segments[i].text, 1, anchors);
        }
    }
    let refs: Vec<&str> = working.iter().map(|s| s.as_str()).collect();
    let result = info.reassemble(&refs);
    if result.len() <= max_len {
        return result;
    }

    // Phase 2: Fish context segments
    for (i, priority) in priorities.iter().enumerate() {
        if *priority == SegmentPriority::Context {
            working[i] = abbreviate_segment(&info.segments[i].text, 1, anchors);
        }
    }
    let refs: Vec<&str> = working.iter().map(|s| s.as_str()).collect();
    let result = info.reassemble(&refs);
    if result.len() <= max_len {
        return result;
    }

    // Phase 3: Collapse consecutive abbreviated (single-char) segments into ellipsis.
    // Keep segments nearest to identity (beginning) and filename (end).
    // Find runs of short segments (len <= 2) in the middle and replace with ellipsis.
    let n = working.len();
    if n > 2 {
        let collapsed = collapse_middle(&working, &priorities, info, max_len, ellipsis);
        if collapsed.len() <= max_len {
            return collapsed;
        }
    }

    // Phase 4: Fish identity segments (last resort)
    for (i, priority) in priorities.iter().enumerate() {
        if *priority == SegmentPriority::Identity {
            working[i] = abbreviate_segment(&info.segments[i].text, 1, anchors);
        }
    }

    // Try collapse again after identity is fished
    let n = working.len();
    if n > 2 {
        let collapsed = collapse_middle(&working, &priorities, info, max_len, ellipsis);
        if collapsed.len() <= max_len {
            return collapsed;
        }
    }

    // Final: just prefix + ellipsis + filename
    let sep = platform::separator(info.style);
    let sep_str = sep.to_string();
    let prefix_sep = if info.prefix.is_empty() { "" } else { &sep_str };
    let minimal = format!(
        "{}{}{}{}{}",
        info.prefix, prefix_sep, ellipsis, &sep_str, info.filename,
    );
    if minimal.len() <= max_len {
        return minimal;
    }

    // Absolute last resort: filename only
    info.filename.clone()
}

/// Collapse middle segments into an ellipsis, keeping head and tail segments.
fn collapse_middle(
    working: &[String],
    _priorities: &[SegmentPriority],
    info: &PathInfo,
    max_len: usize,
    ellipsis: &str,
) -> String {
    let n = working.len();

    // Try keeping more head segments and fewer tail, then adjust
    // Start with keeping 1 head + 1 tail, increase if budget allows
    for keep_tail in (0..=n.min(3)).rev() {
        for keep_head in (0..=n.min(3)).rev() {
            if keep_head + keep_tail >= n {
                continue;
            }

            let mut parts: Vec<&str> = Vec::new();
            for w in working.iter().take(keep_head) {
                parts.push(w);
            }
            parts.push(ellipsis);
            for w in working.iter().skip(n - keep_tail) {
                parts.push(w);
            }

            let result = info.reassemble(&parts);
            if result.len() <= max_len {
                return result;
            }
        }
    }

    // Nothing fit
    let sep = platform::separator(info.style);
    let sep_str = sep.to_string();
    let prefix_sep = if info.prefix.is_empty() { "" } else { &sep_str };
    format!(
        "{}{}{}{}{}",
        info.prefix, prefix_sep, ellipsis, &sep_str, info.filename,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn already_short() {
        let info = PathInfo::parse("/home/user/file.txt", None);
        assert_eq!(shrink_hybrid(&info, 50, "...", &[]), "/home/user/file.txt");
    }

    #[test]
    fn phase1_fish_expendable() {
        let info = PathInfo::parse("/home/john/projects/rust/myapp/src/main.rs", None);
        let result = shrink_hybrid(&info, 35, "...", &[]);
        // Should fish expendable segments first
        assert!(result.len() <= 35, "got len {}: {}", result.len(), result);
        assert!(result.ends_with("main.rs"));
        assert!(
            result.contains("john"),
            "should preserve identity: {result}"
        );
    }

    #[test]
    fn phase3_collapse() {
        let info = PathInfo::parse(
            "/home/john/projects/rust/myapp/src/deep/nested/main.rs",
            None,
        );
        let result = shrink_hybrid(&info, 30, "...", &[]);
        assert!(result.len() <= 30, "got len {}: {}", result.len(), result);
        assert!(result.ends_with("main.rs"));
    }

    #[test]
    fn windows_hybrid() {
        let info = PathInfo::parse(
            "C:\\Users\\Admin\\AppData\\Local\\Temp\\deep\\file.txt",
            None,
        );
        let result = shrink_hybrid(&info, 35, "...", &[]);
        assert!(result.len() <= 35, "got len {}: {}", result.len(), result);
        assert!(result.ends_with("file.txt"));
        // Should try to preserve Admin
        assert!(
            result.contains("Admin") || result.contains("A"),
            "should preserve identity somehow: {result}",
        );
    }

    #[test]
    fn tilde_hybrid() {
        let info = PathInfo::parse("~/projects/rust/app/src/lib.rs", None);
        let result = shrink_hybrid(&info, 25, "...", &[]);
        assert!(result.len() <= 25, "got len {}: {}", result.len(), result);
        assert!(result.ends_with("lib.rs"));
    }

    #[test]
    fn filename_exceeds() {
        let info = PathInfo::parse("/a/b/c/very_long_filename.txt", None);
        let result = shrink_hybrid(&info, 10, "...", &[]);
        assert_eq!(result, "very_long_filename.txt");
    }

    #[test]
    fn macos_app_support() {
        let info = PathInfo::parse(
            "/Users/john/Library/Application Support/Code/User/settings.json",
            None,
        );
        let result = shrink_hybrid(&info, 45, "...", &[]);
        assert!(result.len() <= 45, "got len {}: {}", result.len(), result);
        assert!(result.ends_with("settings.json"));
        assert!(
            result.contains("john"),
            "should preserve identity: {result}"
        );
    }

    #[test]
    fn dot_backslash() {
        let info = PathInfo::parse(
            ".\\Users\\Admin\\AppData\\Local\\Packages\\Microsoft.MicrosoftEdge\\file.txt",
            None,
        );
        let result = shrink_hybrid(&info, 40, "...", &[]);
        assert!(result.len() <= 40, "got len {}: {}", result.len(), result);
        assert!(result.ends_with("file.txt"));
    }
}

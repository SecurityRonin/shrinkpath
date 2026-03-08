use crate::path_info::{PathInfo, SegmentPriority};
use crate::platform;

/// Apply ellipsis-style shortening: replace middle segments with `...`.
///
/// Priorities when fitting into `max_len`:
/// 1. Filename is sacred (never truncated)
/// 2. Identity segments (username) preserved when possible
/// 3. Segments nearest to the filename are kept next (more context)
/// 4. Middle segments are collapsed into the ellipsis string
pub fn shrink_ellipsis(info: &PathInfo, max_len: usize, ellipsis: &str) -> String {
    // If no segments, just return prefix + filename
    if info.segments.is_empty() {
        return info.reassemble(&[]);
    }

    let sep = platform::separator(info.style);
    let sep_len = 1; // separator is always 1 char

    // Full reassembly
    let texts: Vec<&str> = info.segments.iter().map(|s| s.text.as_str()).collect();
    let full = info.reassemble(&texts);
    if full.len() <= max_len {
        return full;
    }

    // Base cost: prefix + separator + filename
    let base_len = info.prefix.len()
        + if !info.prefix.is_empty() && !info.filename.is_empty() {
            sep_len
        } else {
            0
        }
        + info.filename.len();

    // If filename alone exceeds max_len, return filename (sacred)
    if info.filename.len() >= max_len {
        return info.filename.clone();
    }

    // Try: prefix + head segments (up to and including identity) + ... + trailing + filename
    // Find the last identity segment index, then keep everything up to and including it.
    let identity_end = info
        .segments
        .iter()
        .enumerate()
        .filter(|(_, s)| s.priority == SegmentPriority::Identity)
        .map(|(i, _)| i + 1)
        .next_back()
        .unwrap_or(0);

    // Greedily add segments from the right (closest to filename)
    let head = &info.segments[..identity_end];
    let tail_candidates = &info.segments[identity_end..];

    // Compute head cost
    let head_cost: usize = head.iter().map(|s| s.text.len() + sep_len).sum();

    // Cost of the ellipsis marker
    let ellipsis_cost = ellipsis.len() + sep_len; // "..." + separator

    // Available budget for tail segments
    let fixed_cost = base_len + head_cost + ellipsis_cost;

    if fixed_cost >= max_len {
        // Can't even fit head + ellipsis + filename
        // Try without head: just prefix + ... + filename
        let sep_str = sep.to_string();
        let prefix_sep = if info.prefix.is_empty() { "" } else { &sep_str };
        let minimal = format!(
            "{}{}{}{}{}",
            info.prefix, prefix_sep, ellipsis, &sep_str, info.filename
        );
        if minimal.len() <= max_len {
            return minimal;
        }
        // Last resort: just filename
        return info.filename.clone();
    }

    let budget = max_len - fixed_cost;

    // Greedily add tail segments from right to left
    let mut tail_count = 0;
    let mut tail_len = 0;
    for seg in tail_candidates.iter().rev() {
        let cost = seg.text.len() + sep_len;
        if tail_len + cost <= budget {
            tail_len += cost;
            tail_count += 1;
        } else {
            break;
        }
    }

    // Build the result
    let mut parts: Vec<&str> = Vec::new();
    for seg in head {
        parts.push(&seg.text);
    }
    parts.push(ellipsis);
    let tail_start = tail_candidates.len() - tail_count;
    for seg in &tail_candidates[tail_start..] {
        parts.push(&seg.text);
    }

    info.reassemble(&parts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn already_short() {
        let info = PathInfo::parse("/home/user/file.txt", None);
        assert_eq!(shrink_ellipsis(&info, 50, "..."), "/home/user/file.txt");
    }

    #[test]
    fn basic_ellipsis() {
        let info = PathInfo::parse("/home/john/projects/rust/myapp/src/lib.rs", None);
        let result = shrink_ellipsis(&info, 30, "...");
        assert!(result.len() <= 30, "got len {}: {}", result.len(), result);
        assert!(result.ends_with("lib.rs"));
        assert!(result.contains("..."));
    }

    #[test]
    fn preserves_identity() {
        let info = PathInfo::parse("/home/john/projects/rust/myapp/src/lib.rs", None);
        let result = shrink_ellipsis(&info, 35, "...");
        assert!(
            result.contains("john"),
            "should preserve identity: {result}"
        );
    }

    #[test]
    fn windows_ellipsis() {
        let info = PathInfo::parse("C:\\Users\\Admin\\AppData\\Local\\Temp\\file.txt", None);
        let result = shrink_ellipsis(&info, 30, "...");
        assert!(result.len() <= 30, "got len {}: {}", result.len(), result);
        assert!(result.ends_with("file.txt"));
    }

    #[test]
    fn filename_exceeds_maxlen() {
        let info = PathInfo::parse(
            "/a/b/c/very_long_filename_that_exceeds_everything.txt",
            None,
        );
        let result = shrink_ellipsis(&info, 10, "...");
        assert_eq!(result, "very_long_filename_that_exceeds_everything.txt");
    }

    #[test]
    fn filename_only() {
        let info = PathInfo::parse("file.txt", None);
        let result = shrink_ellipsis(&info, 5, "...");
        assert_eq!(result, "file.txt");
    }

    #[test]
    fn keeps_right_segments() {
        let info = PathInfo::parse("/home/john/a/b/c/d/e/src/lib.rs", None);
        // max_len=25 forces ellipsis (original is 31 chars)
        let result = shrink_ellipsis(&info, 25, "...");
        // Should keep segments nearest to filename
        assert!(result.len() <= 25, "got len {}: {}", result.len(), result);
        assert!(result.ends_with("lib.rs"));
        assert!(result.contains("..."));
    }
}

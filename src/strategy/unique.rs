use crate::path_info::PathInfo;

/// Extract the comparable part of a segment for prefix matching.
/// For dot-prefixed segments, returns the part after the dot.
/// For normal segments, returns the full text.
fn comparable_text(text: &str) -> &str {
    if text.starts_with('.') && text.len() > 1 {
        &text[1..]
    } else {
        text
    }
}

/// Find the shortest prefix length that makes `target` unique among `others`.
/// Returns `None` if `target` is identical to any element in `others` (can't disambiguate).
/// The returned length refers to chars in the comparable portion (after dot for dot-prefixed).
fn unique_prefix_len(target: &str, others: &[&str]) -> Option<usize> {
    let target_cmp = comparable_text(target);
    let other_cmps: Vec<&str> = others.iter().map(|o| comparable_text(o)).collect();

    // If any other segment has the same comparable text, we can't disambiguate
    if other_cmps.contains(&target_cmp) {
        return None;
    }

    let target_chars: Vec<char> = target_cmp.chars().collect();

    // Find the minimum prefix length that distinguishes from all others
    for len in 1..=target_chars.len() {
        let prefix: String = target_chars[..len].iter().collect();
        let is_unique = other_cmps.iter().all(|other| {
            let other_chars: Vec<char> = other.chars().collect();
            if other_chars.len() < len {
                // Other is shorter than our prefix, so our prefix can't match it
                true
            } else {
                let other_prefix: String = other_chars[..len].iter().collect();
                prefix != other_prefix
            }
        });
        if is_unique {
            return Some(len);
        }
    }

    // Full length needed (shouldn't happen if no duplicates, but just in case)
    Some(target_chars.len())
}

/// Build the abbreviated form of a segment given the unique prefix length.
fn abbreviate_with_len(text: &str, prefix_len: usize) -> String {
    if text.starts_with('.') && text.len() > 1 {
        let after_dot: String = text[1..].chars().take(prefix_len).collect();
        format!(".{after_dot}")
    } else {
        text.chars().take(prefix_len).collect()
    }
}

/// Shrink a path by finding the shortest unique prefix for each directory segment
/// among all segments in the same path. NOT filesystem-aware.
///
/// - Filename is never abbreviated (sacred).
/// - Dot-prefixed segments: compare chars after the dot, then prepend dot.
/// - If a segment's first char is unique among all others, use 1 char.
/// - Identical segments cannot be disambiguated; they are kept in full.
/// - Anchored segments are never abbreviated.
pub fn shrink_unique(info: &PathInfo, anchors: &[String]) -> String {
    let texts: Vec<&str> = info.segments.iter().map(|s| s.text.as_str()).collect();

    let abbreviated: Vec<String> = texts
        .iter()
        .enumerate()
        .map(|(i, &text)| {
            // Anchored segments are never abbreviated
            if anchors.iter().any(|a| a == text) {
                return text.to_string();
            }

            // Collect all OTHER segment texts
            let others: Vec<&str> = texts
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, &t)| t)
                .collect();

            match unique_prefix_len(text, &others) {
                Some(len) => abbreviate_with_len(text, len),
                None => text.to_string(), // Can't disambiguate, keep full
            }
        })
        .collect();

    let refs: Vec<&str> = abbreviated.iter().map(|s| s.as_str()).collect();
    info.reassemble(&refs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path_info::PathInfo;

    #[test]
    fn all_unique_first_chars() {
        let info = PathInfo::parse("/home/john/projects/rust/lib.rs", None);
        let result = shrink_unique(&info, &[]);
        assert_eq!(result, "/h/j/p/r/lib.rs");
    }

    #[test]
    fn identical_segments_kept_full() {
        let info = PathInfo::parse("/dev/dev/dev/file.txt", None);
        let result = shrink_unique(&info, &[]);
        assert_eq!(result, "/dev/dev/dev/file.txt");
    }

    #[test]
    fn partial_disambiguation() {
        let info = PathInfo::parse("/home/documents/downloads/file.txt", None);
        let result = shrink_unique(&info, &[]);
        // h is unique. documents vs downloads: "doc" vs "dow"
        assert_eq!(result, "/h/doc/dow/file.txt");
    }

    #[test]
    fn dot_prefixed_disambiguation() {
        let info = PathInfo::parse("/home/user/.config/.cache/file.txt", None);
        let result = shrink_unique(&info, &[]);
        // home→h, user→u unique. .config vs .cache: after dot, "co" vs "ca"
        assert_eq!(result, "/h/u/.co/.ca/file.txt");
    }

    #[test]
    fn windows_unique() {
        let info = PathInfo::parse("C:\\Users\\Admin\\AppData\\Application\\file.txt", None);
        let result = shrink_unique(&info, &[]);
        // Users→U unique. Admin shares 'A' with AppData/Application → "Ad".
        // AppData vs Application: "AppD" vs "Appl"
        assert_eq!(result, "C:\\U\\Ad\\AppD\\Appl\\file.txt");
    }

    #[test]
    fn single_segment() {
        let info = PathInfo::parse("/home/file.txt", None);
        let result = shrink_unique(&info, &[]);
        assert_eq!(result, "/h/file.txt");
    }

    #[test]
    fn anchored_segment_preserved() {
        let info = PathInfo::parse("/home/john/src/lib.rs", None);
        let anchors = vec!["src".to_string()];
        let result = shrink_unique(&info, &anchors);
        assert_eq!(result, "/h/j/src/lib.rs");
    }
}

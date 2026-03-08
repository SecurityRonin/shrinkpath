use crate::path_info::PathInfo;
use crate::platform;

/// Abbreviate a segment to its first `len` characters.
/// Dot-prefixed segments keep the dot + `len` chars after: `.config` with len=1 -> `.c`
/// If the segment text matches any anchor, it is returned unchanged.
pub fn abbreviate_segment(text: &str, len: usize, anchors: &[String]) -> String {
    if text.is_empty() {
        return String::new();
    }
    // If this segment is an anchor, never abbreviate it
    if anchors.iter().any(|a| a == text) {
        return text.to_string();
    }
    let mut chars = text.chars();
    let first = chars.next().unwrap();
    if first == '.' {
        let after_dot: String = chars.take(len).collect();
        if after_dot.is_empty() {
            return ".".to_string();
        }
        return format!(".{after_dot}");
    }
    // Take `len` chars total (first + len-1 more)
    let mut result = String::with_capacity(len);
    result.push(first);
    for c in chars.take(len - 1) {
        result.push(c);
    }
    result
}

/// Apply fish-style abbreviation: all directory segments become first-char.
/// Filename is never abbreviated.
/// `dir_length`: number of characters to keep per abbreviated segment.
/// `full_length_dirs`: number of trailing directory segments to keep unabbreviated.
/// `anchors`: segment names that should never be abbreviated.
pub fn shrink_fish(
    info: &PathInfo,
    dir_length: usize,
    full_length_dirs: usize,
    anchors: &[String],
) -> String {
    let seg_count = info.segments.len();
    let abbreviated: Vec<String> = info
        .segments
        .iter()
        .enumerate()
        .map(|(i, s)| {
            if full_length_dirs > 0 && i >= seg_count.saturating_sub(full_length_dirs) {
                s.text.clone()
            } else {
                abbreviate_segment(&s.text, dir_length, anchors)
            }
        })
        .collect();

    let sep = platform::separator(info.style);
    let mut result = info.prefix.clone();

    for (i, text) in abbreviated.iter().enumerate() {
        if i > 0 || (!result.is_empty() && !result.ends_with(sep)) {
            result.push(sep);
        }
        result.push_str(text);
    }

    if !info.filename.is_empty() {
        if !result.is_empty() && !result.ends_with(sep) {
            result.push(sep);
        }
        result.push_str(&info.filename);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abbreviate_normal() {
        assert_eq!(abbreviate_segment("Users", 1, &[]), "U");
        assert_eq!(abbreviate_segment("projects", 1, &[]), "p");
    }

    #[test]
    fn abbreviate_dotfile() {
        assert_eq!(abbreviate_segment(".config", 1, &[]), ".c");
        assert_eq!(abbreviate_segment(".local", 1, &[]), ".l");
        assert_eq!(abbreviate_segment(".", 1, &[]), ".");
    }

    #[test]
    fn abbreviate_empty() {
        assert_eq!(abbreviate_segment("", 1, &[]), "");
    }

    #[test]
    fn fish_unix() {
        let info = PathInfo::parse("/home/john/projects/rust/myapp/src/lib.rs", None);
        assert_eq!(shrink_fish(&info, 1, 0, &[]), "/h/j/p/r/m/s/lib.rs");
    }

    #[test]
    fn fish_dotfiles() {
        let info = PathInfo::parse("/home/john/.config/nvim/init.lua", None);
        assert_eq!(shrink_fish(&info, 1, 0, &[]), "/h/j/.c/n/init.lua");
    }

    #[test]
    fn fish_windows() {
        let info = PathInfo::parse("C:\\Users\\john\\AppData\\Local\\Temp\\file.txt", None);
        assert_eq!(shrink_fish(&info, 1, 0, &[]), "C:\\U\\j\\A\\L\\T\\file.txt");
    }

    #[test]
    fn fish_tilde() {
        let info = PathInfo::parse("~/projects/rust/file.rs", None);
        assert_eq!(shrink_fish(&info, 1, 0, &[]), "~/p/r/file.rs");
    }

    #[test]
    fn fish_filename_only() {
        let info = PathInfo::parse("file.txt", None);
        assert_eq!(shrink_fish(&info, 1, 0, &[]), "file.txt");
    }

    #[test]
    fn fish_unc() {
        let info = PathInfo::parse("\\\\server\\share\\dept\\project\\file.xlsx", None);
        assert_eq!(
            shrink_fish(&info, 1, 0, &[]),
            "\\\\server\\share\\d\\p\\file.xlsx"
        );
    }

    #[test]
    fn fish_unicode() {
        let info = PathInfo::parse("/home/user/Schone/Musik/file.mp3", None);
        assert_eq!(shrink_fish(&info, 1, 0, &[]), "/h/u/S/M/file.mp3");
    }

    #[test]
    fn abbreviate_segment_multi_char() {
        assert_eq!(abbreviate_segment("projects", 2, &[]), "pr");
        assert_eq!(abbreviate_segment("projects", 3, &[]), "pro");
        assert_eq!(abbreviate_segment("projects", 1, &[]), "p");
        assert_eq!(abbreviate_segment(".config", 2, &[]), ".co");
        assert_eq!(abbreviate_segment(".config", 1, &[]), ".c");
        assert_eq!(abbreviate_segment("a", 3, &[]), "a"); // shorter than N
    }
}

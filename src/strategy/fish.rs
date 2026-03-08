use crate::path_info::PathInfo;
use crate::platform;

/// Abbreviate a segment to its first character.
/// Dot-prefixed segments keep the dot + first char: `.config` -> `.c`
pub fn abbreviate_segment(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let mut chars = text.chars();
    let first = chars.next().unwrap();
    if first == '.' {
        if let Some(second) = chars.next() {
            return format!(".{second}");
        }
        return ".".to_string();
    }
    first.to_string()
}

/// Apply fish-style abbreviation: all directory segments become first-char.
/// Filename is never abbreviated.
pub fn shrink_fish(info: &PathInfo) -> String {
    let abbreviated: Vec<String> = info
        .segments
        .iter()
        .map(|s| abbreviate_segment(&s.text))
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
        assert_eq!(abbreviate_segment("Users"), "U");
        assert_eq!(abbreviate_segment("projects"), "p");
    }

    #[test]
    fn abbreviate_dotfile() {
        assert_eq!(abbreviate_segment(".config"), ".c");
        assert_eq!(abbreviate_segment(".local"), ".l");
        assert_eq!(abbreviate_segment("."), ".");
    }

    #[test]
    fn abbreviate_empty() {
        assert_eq!(abbreviate_segment(""), "");
    }

    #[test]
    fn fish_unix() {
        let info = PathInfo::parse("/home/john/projects/rust/myapp/src/lib.rs", None);
        assert_eq!(shrink_fish(&info), "/h/j/p/r/m/s/lib.rs");
    }

    #[test]
    fn fish_dotfiles() {
        let info = PathInfo::parse("/home/john/.config/nvim/init.lua", None);
        assert_eq!(shrink_fish(&info), "/h/j/.c/n/init.lua");
    }

    #[test]
    fn fish_windows() {
        let info = PathInfo::parse("C:\\Users\\john\\AppData\\Local\\Temp\\file.txt", None);
        assert_eq!(shrink_fish(&info), "C:\\U\\j\\A\\L\\T\\file.txt");
    }

    #[test]
    fn fish_tilde() {
        let info = PathInfo::parse("~/projects/rust/file.rs", None);
        assert_eq!(shrink_fish(&info), "~/p/r/file.rs");
    }

    #[test]
    fn fish_filename_only() {
        let info = PathInfo::parse("file.txt", None);
        assert_eq!(shrink_fish(&info), "file.txt");
    }

    #[test]
    fn fish_unc() {
        let info = PathInfo::parse("\\\\server\\share\\dept\\project\\file.xlsx", None);
        assert_eq!(shrink_fish(&info), "\\\\server\\share\\d\\p\\file.xlsx");
    }

    #[test]
    fn fish_unicode() {
        let info = PathInfo::parse("/home/user/Schone/Musik/file.mp3", None);
        assert_eq!(shrink_fish(&info), "/h/u/S/M/file.mp3");
    }
}

use crate::platform::{self, PathStyle};

/// Priority of a path segment for truncation decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SegmentPriority {
    /// Filename: never shrink.
    Sacred = 0,
    /// User identity segment (username/profile name): preserve when possible.
    Identity = 1,
    /// Well-known application/service context: nice to have.
    Context = 2,
    /// Generic intermediate directory: first to be shortened.
    Expendable = 3,
}

/// A single path segment with its text and computed priority.
#[derive(Debug, Clone)]
pub struct Segment {
    pub text: String,
    pub priority: SegmentPriority,
}

/// Parsed representation of a path.
#[derive(Debug, Clone)]
pub struct PathInfo {
    /// Prefix: root (`/`, `C:\`, `\\server\share\`, `.`, `~`, or empty).
    pub prefix: String,
    /// Directory segments between prefix and filename.
    pub segments: Vec<Segment>,
    /// The final component (filename). Sacred, never truncated.
    pub filename: String,
    /// Detected or forced path style.
    pub style: PathStyle,
}

impl PathInfo {
    /// Parse a path string into structured components with priority classification.
    pub fn parse(path: &str, force_style: Option<PathStyle>) -> Self {
        let style = force_style.unwrap_or_else(|| platform::detect_style(path));
        let sep = platform::separator(style);

        // Normalize: replace the "other" separator with the canonical one
        let normalized: String = match style {
            PathStyle::Unix => path.replace('\\', "/"),
            PathStyle::Windows => path.replace('/', "\\"),
        };

        let (prefix, remainder) = extract_prefix(&normalized, style);

        // Split remaining path into parts
        let parts: Vec<&str> = remainder.split(sep).filter(|s| !s.is_empty()).collect();

        if parts.is_empty() {
            return PathInfo {
                prefix,
                segments: Vec::new(),
                filename: String::new(),
                style,
            };
        }

        // Last part is the filename (sacred)
        let filename = parts.last().unwrap().to_string();
        let dir_parts = &parts[..parts.len() - 1];

        // Classify segments
        let segments = classify_segments(dir_parts, &prefix, style);

        PathInfo {
            prefix,
            segments,
            filename,
            style,
        }
    }

    /// Reassemble the path from prefix + segments + filename using the given separator.
    pub fn reassemble(&self, segment_texts: &[&str]) -> String {
        let sep = platform::separator(self.style);
        let mut result = self.prefix.clone();

        for (i, text) in segment_texts.iter().enumerate() {
            if i > 0 || (!result.is_empty() && !result.ends_with(sep)) {
                result.push(sep);
            }
            result.push_str(text);
        }

        if !self.filename.is_empty() {
            if !result.is_empty() && !result.ends_with(sep) {
                result.push(sep);
            }
            result.push_str(&self.filename);
        }

        result
    }
}

/// Extract the prefix (root portion) from a normalized path.
fn extract_prefix(path: &str, style: PathStyle) -> (String, &str) {
    match style {
        PathStyle::Windows => {
            let bytes = path.as_bytes();
            // UNC: \\server\share
            if bytes.starts_with(b"\\\\") {
                let after_slashes = &path[2..];
                // Find server
                if let Some(server_end) = after_slashes.find('\\') {
                    let after_server = &after_slashes[server_end + 1..];
                    // Find share
                    let share_end = after_server.find('\\').unwrap_or(after_server.len());
                    let prefix_end = 2 + server_end + 1 + share_end;
                    let prefix = &path[..prefix_end];
                    let remainder = if prefix_end < path.len() {
                        &path[prefix_end + 1..]
                    } else {
                        ""
                    };
                    return (prefix.to_string(), remainder);
                }
                return (path.to_string(), "");
            }
            // Drive letter: C:\
            if bytes.len() >= 3
                && bytes[0].is_ascii_alphabetic()
                && bytes[1] == b':'
                && bytes[2] == b'\\'
            {
                return (path[..3].to_string(), &path[3..]);
            }
            // Dot-backslash: .\
            if bytes.starts_with(b".\\") {
                return (".".to_string(), &path[2..]);
            }
            // Just backslash
            if bytes.starts_with(b"\\") {
                return ("\\".to_string(), &path[1..]);
            }
            (String::new(), path)
        }
        PathStyle::Unix => {
            let bytes = path.as_bytes();
            // Absolute: /
            if bytes.starts_with(b"/") {
                return ("/".to_string(), &path[1..]);
            }
            // Tilde: ~/
            if bytes.starts_with(b"~/") {
                return ("~".to_string(), &path[2..]);
            }
            if path == "~" {
                return ("~".to_string(), "");
            }
            (String::new(), path)
        }
    }
}

/// Classify directory segments by priority.
fn classify_segments(parts: &[&str], prefix: &str, style: PathStyle) -> Vec<Segment> {
    let home_roots = match style {
        PathStyle::Unix => platform::UNIX_HOME_ROOTS,
        PathStyle::Windows => platform::WIN_HOME_ROOTS,
    };

    // Find the identity segment index: the segment AFTER a home root
    let identity_idx = parts.iter().enumerate().find_map(|(i, &part)| {
        if i == 0
            && home_roots
                .iter()
                .any(|&root| part.eq_ignore_ascii_case(root))
        {
            // The next segment (i+1) is the username
            if i + 1 < parts.len() {
                return Some(i + 1);
            }
        }
        None
    });

    // Special case: tilde prefix means no explicit identity segment in parts
    // (the ~ already encodes the user)

    parts
        .iter()
        .enumerate()
        .map(|(i, &text)| {
            let priority = if Some(i) == identity_idx {
                SegmentPriority::Identity
            } else if i == 0
                && home_roots
                    .iter()
                    .any(|&root| text.eq_ignore_ascii_case(root))
            {
                // "Users" or "home" segment itself — keep it (Context)
                SegmentPriority::Context
            } else if is_well_known_prefix(prefix) && i == 0 {
                // First segment after a well-known prefix like C:\Windows
                SegmentPriority::Context
            } else {
                SegmentPriority::Expendable
            };

            Segment {
                text: text.to_string(),
                priority,
            }
        })
        .collect()
}

/// Check if the prefix indicates a well-known system path.
fn is_well_known_prefix(_prefix: &str) -> bool {
    false // Conservative: don't over-classify
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_unix_home() {
        let info = PathInfo::parse("/home/john/projects/rust/file.rs", None);
        assert_eq!(info.prefix, "/");
        assert_eq!(info.filename, "file.rs");
        assert_eq!(info.segments.len(), 4);
        assert_eq!(info.segments[0].text, "home");
        assert_eq!(info.segments[0].priority, SegmentPriority::Context);
        assert_eq!(info.segments[1].text, "john");
        assert_eq!(info.segments[1].priority, SegmentPriority::Identity);
        assert_eq!(info.segments[2].priority, SegmentPriority::Expendable);
    }

    #[test]
    fn parse_windows_drive() {
        let info = PathInfo::parse("C:\\Users\\Admin\\AppData\\Local\\file.txt", None);
        assert_eq!(info.prefix, "C:\\");
        assert_eq!(info.filename, "file.txt");
        assert_eq!(info.segments[0].text, "Users");
        assert_eq!(info.segments[0].priority, SegmentPriority::Context);
        assert_eq!(info.segments[1].text, "Admin");
        assert_eq!(info.segments[1].priority, SegmentPriority::Identity);
    }

    #[test]
    fn parse_unc() {
        let info = PathInfo::parse("\\\\server\\share\\dept\\file.xlsx", None);
        assert_eq!(info.prefix, "\\\\server\\share");
        assert_eq!(info.filename, "file.xlsx");
        assert_eq!(info.segments.len(), 1);
        assert_eq!(info.segments[0].text, "dept");
    }

    #[test]
    fn parse_tilde() {
        let info = PathInfo::parse("~/projects/rust/file.rs", None);
        assert_eq!(info.prefix, "~");
        assert_eq!(info.filename, "file.rs");
        assert_eq!(info.segments.len(), 2);
    }

    #[test]
    fn parse_dot_backslash() {
        let info = PathInfo::parse(".\\Users\\Admin\\file.txt", None);
        assert_eq!(info.prefix, ".");
        assert_eq!(info.style, PathStyle::Windows);
        assert_eq!(info.filename, "file.txt");
    }

    #[test]
    fn parse_empty() {
        let info = PathInfo::parse("", None);
        assert_eq!(info.prefix, "");
        assert_eq!(info.filename, "");
        assert!(info.segments.is_empty());
    }

    #[test]
    fn parse_filename_only() {
        let info = PathInfo::parse("file.txt", None);
        assert_eq!(info.filename, "file.txt");
        assert!(info.segments.is_empty());
    }

    #[test]
    fn reassemble() {
        let info = PathInfo::parse("/home/john/projects/file.rs", None);
        let texts: Vec<&str> = info.segments.iter().map(|s| s.text.as_str()).collect();
        let result = info.reassemble(&texts);
        assert_eq!(result, "/home/john/projects/file.rs");
    }

    #[test]
    fn reassemble_windows() {
        let info = PathInfo::parse("C:\\Users\\Admin\\Docs\\file.txt", None);
        let texts: Vec<&str> = info.segments.iter().map(|s| s.text.as_str()).collect();
        let result = info.reassemble(&texts);
        assert_eq!(result, "C:\\Users\\Admin\\Docs\\file.txt");
    }
}

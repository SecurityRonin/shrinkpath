pub mod ellipsis;
pub mod fish;
pub mod hybrid;

/// Shortening strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    /// Replace middle segments with `...`: `/Users/john/.../file.txt`
    Ellipsis,
    /// Abbreviate intermediate dirs to first char: `/U/j/p/r/file.txt`
    Fish,
    /// Graduated approach: fish expendable segments first, then ellipsis, then fish identity.
    Hybrid,
}

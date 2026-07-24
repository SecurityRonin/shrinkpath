# 2. Segment priority model — filename sacred, identity preserved last

Date: 2026-07-24
Status: Accepted

## Context

Any path-shortening scheme must answer one question: when space runs out, *what
gets dropped first?* A naive left-to-right or middle-out truncation destroys the
two pieces a reader most needs — the filename (which file is this?) and the
username/profile segment (whose file is this?). The whole value of shrinkpath over
a plain `str[..n]` truncation is that it keeps identity while sacrificing
noise.

## Decision

Parse every path into `prefix + segments + filename`, and classify each directory
segment by an ordered `SegmentPriority` that drives all four strategies:

| Priority | Ordinal | Meaning | When dropped |
|---|---|---|---|
| `Sacred` | 0 | the filename | never |
| `Identity` | 1 | username / profile (segment after a home root, or implied by `~`) | last |
| `Context` | 2 | home root, well-known dirs (`home`, `Users`) | middle |
| `Expendable` | 3 | every other intermediate directory | first |

- The **filename is never truncated.** If the filename alone exceeds the target
  length, it is returned whole rather than cut.
- **Identity is recognized structurally**, not by a name allow-list: the segment
  immediately following a home root (`home` on Unix, `Users` on either) is the
  identity segment, and a `~` prefix encodes identity implicitly.
- The ordinal ordering (`derive(PartialOrd, Ord)`) lets strategies shorten
  higher-ordinal (more expendable) segments before lower ones.

## Consequences

- Shortening degrades in a predictable, information-preserving order across all
  strategies: expendable dirs blur first, then context, and identity only as a
  last resort — exactly the Hybrid phase order (ADR 0003).
- Identity detection is a general rule (position after a home root), not a
  hard-coded username list, so it works for arbitrary usernames and both
  platforms without special cases.
- The `Segment { text, priority }` split and the `PathInfo { prefix, segments,
  filename, style }` structure are the shared substrate every strategy reads, so a
  new strategy only decides *how* to abbreviate, never *what to keep*.
- Grounding: `src/path_info.rs` (`SegmentPriority`, `Segment`, `PathInfo::parse`,
  `classify_segments`); `src/platform.rs` (`UNIX_HOME_ROOTS`, `WIN_HOME_ROOTS`);
  README "Segment Priority" table.

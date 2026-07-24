# 4. Detect path style from the input string, not the host OS

Date: 2026-07-24
Status: Accepted

## Context

A forensic or cross-platform tool routinely handles paths that did not originate
on the machine it runs on: a Windows `C:\Users\...` path shown on a Linux
analysis workstation, a UNC `\\server\share\...` path in a report generated on
macOS. If path-style handling were keyed off `cfg!(windows)` or the runtime OS,
those paths would be mis-parsed — backslashes split wrong, drive letters mistaken
for segments, identity detection broken.

## Decision

Auto-detect the path style (`PathStyle::Unix` vs `PathStyle::Windows`) purely from
the **content of the input string**, independent of the host OS, using ordered
heuristics in `platform::detect_style`:

1. leading `\\` → Windows (UNC),
2. `X:\` or `X:/` drive letter → Windows,
3. leading `.\` → Windows (dot-relative),
4. contains `\` and no `/` → Windows,
5. otherwise → Unix (the default, also covering `~/…` and bare relative paths).

The detected style selects the separator and normalizes the "other" separator
before parsing. A caller may override detection with
`ShrinkOptions::path_style(...)` when the input is ambiguous.

## Consequences

- The same binary shortens a Windows path correctly on Linux and a Unix path
  correctly on Windows — essential for a display helper used across a
  cross-platform fleet.
- Detection is heuristic and therefore fallible on genuinely ambiguous strings
  (e.g. a Unix filename containing a backslash); the explicit `path_style`
  override is the escape hatch, and the default (Unix) is the safe fallthrough.
- The style is captured once in `PathInfo.style` and threaded through parsing and
  reassembly, so no strategy re-detects or re-guesses.
- Grounding: `src/platform.rs` (`detect_style`, `separator`, `PathStyle`);
  `src/path_info.rs` (`PathInfo::parse` normalizes on the detected separator);
  README "Platform Support" table and "The path style ... is auto-detected".

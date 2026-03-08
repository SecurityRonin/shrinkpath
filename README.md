# shrinkpath

**Smart cross-platform path shortening for Rust.**

[![Crates.io](https://img.shields.io/crates/v/shrinkpath.svg)](https://crates.io/crates/shrinkpath)
[![docs.rs](https://img.shields.io/docsrs/shrinkpath)](https://docs.rs/shrinkpath)
[![CI](https://github.com/SecurityRonin/shrinkpath/actions/workflows/ci.yml/badge.svg)](https://github.com/SecurityRonin/shrinkpath/actions)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/shrinkpath.svg)](#license)
[![Sponsor](https://img.shields.io/badge/sponsor-%E2%9D%A4-pink?logo=github)](https://github.com/sponsors/h4x0r)

```
/Users/john/Library/Application Support/Code/User/settings.json  (62 chars)
                            ↓  shrink_to(path, 35)
             /Users/john/.../User/settings.json                   (35 chars)
```

Filename is never truncated. Username is preserved when possible. Everything else
adapts to fit your target length.

## Why shrinkpath?

- **Target-length guarantee** &mdash; `shrink_to(path, 30)` always returns &le; 30 chars (unless the filename itself is longer &mdash; filenames are sacred)
- **Identity-preserving** &mdash; the username/profile segment is the last thing to go, not the first
- **Cross-platform** &mdash; handles `/home/...`, `~/...`, `C:\Users\...`, `\\server\share\...`, `.\...` from any host OS
- **Zero dependencies** &mdash; the library has no dependencies; `clap` is only pulled in for the optional CLI
- **No filesystem access** &mdash; works on path strings alone, so it runs in WASM, embedded, or anywhere (opt-in `fs` feature for filesystem-aware disambiguation)

## Quick Start

```toml
# Cargo.toml
[dependencies]
shrinkpath = "0.1"
```

```rust
use shrinkpath::{shrink_to, shrink_fish};

// Hybrid strategy: graduated shortening to fit a target length
let short = shrink_to("/home/john/projects/rust/myapp/src/lib.rs", 30);
assert!(short.len() <= 30);
assert!(short.ends_with("lib.rs"));

// Fish strategy: every directory segment becomes its first character
let fish = shrink_fish("/home/john/projects/rust/myapp/src/lib.rs");
assert_eq!(fish, "/h/j/p/r/m/s/lib.rs");
```

## Strategies

shrinkpath ships four strategies. Pick the one that fits your use case, or use
**Hybrid** (the default) and let the algorithm decide.

### Fish

Abbreviates every directory segment to its first character. Dot-prefixed
directories keep the dot: `.config` &rarr; `.c`.

```
/home/john/projects/rust/myapp/src/lib.rs  →  /h/j/p/r/m/s/lib.rs
C:\Users\Admin\AppData\Local\Temp\file.txt →  C:\U\A\A\L\T\file.txt
~/projects/rust/file.rs                    →  ~/p/r/file.rs
```

Fish produces the shortest possible result. Use it when every character counts
(prompts, status bars) and the user can infer the full path from context.

**Tuning knobs:**

```rust
use shrinkpath::{shrink, ShrinkOptions, Strategy};

// Keep 2 chars per segment instead of 1 (like Starship's fish_style_pwd_dir_length)
let opts = ShrinkOptions::new(50).strategy(Strategy::Fish).dir_length(2);
let result = shrink("/home/john/projects/rust/myapp/src/lib.rs", &opts);
assert_eq!(result, "/ho/jo/pr/ru/my/sr/lib.rs");

// Keep the last N directory segments unabbreviated
let opts = ShrinkOptions::new(50).strategy(Strategy::Fish).full_length_dirs(1);
let result = shrink("/home/john/projects/rust/myapp/src/lib.rs", &opts);
assert_eq!(result, "/h/j/p/r/m/src/lib.rs");
```

### Ellipsis

Replaces middle segments with `...`, keeping the identity head (username) and
the segments nearest the filename.

```
/home/john/projects/rust/myapp/src/lib.rs  →  /home/john/.../src/lib.rs
C:\Users\Admin\AppData\Local\Temp\file.txt →  C:\Users\Admin\...\file.txt
```

Ellipsis is the most readable strategy. Use it when you have moderate space and
want humans to immediately understand the path.

### Hybrid (default)

A graduated four-phase approach that produces the best result for any target
length:

```
Phase 1 — fish expendable segments only:    /home/john/p/r/m/src/lib.rs
Phase 2 — fish context segments too:        /h/john/p/r/m/s/lib.rs
Phase 3 — collapse abbreviated runs to ...: /h/john/.../s/lib.rs
Phase 4 — fish identity (last resort):      /h/j/.../s/lib.rs
```

Each phase stops as soon as the result fits. If nothing fits, it falls back to
`/.../<filename>`, then the filename alone.

### Unique

Disambiguates segments against each other within the same path. Each segment is
abbreviated to the shortest prefix that distinguishes it from every other segment.

```
/home/documents/downloads/file.txt  →  /h/doc/dow/file.txt
/Users/Admin/AppData/Application/f  →  /U/Ad/AppD/Appl/f
```

When all first characters are unique, Unique behaves like Fish. When segments
share prefixes, it uses the minimum characters needed. Identical segments are
kept in full (they can't be disambiguated).

```rust
use shrinkpath::shrink_unique;

let result = shrink_unique("/home/documents/downloads/file.txt");
assert_eq!(result, "/h/doc/dow/file.txt");
```

## Features

### Mapped Locations

Substitute known path prefixes before shortening. Useful for replacing home
directories, project roots, or well-known paths with short aliases.

```rust
use shrinkpath::{shrink, ShrinkOptions};

let opts = ShrinkOptions::new(50)
    .map_location("/home/john", "~")
    .map_location("/home/john/projects", "PROJ:");

// Longest match wins
let result = shrink("/home/john/projects/rust/lib.rs", &opts);
assert!(result.starts_with("PROJ:"));
```

### Anchor Segments

Mark directory names that should never be abbreviated, regardless of strategy.

```rust
use shrinkpath::{shrink, ShrinkOptions, Strategy};

let opts = ShrinkOptions::new(50)
    .strategy(Strategy::Fish)
    .anchor("src")
    .anchor("myapp");

let result = shrink("/home/john/projects/rust/myapp/src/lib.rs", &opts);
// "myapp" and "src" kept full, everything else abbreviated
assert!(result.contains("myapp"));
assert!(result.contains("/src/"));
```

### Segment Metadata

`shrink_detailed()` returns per-segment metadata for building colored prompts,
clickable breadcrumbs, or tooltip UIs.

```rust
use shrinkpath::{shrink_detailed, ShrinkOptions, Strategy};

let opts = ShrinkOptions::new(usize::MAX).strategy(Strategy::Fish);
let result = shrink_detailed("/home/john/projects/lib.rs", &opts);

for seg in &result.segments {
    if seg.was_abbreviated {
        // render abbreviated segments in dim color
        print!("{}", seg.shortened);
    } else if seg.is_filename {
        // render filename in bold
        print!("{}", seg.shortened);
    } else {
        print!("{}", seg.shortened);
    }
}
// seg.original always contains the full text for tooltips
```

### Filesystem-Aware Features (opt-in)

Enable the `fs` feature for features that require filesystem access:

```toml
[dependencies]
shrinkpath = { version = "0.1", features = ["fs"] }
```

**Git repo root detection** &mdash; find the repository name for a file path:

```rust
use shrinkpath::fs_aware::find_git_root;

if let Some(repo) = find_git_root("/home/john/projects/myapp/src/lib.rs") {
    println!("repo: {}", repo); // "myapp"
}
```

**Filesystem-aware disambiguation** &mdash; find the shortest unique prefix by
checking against actual sibling directories (like Powerlevel10k):

```rust
use shrinkpath::fs_aware::disambiguate_segment;
use std::path::Path;

// If /home contains "documents", "downloads", "desktop":
let short = disambiguate_segment(Path::new("/home"), "documents");
// Returns "doc" (shortest prefix unique among siblings)
```

## How It Works

### Path Parsing

Every input path is parsed into three parts:

```
  prefix       segments (directories)         filename
    │          │                                │
    ▼          ▼                                ▼
    /    home / john / projects / rust / src /  lib.rs
   ~~   ~~~~~~~~~~~~~~~~~~~~~~~~~~────────────  ~~~~~~
```

**Prefix** is the root: `/`, `~`, `C:\`, `\\server\share`, `.`, or empty.
**Filename** is the last component. **Segments** are everything in between.

The path style (Unix or Windows) is auto-detected from the input string &mdash;
drive letters, UNC prefixes, and backslash heuristics are all recognized. You
can also force a style with `ShrinkOptions::path_style()`.

### Segment Priority

Each segment is classified by how important it is to keep:

| Priority | What | Example | Dropped |
|---|---|---|---|
| **Sacred** | Filename | `lib.rs` | Never |
| **Identity** | Username / profile | `john`, `Admin` | Last |
| **Context** | Home root, well-known dirs | `home`, `Users` | Middle |
| **Expendable** | Everything else | `projects`, `src` | First |

Identity is detected by recognizing the segment after a home root (`home` on
Unix, `Users` on Windows). The tilde prefix (`~`) encodes identity implicitly.

### Fish Algorithm

```
for each directory segment (from left, skipping last full_length_dirs):
    if segment is anchored:
        keep full text
    else if segment starts with '.':
        keep '.' + first dir_length chars after dot    (.config → .c)
    else:
        keep first dir_length chars                    (projects → p)
filename is never touched
```

### Ellipsis Algorithm

```
1. Compute identity head = all segments up to and including the username
2. Base cost = prefix + separator + filename
3. Add head cost + ellipsis marker cost
4. With remaining budget, greedily add tail segments from right to left
5. If head doesn't fit, fall back to prefix + ... + filename
```

The greedy right-to-left fill keeps the segments closest to the filename, which
provide the most context about what the file actually is.

### Hybrid Algorithm

```
Phase 1: Fish all Expendable segments (respect anchors)
         → check if result fits target length
Phase 2: Fish all Context segments
         → check fit
Phase 3: Collapse consecutive abbreviated segments into ...
         Try keeping 0-3 head + 0-3 tail segments, pick best fit
         → check fit
Phase 4: Fish all Identity segments (last resort)
         → try collapse again
Fallback: prefix + ... + filename
Last resort: filename only
```

The key insight is that each phase is strictly less destructive than the next.
A 30-char budget on a moderately deep path might only need Phase 1. A 15-char
budget on a deeply nested Windows path might need all four phases. The user sees
the best possible result for their budget.

### Unique Algorithm

```
1. Collect all segment texts
2. For each segment, find minimum prefix length L such that:
   - segment[..L] differs from every other segment's first L chars
   - For dot-prefixed: compare the part after the dot
3. Identical segments keep their full text (can't disambiguate)
4. Anchored segments are never abbreviated
```

## CLI

```sh
cargo install shrinkpath
```

```sh
# Basic usage
shrinkpath "/home/john/projects/rust/myapp/src/lib.rs" -m 30

# Pipe paths from find, fd, rg, etc.
fd -t f | shrinkpath -m 40

# Fish strategy
shrinkpath -s fish "/home/john/projects/rust/myapp/src/lib.rs"

# Unique strategy
shrinkpath -s unique "/home/documents/downloads/file.txt"

# JSON output with metadata
shrinkpath --json "/home/john/projects/src/lib.rs" -m 25
# {"original":"/home/john/projects/src/lib.rs","shortened":"/home/john/.../lib.rs",...}

# Custom ellipsis marker
shrinkpath --ellipsis "~" -m 30 "C:\Users\Admin\AppData\Local\Temp\file.txt"
```

**Flags:**

| Flag | Default | Description |
|---|---|---|
| `-m`, `--max-len` | `40` | Target maximum output length |
| `-s`, `--strategy` | `hybrid` | `fish`, `ellipsis`, `hybrid`, or `unique` |
| `--style` | `auto` | Force `unix` or `windows` separator style |
| `--ellipsis` | `...` | Custom ellipsis marker string |
| `--json` | off | Output JSON with original, shortened, lengths, style |

## API

```rust
// One-liner convenience functions
shrinkpath::shrink_to(path, 30)        // Hybrid strategy, target length
shrinkpath::shrink_fish(path)          // Fish abbreviation, no length target
shrinkpath::shrink_ellipsis(path, 30)  // Ellipsis strategy, target length
shrinkpath::shrink_unique(path)        // Unique disambiguation, no length target

// Full control
use shrinkpath::{shrink, ShrinkOptions, Strategy, PathStyle};

let opts = ShrinkOptions::new(30)
    .strategy(Strategy::Ellipsis)
    .path_style(PathStyle::Windows)
    .ellipsis("..")
    .dir_length(2)           // chars per abbreviated segment
    .full_length_dirs(1)     // keep last N dirs unabbreviated
    .anchor("src")           // never abbreviate "src"
    .map_location("~", "/home/john");

let result = shrink(path, &opts);

// With segment metadata
let detailed = shrinkpath::shrink_detailed(path, &opts);
for seg in &detailed.segments {
    println!("{} → {} (abbreviated: {})",
        seg.original, seg.shortened, seg.was_abbreviated);
}
```

## Platform Support

| Path format | Example | Detected as |
|---|---|---|
| Unix absolute | `/home/john/file.rs` | Unix |
| Tilde home | `~/projects/file.rs` | Unix |
| macOS `/Users` | `/Users/john/Documents/file.txt` | Unix |
| Windows drive | `C:\Users\Admin\file.txt` | Windows |
| Windows UNC | `\\server\share\dept\file.xlsx` | Windows |
| Dot-relative | `.\src\main.rs` | Windows |
| Forward-slash drive | `C:/Users/Admin/file.txt` | Windows |
| Relative (no prefix) | `src/lib.rs` | Unix |
| Backslash heuristic | `Users\Admin\file.txt` | Windows |

Detection is automatic. Use `ShrinkOptions::path_style()` to override.

## Cargo Features

| Feature | Default | Description |
|---|---|---|
| `cli` | Yes | Builds the `shrinkpath` binary (pulls in `clap`) |
| `fs` | No | Enables filesystem-aware features (`find_git_root`, `disambiguate_segment`) |

```toml
# Library only, zero dependencies
shrinkpath = { version = "0.1", default-features = false }

# With filesystem features
shrinkpath = { version = "0.1", features = ["fs"] }
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT License](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

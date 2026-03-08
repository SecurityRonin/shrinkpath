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
- **No filesystem access** &mdash; works on path strings alone, so it runs in WASM, embedded, or anywhere

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

shrinkpath ships three strategies. Pick the one that fits your use case, or use
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
for each directory segment:
    if segment starts with '.':
        keep '.' + first char after dot    (.config → .c)
    else:
        keep first char only               (projects → p)
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
Phase 1: Fish all Expendable segments
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
| `-s`, `--strategy` | `hybrid` | `fish`, `ellipsis`, or `hybrid` |
| `--style` | `auto` | Force `unix` or `windows` separator style |
| `--ellipsis` | `...` | Custom ellipsis marker string |
| `--json` | off | Output JSON with original, shortened, lengths, style |

## API

```rust
// One-liner convenience functions
shrinkpath::shrink_to(path, 30)        // Hybrid strategy, target length
shrinkpath::shrink_fish(path)          // Fish abbreviation, no length target
shrinkpath::shrink_ellipsis(path, 30)  // Ellipsis strategy, target length

// Full control
use shrinkpath::{shrink, ShrinkOptions, Strategy, PathStyle};

let opts = ShrinkOptions::new(30)
    .strategy(Strategy::Ellipsis)
    .path_style(PathStyle::Windows)
    .ellipsis("..");

let result = shrink(path, &opts);

// With metadata
let detailed = shrinkpath::shrink_detailed(path, &opts);
println!("truncated: {}", detailed.was_truncated);
println!("style: {:?}", detailed.detected_style);
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

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT License](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

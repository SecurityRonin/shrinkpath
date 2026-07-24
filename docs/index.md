# shrinkpath

**Smart cross-platform path shortening for Rust and JavaScript/TypeScript.**

```
/Users/john/Library/Application Support/Code/User/settings.json  (62 chars)
                            ↓  shrink_to(path, 35)
             /Users/john/.../User/settings.json                   (35 chars)
```

The filename is never truncated. The username is preserved when possible. Everything else adapts to fit your target length.

**[GitHub Repository →](https://github.com/SecurityRonin/shrinkpath)**

---

## What it does

shrinkpath shortens a path string to fit a target length, working on the string alone — no filesystem access, so it runs in WASM, embedded, or anywhere. It handles `/home/...`, `~/...`, `C:\Users\...`, `\\server\share\...`, and `.\...` from any host OS, and it always returns a result at or under your target length (unless the filename itself is longer — filenames are sacred).

```rust
use shrinkpath::{shrink_to, shrink_fish};

let short = shrink_to("/home/john/projects/rust/myapp/src/lib.rs", 30);
assert!(short.len() <= 30);
assert!(short.ends_with("lib.rs"));

let fish = shrink_fish("/home/john/projects/rust/myapp/src/lib.rs");
assert_eq!(fish, "/h/j/p/r/m/s/lib.rs");
```

---

## Packages

| Platform | Package | Install |
|---|---|---|
| Rust | [crates.io/crates/shrinkpath](https://crates.io/crates/shrinkpath) | `cargo add shrinkpath` |
| JavaScript/TypeScript | [@4n6h4x0r/shrinkpath](https://www.npmjs.com/package/@4n6h4x0r/shrinkpath) | `npm install @4n6h4x0r/shrinkpath` |

Both packages have full feature parity. The npm package is a pure TypeScript implementation (not WASM) with zero dependencies.

---

## Strategies

shrinkpath ships four strategies. Pick the one that fits, or use **Hybrid** (the default) and let the algorithm decide.

- **Fish** — abbreviate every directory segment to its first character (`/home/john/projects/lib.rs` → `/h/j/p/lib.rs`). The shortest possible result.
- **Ellipsis** — replace middle segments with `...`, keeping the identity head and the segments nearest the filename (`/home/john/projects/rust/src/lib.rs` → `/home/john/.../src/lib.rs`). The most readable.
- **Hybrid (default)** — a graduated four-phase approach that produces the best result for any target length, each phase strictly less destructive than the next.
- **Unique** — abbreviate each segment to the shortest prefix that distinguishes it from every other segment in the path.

Segments are classified by how important they are to keep: the filename is sacred, the username/profile is identity (dropped last), home roots are context, and everything else is expendable (dropped first).

---

## Highlights

- **Target-length guarantee** — `shrink_to(path, 30)` always returns ≤ 30 chars (unless the filename is longer).
- **Identity-preserving** — the username segment is the last thing to go, not the first.
- **Cross-platform** — Unix, tilde, Windows drive, UNC, and dot-relative paths, auto-detected from the string.
- **Zero dependencies** — the library has none; `clap` is only pulled in for the optional CLI.
- **Segment metadata** — `shrink_detailed()` returns per-segment data for colored prompts, breadcrumbs, or tooltips.
- **Optional `fs` feature** — git-repo-root detection and filesystem-aware disambiguation when you opt in.

See the [README](https://github.com/SecurityRonin/shrinkpath#readme) for the full API, CLI flags, and algorithm details.

---

[Privacy Policy](privacy.md) · [Terms of Service](terms.md) · [GitHub](https://github.com/SecurityRonin/shrinkpath) · © 2026 Security Ronin Ltd.

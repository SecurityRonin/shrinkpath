# 5. Filesystem access is opt-in behind the `fs` feature

Date: 2026-07-24
Status: Accepted

## Context

Two shortening behaviors genuinely need to touch the disk: finding a file's git
repository root (walk up looking for `.git`), and Powerlevel10k-style
disambiguation that abbreviates a directory to the shortest prefix unique among
its *actual* filesystem siblings. Both are valuable, but both break the library's
core promise — that it works on path strings alone and runs in WASM, browser, and
embedded contexts where there is no filesystem, and that it never performs I/O a
caller did not ask for.

## Decision

Gate all filesystem-touching functionality behind a non-default `fs` Cargo
feature.

- The `fs` feature (`[features] fs = []`) enables the `fs_aware` module, compiled
  only under `#[cfg(feature = "fs")]`.
- `fs_aware` holds exactly the disk-touching functions: `find_git_root` and
  `disambiguate_segment`.
- The **default build performs no I/O.** The string-only strategies (Fish,
  Ellipsis, Hybrid, Unique) never read the disk.
- The filesystem helpers **degrade gracefully**: `find_git_root` returns `None`
  when no `.git` ancestor exists or the path is absent; `disambiguate_segment`
  returns the full segment when the directory cannot be read.

## Consequences

- The zero-config, zero-knowledge path is the safe one — a caller gets pure,
  side-effect-free shortening unless it deliberately opts into `fs`. This is
  secure-by-default applied to I/O surface.
- The library keeps compiling for WASM/browser/embedded targets, because the code
  that needs `std::fs` and `std::path` only exists when `fs` is on.
- The npm port mirrors this exactly: the filesystem helpers live on a separate
  `shrinkpath/fs` subpath export that is Node-only, so the browser bundle never
  pulls them in (see ADR 0008).
- Grounding: `Cargo.toml` (`[features] fs = []`); `src/lib.rs`
  (`#[cfg(feature = "fs")] pub mod fs_aware;`); `src/fs_aware.rs` (`find_git_root`,
  `disambiguate_segment`, both returning safe fallbacks on failure); README
  "Filesystem-Aware Features (opt-in)".

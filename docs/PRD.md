# shrinkpath — Purpose & Scope

> Library-tier intent doc (Purpose & Scope, not a product PRD). shrinkpath is a
> general-purpose utility library that other code *links*, not a forensic tool an
> examiner *runs*. Per the fleet PRD & ADR Standard, a linked library gets this
> lighter `docs/PRD.md`; the load-bearing decisions live in `docs/decisions/`.

## What it is

shrinkpath is a zero-dependency, string-only library that shortens file paths to
fit a target display width while preserving the information a reader needs most:
the **filename** (never truncated) and the **user identity** segment (the last
thing to go). It ships in two forms with full feature parity — a Rust crate
(`shrinkpath` on crates.io) and a pure-TypeScript npm package
(`@4n6h4x0r/shrinkpath`) — plus a thin optional CLI wrapper.

It operates on path *strings* alone. It reads no filesystem, makes no OS calls,
and holds no dependencies in its default library build, so it runs unchanged in
WASM, embedded, browser, and forensic-workstation contexts.

## Who links it

- **Fleet CLI / TUI / GUI front-ends** that render file paths in constrained
  space — prompts, status bars, table columns, breadcrumb widgets. Any tool that
  displays a long evidence path (`disk4n6`, `br4n6`, `ev4n6`, the `-tui`/`-gui`
  surfaces) can render it compactly without hand-rolling truncation.
- **External Rust and JavaScript/TypeScript developers** building shell prompts,
  editors, log viewers, or any UI that shows paths — the same audience served by
  fish-shell abbreviation, Starship, and Powerlevel10k.

It is a **cross-cutting display utility leaf**, in the spirit of `jsonguard`
(output sanitization): not a forensic format reader, not on any analyzer's
dependency-inversion path, linked *down* by whatever front-end needs it.

## What it does

- **Target-length shortening** — `shrink_to(path, n)` returns a string of at most
  `n` characters (unless the filename alone is longer; filenames are sacred).
- **Four strategies** — Fish (every dir → first char), Ellipsis (`...` for the
  middle), Hybrid (graduated four-phase, the default), and Unique (shortest
  prefix that disambiguates siblings).
- **Cross-platform** — auto-detects Unix, Windows drive (`C:\`), UNC
  (`\\server\share`), tilde (`~`), and dot-relative paths from the input string,
  independent of the host OS; the style can also be forced.
- **Tuning** — chars-per-segment, trailing full-length dirs, anchor segments
  (never abbreviate), mapped-location prefix substitution, custom ellipsis marker.
- **Segment metadata** — `shrink_detailed()` returns per-segment original/shortened
  text and classification for building colored prompts or clickable breadcrumbs.
- **Optional filesystem awareness** (`fs` feature) — git-root detection and
  sibling-aware disambiguation, the only path that touches disk.

## Scope

- Path-string transformation for **display**. The output is for human eyes; it is
  not a canonical or round-trippable path.
- Deterministic and pure in the default build — same input, same output, no I/O.
- Rust and TypeScript kept at feature parity from one design.

## Non-goals

- **Not a path manipulation library** — no join/normalize/canonicalize/resolve.
  Use `std::path` / `node:path` for those.
- **Not filesystem-truth by default** — the string-only strategies never stat the
  disk; only the opt-in `fs` feature does, and it degrades gracefully when a path
  is absent.
- **Not reversible** — an abbreviated path cannot be expanded back to the original;
  it is a lossy display rendering.
- **Not a forensic analyzer** — it emits no findings, no `forensicnomicon::report`
  types, and takes no part in the evidence chain. It is a formatting helper.

## Correctness approach

The load-bearing invariants are property-shaped and unit-tested in both
implementations: the filename is never truncated; `shrink_to(path, n)` yields
`len ≤ n` whenever the filename fits; each Hybrid phase is strictly less
destructive than the next; path-style detection is stable across host OSes; and
the Rust and TypeScript ports agree on the same fixtures (parity). The `fs`
feature's disambiguation is validated against real sibling directories on disk.

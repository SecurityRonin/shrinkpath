# 3. Four pluggable strategies with a graduated Hybrid default

Date: 2026-07-24
Status: Accepted

## Context

There is no single "right" way to shorten a path — the best rendering depends on
how much space is available and how much the reader can infer from context. A
shell prompt with 20 characters wants aggressive fish-style abbreviation; a
tooltip with 60 wants readable ellipsis; a table column wants "as short as needed
and no shorter." One fixed algorithm serves none of them well.

## Decision

Expose the shortening algorithm as a `Strategy` enum with four members, selected
per call, sharing the segment-priority substrate from ADR 0002:

- **Fish** — abbreviate every directory segment to its first character
  (`.config` keeps the dot → `.c`); shortest possible, no length target.
- **Ellipsis** — replace the middle with `...`, keeping the identity head and a
  greedily-filled tail nearest the filename; most readable.
- **Hybrid** *(default)* — a graduated four-phase pass, each phase strictly less
  destructive than the next, stopping as soon as the result fits the target:
  1. fish the Expendable segments,
  2. fish the Context segments,
  3. collapse consecutive abbreviated runs into `...`,
  4. fish the Identity segments (last resort);
  falling back to `prefix + ... + filename`, then the filename alone.
- **Unique** — abbreviate each segment to the shortest prefix that distinguishes
  it from the other segments in the same path.

`ShrinkOptions::new(max_len)` defaults to `Strategy::Hybrid` so the
zero-knowledge caller gets the best target-length behavior automatically.

## Consequences

- The four strategies were built and covered independently (git history:
  `feat(ts): add fish/ellipsis/unique/hybrid strategy` commits), and each lives in
  its own `src/strategy/*.rs` module behind a common entry point.
- Hybrid is the safe default: a caller who reads nothing and calls
  `shrink_to(path, n)` gets a result of `len ≤ n` (when the filename fits) chosen
  by the least-destructive phase that satisfies the budget — the secure-by-default
  UX principle applied to display.
- Adding a fifth strategy is additive: a new `Strategy` variant plus a module,
  with no change to parsing or classification. (`Strategy` is a small public enum;
  consumers matching it should use a `_` arm to stay forward-compatible.)
- Grounding: `src/strategy/mod.rs` (`Strategy` enum), `src/strategy/hybrid.rs`
  (four-phase doc + `shrink_hybrid`), `src/lib.rs` (`ShrinkOptions::new` defaults
  to Hybrid); README "Strategies".

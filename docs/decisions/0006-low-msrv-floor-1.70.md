# 6. Low MSRV floor (1.70), CI-verified on the library alone

Date: 2026-07-24
Status: Accepted

## Context

shrinkpath is a *published library* meant for broad reuse — inside the fleet and
by external Rust and JavaScript developers. The fleet MSRV policy separates the
**dev toolchain** (pinned to current stable) from the **declared MSRV** (a
downstream-facing compatibility promise). For a library, a low, CI-verified MSRV
is a deliberate feature and a trust signal: raising it narrows the crates.io
audience and should be treated as near-breaking.

The one wrinkle: the optional `clap` CLI dependency (ADR 0001) tracks a newer
compiler than the library needs, so it must not set the floor.

## Decision

1. Declare a **low MSRV of `1.70`** in `Cargo.toml` (`rust-version = "1.70"`),
   well below the pinned dev toolchain.
2. Pin the dev toolchain to current stable in `rust-toolchain.toml`
   (`channel = "1.96.0"`, with `rustfmt` + `clippy` components) — develop on the
   newest stable, promise only what the library needs.
3. **Verify the MSRV against the library only**, not the CLI: the CI matrix runs
   `stable` and `"1.70"`, and the 1.70 job runs `cargo test --no-default-features
   --lib` so `clap` (which needs a newer rustc) is excluded from the floor check.

## Consequences

- The 1.70 CI job is a real, standing guarantee that the library compiles on an
  older compiler — it is not discarded just because the dev toolchain moved to
  1.96.
- The CLI's compiler requirement is decoupled from the library's promise: a
  consumer linking `shrinkpath` with `default-features = false` builds on 1.70,
  while `cargo install shrinkpath` (which builds `clap`) needs whatever `clap`
  needs.
- Grounding: `Cargo.toml` (`rust-version = "1.70"`); `rust-toolchain.toml`
  (`channel = "1.96.0"`); `.github/workflows/ci.yml` (`rust: [stable, "1.70"]`,
  "MSRV job: test lib only (no clap, which requires newer rustc)",
  `cargo test --no-default-features --lib`); git commits ac98ddf
  ("use --no-default-features for MSRV 1.70 test matrix"), cb78b20
  ("pin rust-toolchain 1.96.0"); ronin-issen CLAUDE.md "Rust MSRV & Toolchain
  Policy" (published libraries keep a low CI-verified MSRV).

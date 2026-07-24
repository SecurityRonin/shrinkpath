# 1. String-only, zero-dependency library with an optional CLI

Date: 2026-07-24
Status: Accepted

## Context

shrinkpath exists to render long file paths in constrained display space —
prompts, status bars, table columns. Its callers span forensic front-ends, shell
prompts, editors, and log viewers, and it must run in environments with no
filesystem and no allocator assumptions: WASM, browser, embedded, and forensic
workstations.

Two forces shaped the packaging. First, a display helper linked by many tools
must not drag a dependency tree behind it — every transitive crate is supply-chain
surface a consumer inherits. Second, a command-line front-end (`fd -t f |
shrinkpath -m 40`) is genuinely useful, but an argument parser (`clap`) pulls in
dependencies and raises the minimum compiler the library will build on.

## Decision

1. The **library core operates on path strings only** — no filesystem access, no
   OS calls, no I/O in the default build. `shrink`, `shrink_to`, `shrink_fish`,
   `shrink_ellipsis`, `shrink_unique`, and `shrink_detailed` are pure functions of
   `(&str, options)`.
2. The **library has zero dependencies.** `Cargo.toml` declares only one dependency
   — `clap` — and it is `optional = true`.
3. The **CLI is gated behind a `cli` feature** (`cli = ["clap"]`), and the
   `shrinkpath` binary declares `required-features = ["cli"]`. `cli` is a default
   feature for the convenience of `cargo install shrinkpath`, but
   `default-features = false` gives a pure, dep-free library.
4. The release profile targets a small static binary: `strip = true`,
   `lto = true`, `codegen-units = 1`, `panic = "abort"`.

## Consequences

- A consumer that only needs the library gets zero transitive dependencies and a
  build that compiles anywhere, including `no_std`-adjacent targets that provide
  `alloc` + `String`.
- The CLI's `clap` requirement is isolated: it cannot raise the *library's*
  effective minimum compiler, which lets the MSRV job build the library alone
  (see ADR 0006).
- `panic = "abort"` means the CLI binary aborts rather than unwinding; the library
  contract is that its public functions do not panic on any input string.
- Grounding: `Cargo.toml` (`clap … optional = true`, `[features] cli = ["clap"]`,
  `[[bin]] required-features = ["cli"]`, `[profile.release]`); README "Zero
  dependencies" and "No filesystem access".

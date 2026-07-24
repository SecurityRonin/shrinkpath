# 7. Dual MIT OR Apache-2.0 license

Date: 2026-07-24
Status: Accepted

## Context

The SecurityRonin forensic fleet standardized on **Apache-2.0** for its explicit
patent grant, and the fleet README standard calls for migrating residual MIT
repos. shrinkpath, however, is not a forensic analyzer — it is a general-purpose
path-display utility positioned for the broad Rust and JavaScript ecosystems
(dual-published to crates.io and npm, README targeting shell-prompt / editor /
CLI developers generally). The Rust ecosystem's de-facto convention for
broadly-reusable libraries is the dual `MIT OR Apache-2.0` license.

## Decision

License the crate and the npm package under **`MIT OR Apache-2.0`** (SPDX OR:
consumers pick either), shipping both `LICENSE-APACHE` and `LICENSE-MIT` in the
Rust repo and in the `ts/` package.

- `Cargo.toml`: `license = "MIT OR Apache-2.0"`.
- `ts/package.json`: `"license": "MIT OR Apache-2.0"`, both license files in
  `files`.
- No `## License` prose beyond the standard dual-license contribution note; the
  license files are the source of truth.

## Consequences

- A consumer gets the Apache-2.0 patent grant *and* MIT's brevity, the widest
  acceptance across both ecosystems — appropriate for a leaf utility intended for
  maximum external reuse rather than a fleet-internal forensic component.
- This deliberately diverges from the forensic-repo Apache-only norm; the divergence
  is defensible precisely because shrinkpath is a general-purpose library, not an
  evidence-handling analyzer.
- **Rationale reconstructed from structure; original intent not recovered in
  available history.** The dual-license *choice* is visible in the code (both
  license files, the SPDX expression, the dual-registry positioning), but the git
  history does not record whether it was a deliberate ecosystem decision or a
  not-yet-migrated MIT residual. Documented here as the observed state; if the
  fleet later mandates Apache-only for this repo, that would be a separate,
  recorded decision.
- Grounding: `Cargo.toml` (`license = "MIT OR Apache-2.0"`); `LICENSE-APACHE`,
  `LICENSE-MIT`; `ts/package.json`; git commit 128ec63 ("prep npm publish — scoped
  name, dual license, docs"); README "License" section.

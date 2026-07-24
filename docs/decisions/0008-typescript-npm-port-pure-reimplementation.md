# 8. TypeScript/npm port as a pure reimplementation at feature parity

Date: 2026-07-24
Status: Accepted

## Context

The largest single body of shrinkpath's JavaScript audience — shell prompts,
editors, log viewers, web UIs — lives in Node and the browser, not Rust. Shipping
the algorithm to them had two paths: compile the Rust crate to WASM
(`wasm-pack`), or reimplement it in TypeScript. WASM would guarantee one source of
truth but imposes a `.wasm` binary, a load/instantiate step, awkward
string-marshalling across the boundary, and a bundle that browsers and
tree-shakers handle poorly — a heavy cost for what is a few hundred lines of pure
string logic with zero runtime dependencies.

## Decision

Port shrinkpath to a **pure-TypeScript reimplementation** (not WASM), published as
`@4n6h4x0r/shrinkpath` on npm, living in a `ts/` subdirectory of the same
monorepo, at **full feature parity** with the Rust crate v0.1.1.

- Pure TypeScript, **zero runtime dependencies**, ESM-first with a CJS build,
  `sideEffects: false` and per-module structure so bundlers tree-shake.
- Parity across all four strategies, path-style detection, mapped locations,
  anchors, `dirLength` / `fullLengthDirs`, segment metadata, and custom ellipsis.
- The filesystem-aware helpers are exposed on a **separate `shrinkpath/fs` subpath
  export** that is Node-only, mirroring the Rust `fs` feature gate (ADR 0005) so
  the browser bundle never pulls in `node:fs`.
- Node engine floor `>=18`.

## Consequences

- JavaScript/TypeScript consumers get a native, dependency-free, tree-shakeable
  package with no WASM loading overhead or binary artifact — the same
  zero-dependency promise the Rust library makes.
- The cost is **two implementations to keep in lockstep.** Parity is maintained by
  porting from one design spec and mirroring the Rust module structure
  one-to-one, so a change in the algorithm is a mechanical two-place edit; the
  same fixtures exercise both.
- The port was built module-by-module against a written design spec (git history:
  platform → path-info → fish → ellipsis → unique → hybrid → public API →
  fs-aware), then covered to 100% including removal of a dead post-phase-4 collapse
  branch — evidence the reimplementation was validated, not transliterated blind.
- Grounding: `ts/package.json` (name `@4n6h4x0r/shrinkpath`, zero runtime deps,
  `./fs` subpath export, `engines.node >=18`);
  `docs/superpowers/specs/2026-03-19-npm-typescript-port-design.md` ("pure
  TypeScript npm package with full feature parity … Zero runtime deps"); git
  commits 7118bb6 → 4ab83a6 (the port series); README "pure TypeScript
  implementation (not WASM) with zero dependencies".

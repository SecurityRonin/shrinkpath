# shrinkpath npm (TypeScript) Port — Design Spec

## Overview

Port the Rust `shrinkpath` crate to a pure TypeScript npm package with full feature parity. The package lives in `ts/` within the existing monorepo.

**Package name:** `shrinkpath`
**Target:** Node 18+, modern browsers (core), Node-only (`shrinkpath/fs`)
**Dependencies:** Zero runtime deps

## Scope

Full parity with the Rust crate v0.1.1:

- 4 strategies: Hybrid, Fish, Ellipsis, Unique
- Cross-platform path detection (Unix, Windows drive, UNC, dot-backslash, tilde)
- Mapped locations (prefix substitution)
- Anchor segments (never-abbreviate list)
- `dirLength`, `fullLengthDirs` options
- Segment metadata (`ShrinkResult` with per-segment `SegmentInfo`)
- Custom ellipsis string
- Filesystem-aware features: `disambiguateSegment`, `findGitRoot` (Node-only subpath export)

## Project Setup & Tooling

**Location:** `ts/` directory in the existing repo

**Tooling:**

- TypeScript 5.x — strict mode, ESM-only source
- Vitest — test runner
- tsup — bundler (ESM + CJS dual output)
- Zero runtime dependencies

**Package exports map:**

```json
{
  "name": "shrinkpath",
  "type": "module",
  "exports": {
    ".": {
      "import": "./dist/index.mjs",
      "require": "./dist/index.cjs",
      "types": "./dist/index.d.ts"
    },
    "./fs": {
      "import": "./dist/fs-aware.mjs",
      "require": "./dist/fs-aware.cjs",
      "types": "./dist/fs-aware.d.ts"
    }
  }
}
```

- `import { shrinkTo } from 'shrinkpath'` — pure string operations, works everywhere
- `import { disambiguateSegment } from 'shrinkpath/fs'` — Node-only, uses `node:fs`

## Types & Public API

### Types

```ts
type Strategy = 'hybrid' | 'fish' | 'ellipsis' | 'unique';
type PathStyle = 'unix' | 'windows';
type SegmentPriority = 'sacred' | 'identity' | 'context' | 'expendable';

interface ShrinkOptions {
  maxLen: number;
  strategy?: Strategy;                   // default: 'hybrid'
  pathStyle?: PathStyle;                 // default: auto-detect
  ellipsis?: string;                     // default: '...'
  dirLength?: number;                    // default: 1
  fullLengthDirs?: number;               // default: 0
  mappedLocations?: [string, string][];  // [from, to] pairs
  anchors?: string[];                    // segments to never abbreviate
}

interface SegmentInfo {
  original: string;
  shortened: string;
  wasAbbreviated: boolean;
  isFilename: boolean;
}

interface ShrinkResult {
  shortened: string;
  originalLen: number;
  shortenedLen: number;
  wasTruncated: boolean;
  detectedStyle: PathStyle;
  segments: SegmentInfo[];
}
```

### Public API

```ts
// Core
function shrink(path: string, opts: ShrinkOptions): string;
function shrinkDetailed(path: string, opts: ShrinkOptions): ShrinkResult;

// Convenience
function shrinkTo(path: string, maxLen: number): string;
function shrinkFish(path: string): string;
function shrinkEllipsis(path: string, maxLen: number): string;
function shrinkUnique(path: string): string;

// shrinkpath/fs (Node-only)
function disambiguateSegment(parentPath: string, segment: string): string;
function findGitRoot(path: string): string | null;
```

## Internal Architecture

### Module breakdown (1:1 with Rust)

**`platform.ts`** — Path style detection and constants

- `detectStyle(path): PathStyle` — UNC/drive/dot-backslash/backslash-only heuristics
- `separator(style): string`
- `UNIX_HOME_ROOTS`, `WIN_HOME_ROOTS`

**`path-info.ts`** — Path parsing and segment classification

- `PathInfo` class: `prefix`, `segments`, `filename`, `style`
- `PathInfo.parse(path, forceStyle?)` — static factory
- `PathInfo.reassemble(segmentTexts)` — reconstructs path
- Priority classification: after home root = identity, home root itself = context, else expendable

**`strategy/fish.ts`**

- `abbreviateSegment(text, len, anchors)` — dot-prefix aware
- `shrinkFishStrategy(info, dirLength, fullLengthDirs, anchors)`

**`strategy/ellipsis.ts`**

- `shrinkEllipsisStrategy(info, maxLen, ellipsis)` — budget-aware, identity-preserving, greedy right-to-left tail fill

**`strategy/hybrid.ts`**

- `shrinkHybridStrategy(info, maxLen, ellipsis, anchors)` — 4 phases:
  1. Fish expendable segments
  2. Fish context segments
  3. Collapse middle into ellipsis
  4. Fish identity segments (last resort)
- `collapseMiddle()` helper

**`strategy/unique.ts`**

- `uniquePrefixLen(target, others)` — shortest disambiguating prefix
- `shrinkUniqueStrategy(info, anchors)`

**`fs-aware.ts`** — Separate entry point, Node-only

- `findGitRoot(path)` — walks up for `.git`
- `disambiguateSegment(parentPath, segment)` — `fs.readdirSync` sibling comparison

**`index.ts`** — Public API

- Re-exports types
- `shrink()`, `shrinkDetailed()`, convenience functions
- `applyMappedLocations()`, `buildSegmentMetadata()`

### Key behaviors preserved from Rust

- Filename is sacred (never truncated)
- Identity is last to drop in Hybrid
- Dot-prefixed segments keep dot + N chars (`.config` -> `.c`)
- UNC prefix `\\server\share` stays as a unit
- Anchored segments skip abbreviation in all strategies
- `fullLengthDirs` protects N trailing dir segments
- Ellipsis uses greedy right-to-left fill
- Unique: identical segments kept in full

## Testing Strategy

**Framework:** Vitest

**Structure:**

```
ts/tests/
├── platform.test.ts
├── path-info.test.ts
├── strategy/
│   ├── fish.test.ts
│   ├── ellipsis.test.ts
│   ├── hybrid.test.ts
│   └── unique.test.ts
├── fs-aware.test.ts
├── index.test.ts
└── integration.test.ts
```

**Approach:** Every `#[test]` in the Rust crate becomes a Vitest `test()` with the same name, inputs, and expected outputs. Additional edge cases for JS-specific string handling.

**TDD flow:** Write tests first (red), implement until green, refactor.

**Module build order** (dependency-driven):

1. `platform.ts`
2. `path-info.ts`
3. `strategy/fish.ts`
4. `strategy/ellipsis.ts`
5. `strategy/unique.ts`
6. `strategy/hybrid.ts`
7. `index.ts`
8. `fs-aware.ts`

## Build, Publish & CI

**Build:** tsup — dual ESM/CJS, two entry points, target `node18` + `es2022`

**Scripts:**

```json
{
  "build": "tsup",
  "test": "vitest run",
  "test:watch": "vitest",
  "lint": "tsc --noEmit",
  "prepublishOnly": "npm run lint && npm run test && npm run build"
}
```

**CI:** GitHub Actions job, Node 18/20/22 matrix, runs alongside Rust CI.

**Publish:** Manual `npm publish` from `ts/`.

**npm tarball:** `dist/`, `README.md`, `LICENSE` only.

# shrinkpath

**Smart cross-platform path shortening for JavaScript and TypeScript.**

[![npm](https://img.shields.io/npm/v/@4n6h4x0r/shrinkpath.svg)](https://www.npmjs.com/package/@4n6h4x0r/shrinkpath)
[![CI](https://github.com/SecurityRonin/shrinkpath/actions/workflows/ci.yml/badge.svg)](https://github.com/SecurityRonin/shrinkpath/actions)
[![License: MIT OR Apache-2.0](https://img.shields.io/npm/l/@4n6h4x0r/shrinkpath.svg)](#license)

```
/Users/john/Library/Application Support/Code/User/settings.json  (62 chars)
                            ↓  shrinkTo(path, 35)
             /Users/john/.../User/settings.json                   (35 chars)
```

Filename is never truncated. Username is preserved when possible. Everything else
adapts to fit your target length.

> TypeScript port of the [shrinkpath Rust crate](https://crates.io/crates/shrinkpath). Full feature parity, zero dependencies.

## Why shrinkpath?

- **Target-length guarantee** &mdash; `shrinkTo(path, 30)` always returns &le; 30 chars (unless the filename itself is longer &mdash; filenames are sacred)
- **Identity-preserving** &mdash; the username/profile segment is the last thing to go, not the first
- **Cross-platform** &mdash; handles `/home/...`, `~/...`, `C:\Users\...`, `\\server\share\...`, `.\...` from any host OS
- **Zero dependencies** &mdash; pure TypeScript, nothing to install beyond this package
- **No filesystem access** &mdash; works on path strings alone (opt-in `@4n6h4x0r/shrinkpath/fs` subpath for filesystem-aware features)
- **Dual ESM/CJS** &mdash; works in Node.js, Bun, bundlers, and anywhere JavaScript runs

## Install

```sh
npm install @4n6h4x0r/shrinkpath
```

## Quick Start

```typescript
import { shrinkTo, shrinkFish } from '@4n6h4x0r/shrinkpath';

// Hybrid strategy: graduated shortening to fit a target length
const short = shrinkTo('/home/john/projects/rust/myapp/src/lib.rs', 30);
// => '/home/john/.../src/lib.rs' (≤ 30 chars, ends with lib.rs)

// Fish strategy: every directory becomes its first character
const fish = shrinkFish('/home/john/projects/rust/myapp/src/lib.rs');
// => '/h/j/p/r/m/s/lib.rs'
```

## Strategies

shrinkpath ships four strategies. Pick the one that fits your use case, or use
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

**Tuning knobs:**

```typescript
import { shrink } from '@4n6h4x0r/shrinkpath';

// Keep 2 chars per segment instead of 1
shrink('/home/john/projects/rust/myapp/src/lib.rs', {
  maxLen: 50,
  strategy: 'fish',
  dirLength: 2,
});
// => '/ho/jo/pr/ru/my/sr/lib.rs'

// Keep the last N directory segments unabbreviated
shrink('/home/john/projects/rust/myapp/src/lib.rs', {
  maxLen: 50,
  strategy: 'fish',
  fullLengthDirs: 1,
});
// => '/h/j/p/r/m/src/lib.rs'
```

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

### Unique

Disambiguates segments against each other within the same path. Each segment is
abbreviated to the shortest prefix that distinguishes it from every other segment.

```
/home/documents/downloads/file.txt  →  /h/doc/dow/file.txt
/Users/Admin/AppData/Application/f  →  /U/Ad/AppD/Appl/f
```

When all first characters are unique, Unique behaves like Fish. When segments
share prefixes, it uses the minimum characters needed. Identical segments are
kept in full (they can't be disambiguated).

```typescript
import { shrinkUnique } from '@4n6h4x0r/shrinkpath';

shrinkUnique('/home/documents/downloads/file.txt');
// => '/h/doc/dow/file.txt'
```

## Features

### Mapped Locations

Substitute known path prefixes before shortening. Useful for replacing home
directories, project roots, or well-known paths with short aliases.

```typescript
import { shrink } from '@4n6h4x0r/shrinkpath';

const result = shrink('/home/john/projects/rust/lib.rs', {
  maxLen: 50,
  mappedLocations: [
    ['/home/john', '~'],
    ['/home/john/projects', 'PROJ:'],
  ],
});
// Longest match wins → 'PROJ:/rust/lib.rs'
```

### Anchor Segments

Mark directory names that should never be abbreviated, regardless of strategy.

```typescript
import { shrink } from '@4n6h4x0r/shrinkpath';

const result = shrink('/home/john/projects/rust/myapp/src/lib.rs', {
  maxLen: 50,
  strategy: 'fish',
  anchors: ['src', 'myapp'],
});
// 'myapp' and 'src' kept full, everything else abbreviated
// => '/h/j/p/r/myapp/src/lib.rs'
```

### Segment Metadata

`shrinkDetailed()` returns per-segment metadata for building colored prompts,
clickable breadcrumbs, or tooltip UIs.

```typescript
import { shrinkDetailed } from '@4n6h4x0r/shrinkpath';

const result = shrinkDetailed('/home/john/projects/lib.rs', {
  maxLen: Infinity,
  strategy: 'fish',
});

for (const seg of result.segments) {
  if (seg.wasAbbreviated) {
    // render abbreviated segments in dim color
    process.stdout.write(`\x1b[2m${seg.shortened}\x1b[0m/`);
  } else if (seg.isFilename) {
    // render filename in bold
    process.stdout.write(`\x1b[1m${seg.shortened}\x1b[0m`);
  } else {
    process.stdout.write(`${seg.shortened}/`);
  }
  // seg.original always contains the full text for tooltips
}
```

### Filesystem-Aware Features

Import from `@4n6h4x0r/shrinkpath/fs` for features that require filesystem access (Node.js only):

```typescript
import { findGitRoot, disambiguateSegment } from '@4n6h4x0r/shrinkpath/fs';

// Find the git repository name for a file path
const repo = findGitRoot('/home/john/projects/myapp/src/lib.rs');
// => 'myapp'

// Find shortest unique prefix by checking sibling directories
// If /home contains "documents", "downloads", "desktop":
const short = disambiguateSegment('/home', 'documents');
// => 'doc'
```

## API

### Convenience Functions

```typescript
import { shrinkTo, shrinkFish, shrinkEllipsis, shrinkUnique } from '@4n6h4x0r/shrinkpath';

shrinkTo(path, 30)          // Hybrid strategy, target length
shrinkFish(path)            // Fish abbreviation, no length target
shrinkEllipsis(path, 30)    // Ellipsis strategy, target length
shrinkUnique(path)          // Unique disambiguation, no length target
```

### Full Options

```typescript
import { shrink } from '@4n6h4x0r/shrinkpath';
import type { ShrinkOptions } from '@4n6h4x0r/shrinkpath';

const result = shrink(path, {
  maxLen: 30,                                 // target max length
  strategy: 'ellipsis',                       // 'hybrid' | 'fish' | 'ellipsis' | 'unique'
  pathStyle: 'windows',                       // 'unix' | 'windows' (auto-detected by default)
  ellipsis: '..',                             // custom ellipsis marker
  dirLength: 2,                               // chars per abbreviated segment (fish/hybrid)
  fullLengthDirs: 1,                          // keep last N dirs unabbreviated (fish)
  anchors: ['src'],                           // never abbreviate these segments
  mappedLocations: [['/home/john', '~']],     // prefix substitutions
});
```

### Detailed Result

```typescript
import { shrinkDetailed } from '@4n6h4x0r/shrinkpath';
import type { ShrinkResult, SegmentInfo } from '@4n6h4x0r/shrinkpath';

const result: ShrinkResult = shrinkDetailed(path, { maxLen: 30 });

result.shortened      // the shortened path string
result.originalLen    // original path length
result.shortenedLen   // shortened path length
result.wasTruncated   // whether the path was modified
result.detectedStyle  // 'unix' | 'windows'
result.segments       // SegmentInfo[] with per-segment metadata
```

### Types

```typescript
import type {
  Strategy,       // 'hybrid' | 'fish' | 'ellipsis' | 'unique'
  PathStyle,      // 'unix' | 'windows'
  ShrinkOptions,  // options object
  ShrinkResult,   // detailed result
  SegmentInfo,    // per-segment metadata
} from '@4n6h4x0r/shrinkpath';
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

Detection is automatic. Use `pathStyle` to override.

## How It Works

Every input path is parsed into three parts:

```
  prefix       segments (directories)         filename
    │          │                                │
    ▼          ▼                                ▼
    /    home / john / projects / rust / src /  lib.rs
   ~~   ~~~~~~~~~~~~~~~~~~~~~~~~~~────────────  ~~~~~~
```

Each segment is classified by importance:

| Priority | What | Example | Dropped |
|---|---|---|---|
| **Sacred** | Filename | `lib.rs` | Never |
| **Identity** | Username / profile | `john`, `Admin` | Last |
| **Context** | Home root | `home`, `Users` | Middle |
| **Expendable** | Everything else | `projects`, `src` | First |

Identity is detected by recognizing the segment after a home root (`home` on
Unix, `Users` on Windows). The tilde prefix (`~`) encodes identity implicitly.

## Also Available

- **Rust crate:** [`shrinkpath` on crates.io](https://crates.io/crates/shrinkpath) (the original implementation)

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT License](LICENSE-MIT) at your option.

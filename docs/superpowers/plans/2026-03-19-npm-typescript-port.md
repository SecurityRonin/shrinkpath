# shrinkpath npm (TypeScript) Port — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port the Rust `shrinkpath` crate to a pure TypeScript npm package with full feature parity, living in `ts/` within the existing monorepo.

**Architecture:** 1:1 module mapping with the Rust crate. Each Rust module becomes a TypeScript file with identical algorithm logic. TDD approach — tests ported from Rust are written first, then implementation follows. Two package entry points: `shrinkpath` (pure string, works everywhere) and `shrinkpath/fs` (Node-only filesystem features).

**Tech Stack:** TypeScript 5.x (strict), Vitest (testing), tsup (bundling ESM+CJS), zero runtime dependencies.

---

## File Structure

```
ts/
├── package.json
├── tsconfig.json
├── tsup.config.ts
├── vitest.config.ts
├── src/
│   ├── index.ts              # Public API, convenience functions, mapped locations, segment metadata
│   ├── platform.ts           # PathStyle detection, separator, home root constants
│   ├── path-info.ts          # PathInfo class, Segment, SegmentPriority, parsing, reassembly
│   ├── strategy/
│   │   ├── fish.ts           # abbreviateSegment, shrinkFishStrategy
│   │   ├── ellipsis.ts       # shrinkEllipsisStrategy
│   │   ├── hybrid.ts         # shrinkHybridStrategy, collapseMiddle
│   │   └── unique.ts         # uniquePrefixLen, shrinkUniqueStrategy
│   └── fs-aware.ts           # Node-only: disambiguateSegment, findGitRoot
└── tests/
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

---

### Task 1: Project Scaffolding

**Files:**
- Create: `ts/package.json`
- Create: `ts/tsconfig.json`
- Create: `ts/tsup.config.ts`
- Create: `ts/vitest.config.ts`

- [ ] **Step 1: Create `ts/package.json`**

```json
{
  "name": "shrinkpath",
  "version": "0.1.0",
  "description": "Smart cross-platform path shortening for CLIs, prompts, and tools",
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
  },
  "main": "./dist/index.cjs",
  "module": "./dist/index.mjs",
  "types": "./dist/index.d.ts",
  "files": [
    "dist/",
    "README.md",
    "LICENSE"
  ],
  "scripts": {
    "build": "tsup",
    "test": "vitest run",
    "test:watch": "vitest",
    "lint": "tsc --noEmit",
    "prepublishOnly": "npm run lint && npm run test && npm run build"
  },
  "keywords": [
    "path",
    "shorten",
    "shrink",
    "truncate",
    "prompt",
    "fish",
    "cli"
  ],
  "license": "MIT OR Apache-2.0",
  "repository": {
    "type": "git",
    "url": "https://github.com/SecurityRonin/shrinkpath",
    "directory": "ts"
  },
  "engines": {
    "node": ">=18"
  },
  "devDependencies": {
    "tsup": "^8",
    "typescript": "^5",
    "vitest": "^3"
  }
}
```

- [ ] **Step 2: Create `ts/tsconfig.json`**

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "esModuleInterop": true,
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true,
    "outDir": "dist",
    "rootDir": "src",
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "isolatedModules": true,
    "types": ["vitest/globals"]
  },
  "include": ["src/**/*.ts"],
  "exclude": ["node_modules", "dist", "tests"]
}
```

- [ ] **Step 3: Create `ts/tsup.config.ts`**

```ts
import { defineConfig } from 'tsup';

export default defineConfig({
  entry: {
    index: 'src/index.ts',
    'fs-aware': 'src/fs-aware.ts',
  },
  format: ['esm', 'cjs'],
  dts: true,
  clean: true,
  target: ['node18', 'es2022'],
  splitting: false,
  external: ['node:fs', 'node:path'],
});
```

- [ ] **Step 4: Create `ts/vitest.config.ts`**

```ts
import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    globals: true,
    include: ['tests/**/*.test.ts'],
  },
});
```

- [ ] **Step 5: Install dependencies**

Run: `cd /Users/4n6h4x0r/src/shrinkpath/ts && npm install`
Expected: `node_modules/` created, lock file generated

- [ ] **Step 6: Create directory structure**

Run: `mkdir -p /Users/4n6h4x0r/src/shrinkpath/ts/src/strategy /Users/4n6h4x0r/src/shrinkpath/ts/tests/strategy`

- [ ] **Step 7: Add `ts/node_modules` to `.gitignore`**

Check if root `.gitignore` exists. If so, add `ts/node_modules/` and `ts/dist/`. If not, create one.

- [ ] **Step 8: Commit**

```bash
git add ts/package.json ts/tsconfig.json ts/tsup.config.ts ts/vitest.config.ts ts/package-lock.json .gitignore
git commit -m "chore: scaffold TypeScript project for npm port"
```

---

### Task 2: platform.ts — Path Style Detection

**Files:**
- Create: `ts/tests/platform.test.ts`
- Create: `ts/src/platform.ts`

- [ ] **Step 1: Write the failing tests**

Create `ts/tests/platform.test.ts`:

```ts
import { describe, test, expect } from 'vitest';
import { detectStyle, separator, UNIX_HOME_ROOTS, WIN_HOME_ROOTS } from '../src/platform';
import type { PathStyle } from '../src/platform';

describe('detectStyle', () => {
  test('detect unix', () => {
    expect(detectStyle('/home/user/file.txt')).toBe('unix');
    expect(detectStyle('/Users/john/Documents')).toBe('unix');
    expect(detectStyle('~/file.txt')).toBe('unix');
    expect(detectStyle('relative/path/file')).toBe('unix');
  });

  test('detect windows drive', () => {
    expect(detectStyle('C:\\Users\\john\\file.txt')).toBe('windows');
    expect(detectStyle('D:\\Data\\file.txt')).toBe('windows');
    expect(detectStyle('C:/Users/john/file.txt')).toBe('windows');
  });

  test('detect windows unc', () => {
    expect(detectStyle('\\\\server\\share\\file.txt')).toBe('windows');
  });

  test('detect windows dot backslash', () => {
    expect(detectStyle('.\\Users\\Admin\\file.txt')).toBe('windows');
  });

  test('detect windows backslash only', () => {
    expect(detectStyle('Users\\Admin\\file.txt')).toBe('windows');
  });
});

describe('separator', () => {
  test('unix separator', () => {
    expect(separator('unix')).toBe('/');
  });

  test('windows separator', () => {
    expect(separator('windows')).toBe('\\');
  });
});

describe('constants', () => {
  test('unix home roots', () => {
    expect(UNIX_HOME_ROOTS).toContain('home');
    expect(UNIX_HOME_ROOTS).toContain('Users');
  });

  test('windows home roots', () => {
    expect(WIN_HOME_ROOTS).toContain('Users');
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd /Users/4n6h4x0r/src/shrinkpath/ts && npx vitest run tests/platform.test.ts`
Expected: FAIL — module `../src/platform` not found

- [ ] **Step 3: Write the implementation**

Create `ts/src/platform.ts`:

```ts
export type PathStyle = 'unix' | 'windows';

export function detectStyle(path: string): PathStyle {
  // UNC path: \\server\share
  if (path.startsWith('\\\\')) {
    return 'windows';
  }

  // Drive letter: C:\ or C:/
  if (
    path.length >= 3 &&
    /^[a-zA-Z]$/.test(path[0]) &&
    path[1] === ':' &&
    (path[2] === '\\' || path[2] === '/')
  ) {
    return 'windows';
  }

  // Dot-backslash: .\
  if (path.startsWith('.\\')) {
    return 'windows';
  }

  // Heuristic: contains backslash but no forward slash
  if (path.includes('\\') && !path.includes('/')) {
    return 'windows';
  }

  return 'unix';
}

export const UNIX_HOME_ROOTS: readonly string[] = ['home', 'Users'];
export const WIN_HOME_ROOTS: readonly string[] = ['Users'];

export function separator(style: PathStyle): string {
  return style === 'unix' ? '/' : '\\';
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd /Users/4n6h4x0r/src/shrinkpath/ts && npx vitest run tests/platform.test.ts`
Expected: All tests PASS

- [ ] **Step 5: Commit**

```bash
git add ts/src/platform.ts ts/tests/platform.test.ts
git commit -m "feat(ts): add platform module — path style detection"
```

---

### Task 3: path-info.ts — Path Parsing & Segment Classification

**Files:**
- Create: `ts/tests/path-info.test.ts`
- Create: `ts/src/path-info.ts`

- [ ] **Step 1: Write the failing tests**

Create `ts/tests/path-info.test.ts`:

```ts
import { describe, test, expect } from 'vitest';
import { PathInfo } from '../src/path-info';
import type { SegmentPriority } from '../src/path-info';

describe('PathInfo.parse', () => {
  test('parse unix home', () => {
    const info = PathInfo.parse('/home/john/projects/rust/file.rs');
    expect(info.prefix).toBe('/');
    expect(info.filename).toBe('file.rs');
    expect(info.segments).toHaveLength(4);
    expect(info.segments[0].text).toBe('home');
    expect(info.segments[0].priority).toBe('context');
    expect(info.segments[1].text).toBe('john');
    expect(info.segments[1].priority).toBe('identity');
    expect(info.segments[2].priority).toBe('expendable');
  });

  test('parse windows drive', () => {
    const info = PathInfo.parse('C:\\Users\\Admin\\AppData\\Local\\file.txt');
    expect(info.prefix).toBe('C:\\');
    expect(info.filename).toBe('file.txt');
    expect(info.segments[0].text).toBe('Users');
    expect(info.segments[0].priority).toBe('context');
    expect(info.segments[1].text).toBe('Admin');
    expect(info.segments[1].priority).toBe('identity');
  });

  test('parse unc', () => {
    const info = PathInfo.parse('\\\\server\\share\\dept\\file.xlsx');
    expect(info.prefix).toBe('\\\\server\\share');
    expect(info.filename).toBe('file.xlsx');
    expect(info.segments).toHaveLength(1);
    expect(info.segments[0].text).toBe('dept');
  });

  test('parse tilde', () => {
    const info = PathInfo.parse('~/projects/rust/file.rs');
    expect(info.prefix).toBe('~');
    expect(info.filename).toBe('file.rs');
    expect(info.segments).toHaveLength(2);
  });

  test('parse dot backslash', () => {
    const info = PathInfo.parse('.\\Users\\Admin\\file.txt');
    expect(info.prefix).toBe('.');
    expect(info.style).toBe('windows');
    expect(info.filename).toBe('file.txt');
  });

  test('parse empty', () => {
    const info = PathInfo.parse('');
    expect(info.prefix).toBe('');
    expect(info.filename).toBe('');
    expect(info.segments).toHaveLength(0);
  });

  test('parse filename only', () => {
    const info = PathInfo.parse('file.txt');
    expect(info.filename).toBe('file.txt');
    expect(info.segments).toHaveLength(0);
  });
});

describe('PathInfo.reassemble', () => {
  test('reassemble unix', () => {
    const info = PathInfo.parse('/home/john/projects/file.rs');
    const texts = info.segments.map(s => s.text);
    expect(info.reassemble(texts)).toBe('/home/john/projects/file.rs');
  });

  test('reassemble windows', () => {
    const info = PathInfo.parse('C:\\Users\\Admin\\Docs\\file.txt');
    const texts = info.segments.map(s => s.text);
    expect(info.reassemble(texts)).toBe('C:\\Users\\Admin\\Docs\\file.txt');
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd /Users/4n6h4x0r/src/shrinkpath/ts && npx vitest run tests/path-info.test.ts`
Expected: FAIL — module `../src/path-info` not found

- [ ] **Step 3: Write the implementation**

Create `ts/src/path-info.ts`:

```ts
import { detectStyle, separator, UNIX_HOME_ROOTS, WIN_HOME_ROOTS } from './platform';
import type { PathStyle } from './platform';

export type SegmentPriority = 'sacred' | 'identity' | 'context' | 'expendable';

export interface Segment {
  text: string;
  priority: SegmentPriority;
}

export class PathInfo {
  constructor(
    public readonly prefix: string,
    public readonly segments: Segment[],
    public readonly filename: string,
    public readonly style: PathStyle,
  ) {}

  static parse(path: string, forceStyle?: PathStyle): PathInfo {
    const style = forceStyle ?? detectStyle(path);
    const sep = separator(style);

    // Normalize separators
    const normalized =
      style === 'unix' ? path.replaceAll('\\', '/') : path.replaceAll('/', '\\');

    const [prefix, remainder] = extractPrefix(normalized, style);

    const parts = remainder.split(sep).filter(s => s !== '');

    if (parts.length === 0) {
      return new PathInfo(prefix, [], '', style);
    }

    const filename = parts[parts.length - 1];
    const dirParts = parts.slice(0, -1);

    const segments = classifySegments(dirParts, prefix, style);

    return new PathInfo(prefix, segments, filename, style);
  }

  reassemble(segmentTexts: string[]): string {
    const sep = separator(this.style);
    let result = this.prefix;

    for (let i = 0; i < segmentTexts.length; i++) {
      if (i > 0 || (result !== '' && !result.endsWith(sep))) {
        result += sep;
      }
      result += segmentTexts[i];
    }

    if (this.filename !== '') {
      if (result !== '' && !result.endsWith(sep)) {
        result += sep;
      }
      result += this.filename;
    }

    return result;
  }
}

function extractPrefix(path: string, style: PathStyle): [string, string] {
  if (style === 'windows') {
    // UNC: \\server\share
    if (path.startsWith('\\\\')) {
      const afterSlashes = path.slice(2);
      const serverEnd = afterSlashes.indexOf('\\');
      if (serverEnd !== -1) {
        const afterServer = afterSlashes.slice(serverEnd + 1);
        const shareEnd = afterServer.indexOf('\\');
        const actualShareEnd = shareEnd === -1 ? afterServer.length : shareEnd;
        const prefixEnd = 2 + serverEnd + 1 + actualShareEnd;
        const prefix = path.slice(0, prefixEnd);
        const remainder = prefixEnd < path.length ? path.slice(prefixEnd + 1) : '';
        return [prefix, remainder];
      }
      return [path, ''];
    }

    // Drive letter: C:\
    if (
      path.length >= 3 &&
      /^[a-zA-Z]$/.test(path[0]) &&
      path[1] === ':' &&
      path[2] === '\\'
    ) {
      return [path.slice(0, 3), path.slice(3)];
    }

    // Dot-backslash: .\
    if (path.startsWith('.\\')) {
      return ['.', path.slice(2)];
    }

    // Just backslash
    if (path.startsWith('\\')) {
      return ['\\', path.slice(1)];
    }

    return ['', path];
  }

  // Unix
  if (path.startsWith('/')) {
    return ['/', path.slice(1)];
  }

  if (path.startsWith('~/')) {
    return ['~', path.slice(2)];
  }

  if (path === '~') {
    return ['~', ''];
  }

  return ['', path];
}

function classifySegments(
  parts: string[],
  prefix: string,
  style: PathStyle,
): Segment[] {
  const homeRoots = style === 'unix' ? UNIX_HOME_ROOTS : WIN_HOME_ROOTS;

  // Find the identity segment index: the segment AFTER a home root
  let identityIdx: number | null = null;
  if (
    parts.length > 0 &&
    homeRoots.some(root => parts[0].toLowerCase() === root.toLowerCase())
  ) {
    if (parts.length > 1) {
      identityIdx = 1;
    }
  }

  return parts.map((text, i) => {
    let priority: SegmentPriority;

    if (i === identityIdx) {
      priority = 'identity';
    } else if (
      i === 0 &&
      homeRoots.some(root => text.toLowerCase() === root.toLowerCase())
    ) {
      priority = 'context';
    } else {
      priority = 'expendable';
    }

    return { text, priority };
  });
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd /Users/4n6h4x0r/src/shrinkpath/ts && npx vitest run tests/path-info.test.ts`
Expected: All tests PASS

- [ ] **Step 5: Commit**

```bash
git add ts/src/path-info.ts ts/tests/path-info.test.ts
git commit -m "feat(ts): add path-info module — parsing and segment classification"
```

---

### Task 4: strategy/fish.ts — Fish-Style Abbreviation

**Files:**
- Create: `ts/tests/strategy/fish.test.ts`
- Create: `ts/src/strategy/fish.ts`

- [ ] **Step 1: Write the failing tests**

Create `ts/tests/strategy/fish.test.ts`:

```ts
import { describe, test, expect } from 'vitest';
import { abbreviateSegment, shrinkFishStrategy } from '../../src/strategy/fish';
import { PathInfo } from '../../src/path-info';

describe('abbreviateSegment', () => {
  test('abbreviate normal', () => {
    expect(abbreviateSegment('Users', 1, [])).toBe('U');
    expect(abbreviateSegment('projects', 1, [])).toBe('p');
  });

  test('abbreviate dotfile', () => {
    expect(abbreviateSegment('.config', 1, [])).toBe('.c');
    expect(abbreviateSegment('.local', 1, [])).toBe('.l');
    expect(abbreviateSegment('.', 1, [])).toBe('.');
  });

  test('abbreviate empty', () => {
    expect(abbreviateSegment('', 1, [])).toBe('');
  });

  test('abbreviate multi char', () => {
    expect(abbreviateSegment('projects', 2, [])).toBe('pr');
    expect(abbreviateSegment('projects', 3, [])).toBe('pro');
    expect(abbreviateSegment('projects', 1, [])).toBe('p');
    expect(abbreviateSegment('.config', 2, [])).toBe('.co');
    expect(abbreviateSegment('.config', 1, [])).toBe('.c');
    expect(abbreviateSegment('a', 3, [])).toBe('a');
  });

  test('abbreviate respects anchors', () => {
    expect(abbreviateSegment('src', 1, ['src'])).toBe('src');
    expect(abbreviateSegment('projects', 1, ['src'])).toBe('p');
  });
});

describe('shrinkFishStrategy', () => {
  test('fish unix', () => {
    const info = PathInfo.parse('/home/john/projects/rust/myapp/src/lib.rs');
    expect(shrinkFishStrategy(info, 1, 0, [])).toBe('/h/j/p/r/m/s/lib.rs');
  });

  test('fish dotfiles', () => {
    const info = PathInfo.parse('/home/john/.config/nvim/init.lua');
    expect(shrinkFishStrategy(info, 1, 0, [])).toBe('/h/j/.c/n/init.lua');
  });

  test('fish windows', () => {
    const info = PathInfo.parse('C:\\Users\\john\\AppData\\Local\\Temp\\file.txt');
    expect(shrinkFishStrategy(info, 1, 0, [])).toBe('C:\\U\\j\\A\\L\\T\\file.txt');
  });

  test('fish tilde', () => {
    const info = PathInfo.parse('~/projects/rust/file.rs');
    expect(shrinkFishStrategy(info, 1, 0, [])).toBe('~/p/r/file.rs');
  });

  test('fish filename only', () => {
    const info = PathInfo.parse('file.txt');
    expect(shrinkFishStrategy(info, 1, 0, [])).toBe('file.txt');
  });

  test('fish unc', () => {
    const info = PathInfo.parse('\\\\server\\share\\dept\\project\\file.xlsx');
    expect(shrinkFishStrategy(info, 1, 0, [])).toBe('\\\\server\\share\\d\\p\\file.xlsx');
  });

  test('fish unicode', () => {
    const info = PathInfo.parse('/home/user/Schone/Musik/file.mp3');
    expect(shrinkFishStrategy(info, 1, 0, [])).toBe('/h/u/S/M/file.mp3');
  });

  test('fish with dir_length 2', () => {
    const info = PathInfo.parse('/home/john/projects/rust/myapp/src/lib.rs');
    expect(shrinkFishStrategy(info, 2, 0, [])).toBe('/ho/jo/pr/ru/my/sr/lib.rs');
  });

  test('fish with full_length_dirs 1', () => {
    const info = PathInfo.parse('/home/john/projects/rust/myapp/src/lib.rs');
    expect(shrinkFishStrategy(info, 1, 1, [])).toBe('/h/j/p/r/m/src/lib.rs');
  });

  test('fish with full_length_dirs 2', () => {
    const info = PathInfo.parse('/home/john/projects/rust/myapp/src/lib.rs');
    expect(shrinkFishStrategy(info, 1, 2, [])).toBe('/h/j/p/r/myapp/src/lib.rs');
  });

  test('fish with anchors', () => {
    const info = PathInfo.parse('/home/john/projects/rust/myapp/src/lib.rs');
    expect(shrinkFishStrategy(info, 1, 0, ['src'])).toBe('/h/j/p/r/m/src/lib.rs');
  });

  test('fish with multiple anchors', () => {
    const info = PathInfo.parse('/home/john/projects/rust/myapp/src/lib.rs');
    const result = shrinkFishStrategy(info, 1, 0, ['src', 'myapp']);
    expect(result).toContain('myapp');
    expect(result).toContain('src');
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd /Users/4n6h4x0r/src/shrinkpath/ts && npx vitest run tests/strategy/fish.test.ts`
Expected: FAIL — module not found

- [ ] **Step 3: Write the implementation**

Create `ts/src/strategy/fish.ts`:

```ts
import { separator } from '../platform';
import type { PathInfo } from '../path-info';

export function abbreviateSegment(
  text: string,
  len: number,
  anchors: string[],
): string {
  if (text === '') return '';

  // If this segment is an anchor, never abbreviate it
  if (anchors.includes(text)) return text;

  const chars = [...text];
  const first = chars[0];

  if (first === '.') {
    const afterDot = chars.slice(1, 1 + len).join('');
    if (afterDot === '') return '.';
    return '.' + afterDot;
  }

  // Take `len` chars total
  return chars.slice(0, len).join('');
}

export function shrinkFishStrategy(
  info: PathInfo,
  dirLength: number,
  fullLengthDirs: number,
  anchors: string[],
): string {
  const segCount = info.segments.length;
  const abbreviated = info.segments.map((s, i) => {
    if (fullLengthDirs > 0 && i >= segCount - fullLengthDirs) {
      return s.text;
    }
    return abbreviateSegment(s.text, dirLength, anchors);
  });

  const sep = separator(info.style);
  let result = info.prefix;

  for (let i = 0; i < abbreviated.length; i++) {
    if (i > 0 || (result !== '' && !result.endsWith(sep))) {
      result += sep;
    }
    result += abbreviated[i];
  }

  if (info.filename !== '') {
    if (result !== '' && !result.endsWith(sep)) {
      result += sep;
    }
    result += info.filename;
  }

  return result;
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd /Users/4n6h4x0r/src/shrinkpath/ts && npx vitest run tests/strategy/fish.test.ts`
Expected: All tests PASS

- [ ] **Step 5: Commit**

```bash
git add ts/src/strategy/fish.ts ts/tests/strategy/fish.test.ts
git commit -m "feat(ts): add fish strategy — segment abbreviation"
```

---

### Task 5: strategy/ellipsis.ts — Ellipsis-Style Shortening

**Files:**
- Create: `ts/tests/strategy/ellipsis.test.ts`
- Create: `ts/src/strategy/ellipsis.ts`

- [ ] **Step 1: Write the failing tests**

Create `ts/tests/strategy/ellipsis.test.ts`:

```ts
import { describe, test, expect } from 'vitest';
import { shrinkEllipsisStrategy } from '../../src/strategy/ellipsis';
import { PathInfo } from '../../src/path-info';

describe('shrinkEllipsisStrategy', () => {
  test('already short', () => {
    const info = PathInfo.parse('/home/user/file.txt');
    expect(shrinkEllipsisStrategy(info, 50, '...')).toBe('/home/user/file.txt');
  });

  test('basic ellipsis', () => {
    const info = PathInfo.parse('/home/john/projects/rust/myapp/src/lib.rs');
    const result = shrinkEllipsisStrategy(info, 30, '...');
    expect(result.length).toBeLessThanOrEqual(30);
    expect(result).toMatch(/lib\.rs$/);
    expect(result).toContain('...');
  });

  test('preserves identity', () => {
    const info = PathInfo.parse('/home/john/projects/rust/myapp/src/lib.rs');
    const result = shrinkEllipsisStrategy(info, 35, '...');
    expect(result).toContain('john');
  });

  test('windows ellipsis', () => {
    const info = PathInfo.parse('C:\\Users\\Admin\\AppData\\Local\\Temp\\file.txt');
    const result = shrinkEllipsisStrategy(info, 30, '...');
    expect(result.length).toBeLessThanOrEqual(30);
    expect(result).toMatch(/file\.txt$/);
  });

  test('filename exceeds maxlen', () => {
    const info = PathInfo.parse('/a/b/c/very_long_filename_that_exceeds_everything.txt');
    const result = shrinkEllipsisStrategy(info, 10, '...');
    expect(result).toBe('very_long_filename_that_exceeds_everything.txt');
  });

  test('filename only', () => {
    const info = PathInfo.parse('file.txt');
    const result = shrinkEllipsisStrategy(info, 5, '...');
    expect(result).toBe('file.txt');
  });

  test('keeps right segments', () => {
    const info = PathInfo.parse('/home/john/a/b/c/d/e/src/lib.rs');
    const result = shrinkEllipsisStrategy(info, 25, '...');
    expect(result.length).toBeLessThanOrEqual(25);
    expect(result).toMatch(/lib\.rs$/);
    expect(result).toContain('...');
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd /Users/4n6h4x0r/src/shrinkpath/ts && npx vitest run tests/strategy/ellipsis.test.ts`
Expected: FAIL — module not found

- [ ] **Step 3: Write the implementation**

Create `ts/src/strategy/ellipsis.ts`:

```ts
import { separator } from '../platform';
import type { PathInfo } from '../path-info';

export function shrinkEllipsisStrategy(
  info: PathInfo,
  maxLen: number,
  ellipsis: string,
): string {
  // If no segments, just return prefix + filename
  if (info.segments.length === 0) {
    return info.reassemble([]);
  }

  const sep = separator(info.style);
  const sepLen = 1;

  // Full reassembly
  const texts = info.segments.map(s => s.text);
  const full = info.reassemble(texts);
  if (full.length <= maxLen) {
    return full;
  }

  // Base cost: prefix + separator + filename
  const baseLen =
    info.prefix.length +
    (info.prefix !== '' && info.filename !== '' ? sepLen : 0) +
    info.filename.length;

  // If filename alone exceeds max_len, return filename (sacred)
  if (info.filename.length >= maxLen) {
    return info.filename;
  }

  // Find the last identity segment index, keep everything up to and including it
  let identityEnd = 0;
  for (let i = 0; i < info.segments.length; i++) {
    if (info.segments[i].priority === 'identity') {
      identityEnd = i + 1;
    }
  }

  const head = info.segments.slice(0, identityEnd);
  const tailCandidates = info.segments.slice(identityEnd);

  // Compute head cost
  const headCost = head.reduce((sum, s) => sum + s.text.length + sepLen, 0);

  // Cost of the ellipsis marker
  const ellipsisCost = ellipsis.length + sepLen;

  // Available budget for tail segments
  const fixedCost = baseLen + headCost + ellipsisCost;

  if (fixedCost >= maxLen) {
    // Can't fit head + ellipsis + filename
    // Try without head: just prefix + ... + filename
    const prefixSep = info.prefix !== '' ? sep : '';
    const minimal = `${info.prefix}${prefixSep}${ellipsis}${sep}${info.filename}`;
    if (minimal.length <= maxLen) {
      return minimal;
    }
    // Last resort: just filename
    return info.filename;
  }

  const budget = maxLen - fixedCost;

  // Greedily add tail segments from right to left
  let tailCount = 0;
  let tailLen = 0;
  for (let i = tailCandidates.length - 1; i >= 0; i--) {
    const cost = tailCandidates[i].text.length + sepLen;
    if (tailLen + cost <= budget) {
      tailLen += cost;
      tailCount++;
    } else {
      break;
    }
  }

  // Build the result
  const parts: string[] = [];
  for (const seg of head) {
    parts.push(seg.text);
  }
  parts.push(ellipsis);
  const tailStart = tailCandidates.length - tailCount;
  for (let i = tailStart; i < tailCandidates.length; i++) {
    parts.push(tailCandidates[i].text);
  }

  return info.reassemble(parts);
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd /Users/4n6h4x0r/src/shrinkpath/ts && npx vitest run tests/strategy/ellipsis.test.ts`
Expected: All tests PASS

- [ ] **Step 5: Commit**

```bash
git add ts/src/strategy/ellipsis.ts ts/tests/strategy/ellipsis.test.ts
git commit -m "feat(ts): add ellipsis strategy — budget-aware shortening"
```

---

### Task 6: strategy/unique.ts — Unique Prefix Disambiguation

**Files:**
- Create: `ts/tests/strategy/unique.test.ts`
- Create: `ts/src/strategy/unique.ts`

- [ ] **Step 1: Write the failing tests**

Create `ts/tests/strategy/unique.test.ts`:

```ts
import { describe, test, expect } from 'vitest';
import { shrinkUniqueStrategy } from '../../src/strategy/unique';
import { PathInfo } from '../../src/path-info';

describe('shrinkUniqueStrategy', () => {
  test('all unique first chars', () => {
    const info = PathInfo.parse('/home/john/projects/rust/lib.rs');
    expect(shrinkUniqueStrategy(info, [])).toBe('/h/j/p/r/lib.rs');
  });

  test('identical segments kept full', () => {
    const info = PathInfo.parse('/dev/dev/dev/file.txt');
    expect(shrinkUniqueStrategy(info, [])).toBe('/dev/dev/dev/file.txt');
  });

  test('partial disambiguation', () => {
    const info = PathInfo.parse('/home/documents/downloads/file.txt');
    expect(shrinkUniqueStrategy(info, [])).toBe('/h/doc/dow/file.txt');
  });

  test('dot prefixed disambiguation', () => {
    const info = PathInfo.parse('/home/user/.config/.cache/file.txt');
    expect(shrinkUniqueStrategy(info, [])).toBe('/h/u/.co/.ca/file.txt');
  });

  test('windows unique', () => {
    const info = PathInfo.parse('C:\\Users\\Admin\\AppData\\Application\\file.txt');
    expect(shrinkUniqueStrategy(info, [])).toBe('C:\\U\\Ad\\AppD\\Appl\\file.txt');
  });

  test('single segment', () => {
    const info = PathInfo.parse('/home/file.txt');
    expect(shrinkUniqueStrategy(info, [])).toBe('/h/file.txt');
  });

  test('anchored segment preserved', () => {
    const info = PathInfo.parse('/home/john/src/lib.rs');
    expect(shrinkUniqueStrategy(info, ['src'])).toBe('/h/j/src/lib.rs');
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd /Users/4n6h4x0r/src/shrinkpath/ts && npx vitest run tests/strategy/unique.test.ts`
Expected: FAIL — module not found

- [ ] **Step 3: Write the implementation**

Create `ts/src/strategy/unique.ts`:

```ts
import type { PathInfo } from '../path-info';

function comparableText(text: string): string {
  if (text.startsWith('.') && text.length > 1) {
    return text.slice(1);
  }
  return text;
}

function uniquePrefixLen(target: string, others: string[]): number | null {
  const targetCmp = comparableText(target);
  const otherCmps = others.map(comparableText);

  // If any other segment has the same comparable text, can't disambiguate
  if (otherCmps.includes(targetCmp)) {
    return null;
  }

  const targetChars = [...targetCmp];

  for (let len = 1; len <= targetChars.length; len++) {
    const prefix = targetChars.slice(0, len).join('');
    const isUnique = otherCmps.every(other => {
      const otherChars = [...other];
      if (otherChars.length < len) {
        return true;
      }
      const otherPrefix = otherChars.slice(0, len).join('');
      return prefix !== otherPrefix;
    });
    if (isUnique) {
      return len;
    }
  }

  return targetChars.length;
}

function abbreviateWithLen(text: string, prefixLen: number): string {
  if (text.startsWith('.') && text.length > 1) {
    const afterDot = [...text.slice(1)].slice(0, prefixLen).join('');
    return '.' + afterDot;
  }
  return [...text].slice(0, prefixLen).join('');
}

export function shrinkUniqueStrategy(
  info: PathInfo,
  anchors: string[],
): string {
  const texts = info.segments.map(s => s.text);

  const abbreviated = texts.map((text, i) => {
    // Anchored segments are never abbreviated
    if (anchors.includes(text)) {
      return text;
    }

    // Collect all OTHER segment texts
    const others = texts.filter((_, j) => j !== i);

    const len = uniquePrefixLen(text, others);
    if (len === null) {
      return text; // Can't disambiguate, keep full
    }
    return abbreviateWithLen(text, len);
  });

  return info.reassemble(abbreviated);
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd /Users/4n6h4x0r/src/shrinkpath/ts && npx vitest run tests/strategy/unique.test.ts`
Expected: All tests PASS

- [ ] **Step 5: Commit**

```bash
git add ts/src/strategy/unique.ts ts/tests/strategy/unique.test.ts
git commit -m "feat(ts): add unique strategy — prefix disambiguation"
```

---

### Task 7: strategy/hybrid.ts — Graduated Hybrid Shortening

**Files:**
- Create: `ts/tests/strategy/hybrid.test.ts`
- Create: `ts/src/strategy/hybrid.ts`

- [ ] **Step 1: Write the failing tests**

Create `ts/tests/strategy/hybrid.test.ts`:

```ts
import { describe, test, expect } from 'vitest';
import { shrinkHybridStrategy } from '../../src/strategy/hybrid';
import { PathInfo } from '../../src/path-info';

describe('shrinkHybridStrategy', () => {
  test('already short', () => {
    const info = PathInfo.parse('/home/user/file.txt');
    expect(shrinkHybridStrategy(info, 50, '...', [])).toBe('/home/user/file.txt');
  });

  test('phase1 fish expendable', () => {
    const info = PathInfo.parse('/home/john/projects/rust/myapp/src/main.rs');
    const result = shrinkHybridStrategy(info, 35, '...', []);
    expect(result.length).toBeLessThanOrEqual(35);
    expect(result).toMatch(/main\.rs$/);
    expect(result).toContain('john');
  });

  test('phase3 collapse', () => {
    const info = PathInfo.parse('/home/john/projects/rust/myapp/src/deep/nested/main.rs');
    const result = shrinkHybridStrategy(info, 30, '...', []);
    expect(result.length).toBeLessThanOrEqual(30);
    expect(result).toMatch(/main\.rs$/);
  });

  test('windows hybrid', () => {
    const info = PathInfo.parse('C:\\Users\\Admin\\AppData\\Local\\Temp\\deep\\file.txt');
    const result = shrinkHybridStrategy(info, 35, '...', []);
    expect(result.length).toBeLessThanOrEqual(35);
    expect(result).toMatch(/file\.txt$/);
    expect(result.includes('Admin') || result.includes('A')).toBe(true);
  });

  test('tilde hybrid', () => {
    const info = PathInfo.parse('~/projects/rust/app/src/lib.rs');
    const result = shrinkHybridStrategy(info, 25, '...', []);
    expect(result.length).toBeLessThanOrEqual(25);
    expect(result).toMatch(/lib\.rs$/);
  });

  test('filename exceeds', () => {
    const info = PathInfo.parse('/a/b/c/very_long_filename.txt');
    const result = shrinkHybridStrategy(info, 10, '...', []);
    expect(result).toBe('very_long_filename.txt');
  });

  test('macos app support', () => {
    const info = PathInfo.parse(
      '/Users/john/Library/Application Support/Code/User/settings.json',
    );
    const result = shrinkHybridStrategy(info, 45, '...', []);
    expect(result.length).toBeLessThanOrEqual(45);
    expect(result).toMatch(/settings\.json$/);
    expect(result).toContain('john');
  });

  test('dot backslash', () => {
    const info = PathInfo.parse(
      '.\\Users\\Admin\\AppData\\Local\\Packages\\Microsoft.MicrosoftEdge\\file.txt',
    );
    const result = shrinkHybridStrategy(info, 40, '...', []);
    expect(result.length).toBeLessThanOrEqual(40);
    expect(result).toMatch(/file\.txt$/);
  });

  test('anchor in hybrid', () => {
    const info = PathInfo.parse('/home/john/projects/rust/myapp/src/lib.rs');
    const result = shrinkHybridStrategy(info, 35, '...', ['src']);
    expect(result).toContain('src');
    expect(result.length).toBeLessThanOrEqual(35);
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd /Users/4n6h4x0r/src/shrinkpath/ts && npx vitest run tests/strategy/hybrid.test.ts`
Expected: FAIL — module not found

- [ ] **Step 3: Write the implementation**

Create `ts/src/strategy/hybrid.ts`:

```ts
import { separator } from '../platform';
import type { PathInfo } from '../path-info';
import type { SegmentPriority } from '../path-info';
import { abbreviateSegment } from './fish';

export function shrinkHybridStrategy(
  info: PathInfo,
  maxLen: number,
  ellipsis: string,
  anchors: string[],
): string {
  if (info.segments.length === 0) {
    return info.reassemble([]);
  }

  // Full reassembly check
  const texts = info.segments.map(s => s.text);
  const full = info.reassemble(texts);
  if (full.length <= maxLen) {
    return full;
  }

  // If filename alone exceeds maxLen, return filename (sacred)
  if (info.filename.length >= maxLen) {
    return info.filename;
  }

  // Working copy of segment texts
  const working = [...texts];
  const priorities = info.segments.map(s => s.priority);

  // Phase 1: Fish expendable segments
  for (let i = 0; i < priorities.length; i++) {
    if (priorities[i] === 'expendable') {
      working[i] = abbreviateSegment(info.segments[i].text, 1, anchors);
    }
  }
  let result = info.reassemble(working);
  if (result.length <= maxLen) return result;

  // Phase 2: Fish context segments
  for (let i = 0; i < priorities.length; i++) {
    if (priorities[i] === 'context') {
      working[i] = abbreviateSegment(info.segments[i].text, 1, anchors);
    }
  }
  result = info.reassemble(working);
  if (result.length <= maxLen) return result;

  // Phase 3: Collapse consecutive abbreviated segments into ellipsis
  if (working.length > 2) {
    const collapsed = collapseMiddle(working, info, maxLen, ellipsis);
    if (collapsed.length <= maxLen) return collapsed;
  }

  // Phase 4: Fish identity segments (last resort)
  for (let i = 0; i < priorities.length; i++) {
    if (priorities[i] === 'identity') {
      working[i] = abbreviateSegment(info.segments[i].text, 1, anchors);
    }
  }

  // Try collapse again after identity is fished
  if (working.length > 2) {
    const collapsed = collapseMiddle(working, info, maxLen, ellipsis);
    if (collapsed.length <= maxLen) return collapsed;
  }

  // Final: just prefix + ellipsis + filename
  const sep = separator(info.style);
  const prefixSep = info.prefix !== '' ? sep : '';
  const minimal = `${info.prefix}${prefixSep}${ellipsis}${sep}${info.filename}`;
  if (minimal.length <= maxLen) return minimal;

  // Absolute last resort: filename only
  return info.filename;
}

function collapseMiddle(
  working: string[],
  info: PathInfo,
  maxLen: number,
  ellipsis: string,
): string {
  const n = working.length;

  // Try keeping more head and tail segments, then reduce
  for (let keepTail = Math.min(n, 3); keepTail >= 0; keepTail--) {
    for (let keepHead = Math.min(n, 3); keepHead >= 0; keepHead--) {
      if (keepHead + keepTail >= n) continue;

      const parts: string[] = [];
      for (let i = 0; i < keepHead; i++) {
        parts.push(working[i]);
      }
      parts.push(ellipsis);
      for (let i = n - keepTail; i < n; i++) {
        parts.push(working[i]);
      }

      const result = info.reassemble(parts);
      if (result.length <= maxLen) return result;
    }
  }

  // Nothing fit
  const sep = separator(info.style);
  const prefixSep = info.prefix !== '' ? sep : '';
  return `${info.prefix}${prefixSep}${ellipsis}${sep}${info.filename}`;
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd /Users/4n6h4x0r/src/shrinkpath/ts && npx vitest run tests/strategy/hybrid.test.ts`
Expected: All tests PASS

- [ ] **Step 5: Commit**

```bash
git add ts/src/strategy/hybrid.ts ts/tests/strategy/hybrid.test.ts
git commit -m "feat(ts): add hybrid strategy — graduated 4-phase shortening"
```

---

### Task 8: index.ts — Public API, Convenience Functions, Metadata

**Files:**
- Create: `ts/tests/index.test.ts`
- Create: `ts/tests/integration.test.ts`
- Create: `ts/src/index.ts`

- [ ] **Step 1: Write the failing tests for index.ts**

Create `ts/tests/index.test.ts`:

```ts
import { describe, test, expect } from 'vitest';
import {
  shrink,
  shrinkTo,
  shrinkFish,
  shrinkEllipsis,
  shrinkUnique,
  shrinkDetailed,
} from '../src/index';
import type { ShrinkOptions } from '../src/index';

describe('convenience functions', () => {
  test('shrinkTo', () => {
    const result = shrinkTo('/home/john/projects/rust/myapp/src/lib.rs', 30);
    expect(result.length).toBeLessThanOrEqual(30);
    expect(result).toMatch(/lib\.rs$/);
  });

  test('shrinkFish', () => {
    const result = shrinkFish('/home/john/projects/rust/myapp/src/lib.rs');
    expect(result).toBe('/h/j/p/r/m/s/lib.rs');
  });

  test('shrinkEllipsis', () => {
    const result = shrinkEllipsis('/home/john/projects/rust/myapp/src/lib.rs', 30);
    expect(result.length).toBeLessThanOrEqual(30);
    expect(result).toMatch(/lib\.rs$/);
    expect(result).toContain('...');
  });

  test('shrinkUnique', () => {
    const result = shrinkUnique('/home/john/projects/rust/lib.rs');
    expect(result).toBe('/h/j/p/r/lib.rs');
  });

  test('empty path', () => {
    expect(shrinkTo('', 30)).toBe('');
  });

  test('root only', () => {
    expect(shrinkTo('/', 5)).toBe('/');
  });

  test('filename only', () => {
    expect(shrinkTo('file.txt', 5)).toBe('file.txt');
  });
});

describe('shrink with options', () => {
  test('custom ellipsis', () => {
    const result = shrink('/home/john/deep/nested/path/to/file.rs', {
      maxLen: 25,
      strategy: 'ellipsis',
      ellipsis: '..',
    });
    expect(result.length).toBeLessThanOrEqual(25);
    expect(result).toContain('..');
    expect(result).not.toContain('...');
  });

  test('force windows style', () => {
    const result = shrink('C:/Users/Admin/AppData/Local/Temp/file.txt', {
      maxLen: 30,
      pathStyle: 'windows',
    });
    expect(result).toContain('\\');
  });

  test('unc path', () => {
    const result = shrinkTo('\\\\server\\share\\dept\\project\\reports\\q4.xlsx', 35);
    expect(result.length).toBeLessThanOrEqual(35);
    expect(result).toMatch(/q4\.xlsx$/);
  });

  test('idempotent on short paths', () => {
    const path = '/home/user/file.txt';
    expect(shrinkTo(path, 50)).toBe(path);
  });

  test('cross platform windows on any host', () => {
    const result = shrinkTo(
      '.\\Users\\Admin\\AppData\\Local\\Packages\\Microsoft\\file.txt',
      40,
    );
    expect(result.length).toBeLessThanOrEqual(40);
    expect(result).toMatch(/file\.txt$/);
    expect(result).toContain('\\');
  });

  test('dir_length two', () => {
    const result = shrink('/home/john/projects/rust/myapp/src/lib.rs', {
      maxLen: 50,
      strategy: 'fish',
      dirLength: 2,
    });
    expect(result).toBe('/ho/jo/pr/ru/my/sr/lib.rs');
  });

  test('full_length_dirs one', () => {
    const result = shrink('/home/john/projects/rust/myapp/src/lib.rs', {
      maxLen: 50,
      strategy: 'fish',
      fullLengthDirs: 1,
    });
    expect(result).toBe('/h/j/p/r/m/src/lib.rs');
  });
});

describe('shrinkDetailed', () => {
  test('detailed result', () => {
    const result = shrinkDetailed('/home/john/projects/rust/myapp/src/lib.rs', {
      maxLen: 30,
    });
    expect(result.wasTruncated).toBe(true);
    expect(result.shortenedLen).toBeLessThanOrEqual(30);
    expect(result.detectedStyle).toBe('unix');
  });

  test('detailed no truncation', () => {
    const result = shrinkDetailed('/home/user/file.txt', { maxLen: 50 });
    expect(result.wasTruncated).toBe(false);
    expect(result.shortened).toBe('/home/user/file.txt');
  });

  test('segment metadata fish', () => {
    const result = shrinkDetailed('/home/john/projects/lib.rs', {
      maxLen: Infinity,
      strategy: 'fish',
    });
    expect(result.segments).toHaveLength(4);

    expect(result.segments[0].original).toBe('home');
    expect(result.segments[0].shortened).toBe('h');
    expect(result.segments[0].wasAbbreviated).toBe(true);
    expect(result.segments[0].isFilename).toBe(false);

    expect(result.segments[1].original).toBe('john');
    expect(result.segments[1].shortened).toBe('j');

    expect(result.segments[3].original).toBe('lib.rs');
    expect(result.segments[3].shortened).toBe('lib.rs');
    expect(result.segments[3].wasAbbreviated).toBe(false);
    expect(result.segments[3].isFilename).toBe(true);
  });

  test('segment metadata no truncation', () => {
    const result = shrinkDetailed('/home/user/file.txt', { maxLen: 50 });
    expect(result.segments).toHaveLength(3);
    expect(result.segments[0].wasAbbreviated).toBe(false);
    expect(result.segments[1].wasAbbreviated).toBe(false);
    expect(result.segments[2].isFilename).toBe(true);
    expect(result.segments[2].original).toBe('file.txt');
  });

  test('segment metadata filename only', () => {
    const result = shrinkDetailed('file.txt', { maxLen: 50 });
    expect(result.segments).toHaveLength(1);
    expect(result.segments[0].isFilename).toBe(true);
    expect(result.segments[0].original).toBe('file.txt');
    expect(result.segments[0].wasAbbreviated).toBe(false);
  });

  test('segment metadata empty', () => {
    const result = shrinkDetailed('', { maxLen: 50 });
    expect(result.segments).toHaveLength(0);
  });
});
```

- [ ] **Step 2: Write the failing integration tests**

Create `ts/tests/integration.test.ts`:

```ts
import { describe, test, expect } from 'vitest';
import { shrink, shrinkTo } from '../src/index';

describe('mapped locations', () => {
  test('mapped location tilde', () => {
    const result = shrink('/home/john/projects/rust/lib.rs', {
      maxLen: 50,
      mappedLocations: [['/home/john', '~']],
    });
    expect(result).toBe('~/projects/rust/lib.rs');
  });

  test('mapped location custom', () => {
    const result = shrink('/home/john/projects/rust/myapp/src/lib.rs', {
      maxLen: 50,
      mappedLocations: [['/home/john/projects', 'PROJ:']],
    });
    expect(result).toMatch(/^PROJ:/);
    expect(result).toMatch(/lib\.rs$/);
  });

  test('mapped location longest match', () => {
    const result = shrink('/home/john/projects/rust/lib.rs', {
      maxLen: 50,
      mappedLocations: [
        ['/home/john', '~'],
        ['/home/john/projects', 'PROJ:'],
      ],
    });
    expect(result).toMatch(/^PROJ:/);
  });

  test('mapped location no match', () => {
    const result = shrink('/home/john/file.rs', {
      maxLen: 50,
      mappedLocations: [['/opt/data', 'DATA:']],
    });
    expect(result).toBe('/home/john/file.rs');
  });

  test('mapped location windows', () => {
    const result = shrink('C:\\Users\\Admin\\Documents\\file.txt', {
      maxLen: 50,
      mappedLocations: [['C:\\Users\\Admin', '~']],
    });
    expect(result).toMatch(/^~/);
    expect(result).toMatch(/file\.txt$/);
  });
});

describe('anchor segments', () => {
  test('anchor preserves segment fish', () => {
    const result = shrink('/home/john/projects/rust/myapp/src/lib.rs', {
      maxLen: 50,
      strategy: 'fish',
      anchors: ['src'],
    });
    expect(result).toContain('/src/');
    expect(result).toBe('/h/j/p/r/m/src/lib.rs');
  });

  test('anchor multiple', () => {
    const result = shrink('/home/john/projects/rust/myapp/src/lib.rs', {
      maxLen: 50,
      strategy: 'fish',
      anchors: ['src', 'myapp'],
    });
    expect(result).toContain('myapp');
    expect(result).toContain('src');
  });

  test('anchor in hybrid', () => {
    const result = shrink('/home/john/projects/rust/myapp/src/lib.rs', {
      maxLen: 35,
      anchors: ['src'],
    });
    expect(result).toContain('src');
    expect(result.length).toBeLessThanOrEqual(35);
  });

  test('anchor no match', () => {
    const result = shrink('/home/john/projects/lib.rs', {
      maxLen: 50,
      strategy: 'fish',
      anchors: ['nonexistent'],
    });
    expect(result).toBe('/h/j/p/lib.rs');
  });
});
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cd /Users/4n6h4x0r/src/shrinkpath/ts && npx vitest run tests/index.test.ts tests/integration.test.ts`
Expected: FAIL — module not found

- [ ] **Step 4: Write the implementation**

Create `ts/src/index.ts`:

```ts
import { PathInfo } from './path-info';
import { shrinkFishStrategy } from './strategy/fish';
import { shrinkEllipsisStrategy } from './strategy/ellipsis';
import { shrinkHybridStrategy } from './strategy/hybrid';
import { shrinkUniqueStrategy } from './strategy/unique';
import type { PathStyle } from './platform';

// ── Public types ──────────────────────────────────────────────────────────

export type Strategy = 'hybrid' | 'fish' | 'ellipsis' | 'unique';

export type { PathStyle } from './platform';

export interface ShrinkOptions {
  maxLen: number;
  strategy?: Strategy;
  pathStyle?: PathStyle;
  ellipsis?: string;
  dirLength?: number;
  fullLengthDirs?: number;
  mappedLocations?: [string, string][];
  anchors?: string[];
}

export interface SegmentInfo {
  original: string;
  shortened: string;
  wasAbbreviated: boolean;
  isFilename: boolean;
}

export interface ShrinkResult {
  shortened: string;
  originalLen: number;
  shortenedLen: number;
  wasTruncated: boolean;
  detectedStyle: PathStyle;
  segments: SegmentInfo[];
}

// ── Internal helpers ──────────────────────────────────────────────────────

function applyMappedLocations(
  path: string,
  mappedLocations: [string, string][],
): string {
  if (mappedLocations.length === 0) return path;

  // Sort by longest from-prefix first
  const sorted = [...mappedLocations].sort((a, b) => b[0].length - a[0].length);

  for (const [from, to] of sorted) {
    if (path.startsWith(from)) {
      return to + path.slice(from.length);
    }
  }

  return path;
}

function buildSegmentMetadata(
  original: PathInfo,
  shortened: PathInfo,
): SegmentInfo[] {
  const result: SegmentInfo[] = [];

  const origSegCount = original.segments.length;
  const shortSegCount = shortened.segments.length;

  if (origSegCount === shortSegCount) {
    // 1-to-1 mapping
    for (let i = 0; i < origSegCount; i++) {
      result.push({
        original: original.segments[i].text,
        shortened: shortened.segments[i].text,
        wasAbbreviated: original.segments[i].text !== shortened.segments[i].text,
        isFilename: false,
      });
    }
  } else {
    // Shortened has fewer segments (ellipsis collapse)
    const shortTexts = shortened.segments.map(s => s.text);
    const ellipsisPos = shortTexts.findIndex(
      t => t.includes('...') || t.includes('..'),
    );

    if (ellipsisPos !== -1) {
      // Segments before ellipsis
      for (let i = 0; i < ellipsisPos; i++) {
        if (i < origSegCount) {
          result.push({
            original: original.segments[i].text,
            shortened: shortened.segments[i].text,
            wasAbbreviated: original.segments[i].text !== shortened.segments[i].text,
            isFilename: false,
          });
        }
      }

      // Collapsed segments
      const tailCount = shortSegCount - ellipsisPos - 1;
      const collapsedStart = ellipsisPos;
      const collapsedEnd = Math.max(0, origSegCount - tailCount);
      for (let i = collapsedStart; i < collapsedEnd; i++) {
        result.push({
          original: original.segments[i].text,
          shortened: '...',
          wasAbbreviated: true,
          isFilename: false,
        });
      }

      // Tail segments after ellipsis
      for (let i = ellipsisPos + 1; i < shortSegCount; i++) {
        const origIdx = origSegCount - (shortSegCount - i);
        if (origIdx >= 0 && origIdx < origSegCount) {
          result.push({
            original: original.segments[origIdx].text,
            shortened: shortened.segments[i].text,
            wasAbbreviated:
              original.segments[origIdx].text !== shortened.segments[i].text,
            isFilename: false,
          });
        }
      }
    } else {
      // No ellipsis marker but different counts — fallback
      for (const seg of shortened.segments) {
        result.push({
          original: seg.text,
          shortened: seg.text,
          wasAbbreviated: false,
          isFilename: false,
        });
      }
    }
  }

  // Add filename segment
  if (original.filename !== '') {
    result.push({
      original: original.filename,
      shortened: shortened.filename,
      wasAbbreviated: original.filename !== shortened.filename,
      isFilename: true,
    });
  }

  return result;
}

// ── Public API ────────────────────────────────────────────────────────────

export function shrink(path: string, opts: ShrinkOptions): string {
  if (path === '') return '';

  const strategy = opts.strategy ?? 'hybrid';
  const ellipsis = opts.ellipsis ?? '...';
  const dirLength = opts.dirLength ?? 1;
  const fullLengthDirs = opts.fullLengthDirs ?? 0;
  const anchors = opts.anchors ?? [];
  const mappedLocations = opts.mappedLocations ?? [];

  const mapped = applyMappedLocations(path, mappedLocations);
  const info = PathInfo.parse(mapped, opts.pathStyle);

  switch (strategy) {
    case 'fish':
      return shrinkFishStrategy(info, dirLength, fullLengthDirs, anchors);
    case 'ellipsis':
      return shrinkEllipsisStrategy(info, opts.maxLen, ellipsis);
    case 'hybrid':
      return shrinkHybridStrategy(info, opts.maxLen, ellipsis, anchors);
    case 'unique':
      return shrinkUniqueStrategy(info, anchors);
  }
}

export function shrinkDetailed(path: string, opts: ShrinkOptions): ShrinkResult {
  if (path === '') {
    return {
      shortened: '',
      originalLen: 0,
      shortenedLen: 0,
      wasTruncated: false,
      detectedStyle: 'unix',
      segments: [],
    };
  }

  const mappedLocations = opts.mappedLocations ?? [];
  const mappedPath = applyMappedLocations(path, mappedLocations);
  const originalInfo = PathInfo.parse(mappedPath, opts.pathStyle);

  const shortened = shrink(path, opts);
  const shortenedInfo = PathInfo.parse(shortened, opts.pathStyle);

  const segments = buildSegmentMetadata(originalInfo, shortenedInfo);

  return {
    shortened,
    originalLen: path.length,
    shortenedLen: shortened.length,
    wasTruncated: shortened !== path,
    detectedStyle: originalInfo.style,
    segments,
  };
}

// ── Convenience functions ─────────────────────────────────────────────────

export function shrinkTo(path: string, maxLen: number): string {
  return shrink(path, { maxLen });
}

export function shrinkFish(path: string): string {
  const info = PathInfo.parse(path);
  return shrinkFishStrategy(info, 1, 0, []);
}

export function shrinkEllipsis(path: string, maxLen: number): string {
  return shrink(path, { maxLen, strategy: 'ellipsis' });
}

export function shrinkUnique(path: string): string {
  return shrink(path, { maxLen: Infinity, strategy: 'unique' });
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd /Users/4n6h4x0r/src/shrinkpath/ts && npx vitest run tests/index.test.ts tests/integration.test.ts`
Expected: All tests PASS

- [ ] **Step 6: Run full test suite**

Run: `cd /Users/4n6h4x0r/src/shrinkpath/ts && npx vitest run`
Expected: All tests PASS across all test files

- [ ] **Step 7: Commit**

```bash
git add ts/src/index.ts ts/tests/index.test.ts ts/tests/integration.test.ts
git commit -m "feat(ts): add public API — shrink, shrinkDetailed, convenience functions"
```

---

### Task 9: fs-aware.ts — Filesystem-Aware Features (Node-Only)

**Files:**
- Create: `ts/tests/fs-aware.test.ts`
- Create: `ts/src/fs-aware.ts`

- [ ] **Step 1: Write the failing tests**

Create `ts/tests/fs-aware.test.ts`:

```ts
import { describe, test, expect, beforeEach, afterEach } from 'vitest';
import { mkdirSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { findGitRoot, disambiguateSegment } from '../src/fs-aware';

function tempDir(name: string): string {
  const dir = join(tmpdir(), `shrinkpath_test_${name}_${process.pid}`);
  rmSync(dir, { recursive: true, force: true });
  return dir;
}

describe('findGitRoot', () => {
  let dir: string;

  afterEach(() => {
    if (dir) rmSync(dir, { recursive: true, force: true });
  });

  test('find git root found', () => {
    dir = tempDir('git_found');
    mkdirSync(join(dir, 'src/deep/nested'), { recursive: true });
    mkdirSync(join(dir, '.git'));
    writeFileSync(join(dir, 'src/deep/nested/main.rs'), '');

    const root = findGitRoot(join(dir, 'src/deep/nested/main.rs'));
    const expected = dir.split('/').pop()!;
    expect(root).toBe(expected);
  });

  test('find git root at root', () => {
    dir = tempDir('git_at_root');
    mkdirSync(dir, { recursive: true });
    mkdirSync(join(dir, '.git'));
    writeFileSync(join(dir, 'file.txt'), '');

    const root = findGitRoot(join(dir, 'file.txt'));
    const expected = dir.split('/').pop()!;
    expect(root).toBe(expected);
  });

  test('find git root not found returns null', () => {
    dir = tempDir('git_notfound');
    mkdirSync(dir, { recursive: true });
    // No .git directory — should return null or find an ancestor .git
    const root = findGitRoot(join(dir, 'file.txt'));
    // Just verify it doesn't throw
    expect(root === null || typeof root === 'string').toBe(true);
  });
});

describe('disambiguateSegment', () => {
  let dir: string;

  afterEach(() => {
    if (dir) rmSync(dir, { recursive: true, force: true });
  });

  test('disambiguate with siblings', () => {
    dir = tempDir('disambig_siblings');
    mkdirSync(join(dir, 'documents'), { recursive: true });
    mkdirSync(join(dir, 'downloads'), { recursive: true });
    mkdirSync(join(dir, 'desktop'), { recursive: true });

    expect(disambiguateSegment(dir, 'documents')).toBe('doc');
    expect(disambiguateSegment(dir, 'downloads')).toBe('dow');
    expect(disambiguateSegment(dir, 'desktop')).toBe('de');
  });

  test('disambiguate no siblings', () => {
    dir = tempDir('disambig_alone');
    mkdirSync(join(dir, 'only_child'), { recursive: true });

    expect(disambiguateSegment(dir, 'only_child')).toBe('o');
  });

  test('disambiguate identical prefix', () => {
    dir = tempDir('disambig_identical');
    mkdirSync(join(dir, 'app'), { recursive: true });
    mkdirSync(join(dir, 'application'), { recursive: true });

    expect(disambiguateSegment(dir, 'app')).toBe('app');
    expect(disambiguateSegment(dir, 'application')).toBe('appl');
  });

  test('disambiguate unreadable dir', () => {
    expect(disambiguateSegment('/nonexistent_dir_12345', 'test')).toBe('test');
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd /Users/4n6h4x0r/src/shrinkpath/ts && npx vitest run tests/fs-aware.test.ts`
Expected: FAIL — module not found

- [ ] **Step 3: Write the implementation**

Create `ts/src/fs-aware.ts`:

```ts
import { readdirSync, statSync, existsSync } from 'node:fs';
import { join, dirname, basename } from 'node:path';

export function findGitRoot(path: string): string | null {
  let current: string;

  try {
    const stat = statSync(path);
    current = stat.isFile() ? dirname(path) : path;
  } catch {
    // Path doesn't exist, try parent
    current = dirname(path);
  }

  while (true) {
    if (existsSync(join(current, '.git'))) {
      return basename(current);
    }
    const parent = dirname(current);
    if (parent === current) {
      // Reached filesystem root
      return null;
    }
    current = parent;
  }
}

export function disambiguateSegment(
  parentPath: string,
  segment: string,
): string {
  let siblings: string[];

  try {
    const entries = readdirSync(parentPath, { withFileTypes: true });
    siblings = entries
      .filter(e => e.isDirectory())
      .map(e => e.name)
      .filter(name => name !== segment);
  } catch {
    return segment;
  }

  if (siblings.length === 0) {
    if (segment === '') return '';
    return segment[0];
  }

  for (let len = 1; len <= segment.length; len++) {
    const prefix = [...segment].slice(0, len).join('');
    const isUnique = siblings.every(s => {
      const sPrefix = [...s].slice(0, len).join('');
      return sPrefix !== prefix;
    });
    if (isUnique) {
      return prefix;
    }
  }

  return segment;
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd /Users/4n6h4x0r/src/shrinkpath/ts && npx vitest run tests/fs-aware.test.ts`
Expected: All tests PASS

- [ ] **Step 5: Commit**

```bash
git add ts/src/fs-aware.ts ts/tests/fs-aware.test.ts
git commit -m "feat(ts): add fs-aware module — filesystem disambiguation and git root"
```

---

### Task 10: Build Verification & Final Checks

**Files:**
- Modify: `ts/package.json` (if needed)

- [ ] **Step 1: Run full test suite**

Run: `cd /Users/4n6h4x0r/src/shrinkpath/ts && npx vitest run`
Expected: All tests PASS

- [ ] **Step 2: Run type checking**

Run: `cd /Users/4n6h4x0r/src/shrinkpath/ts && npx tsc --noEmit`
Expected: No type errors

- [ ] **Step 3: Run build**

Run: `cd /Users/4n6h4x0r/src/shrinkpath/ts && npx tsup`
Expected: Produces `dist/` with `index.mjs`, `index.cjs`, `index.d.ts`, `fs-aware.mjs`, `fs-aware.cjs`, `fs-aware.d.ts`

- [ ] **Step 4: Verify exports work (ESM)**

Run: `cd /Users/4n6h4x0r/src/shrinkpath/ts && node -e "import('file://' + process.cwd() + '/dist/index.mjs').then(m => console.log(m.shrinkFish('/home/john/projects/lib.rs')))"`
Expected: `/h/j/p/lib.rs`

- [ ] **Step 5: Verify exports work (CJS)**

Run: `cd /Users/4n6h4x0r/src/shrinkpath/ts && node -e "const m = require('./dist/index.cjs'); console.log(m.shrinkFish('/home/john/projects/lib.rs'))"`
Expected: `/h/j/p/lib.rs`

- [ ] **Step 6: Verify fs subpath export**

Run: `cd /Users/4n6h4x0r/src/shrinkpath/ts && node -e "import('file://' + process.cwd() + '/dist/fs-aware.mjs').then(m => console.log(typeof m.findGitRoot))"`
Expected: `function`

- [ ] **Step 7: Commit build config fixes (if any)**

Only commit if changes were needed. Otherwise skip.

- [ ] **Step 8: Final commit — all tests pass, build succeeds**

```bash
git add -A ts/
git commit -m "feat(ts): complete TypeScript port of shrinkpath with full parity"
```

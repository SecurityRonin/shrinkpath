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

  test('segment metadata with ellipsis collapse', () => {
    // Triggers the buildSegmentMetadata branch where origSegCount !== shortSegCount
    // and an ellipsis marker is present in the shortened segments
    const result = shrinkDetailed('/home/john/a/b/c/d/e/src/lib.rs', {
      maxLen: 25,
      strategy: 'ellipsis',
    });
    expect(result.wasTruncated).toBe(true);
    expect(result.segments.length).toBeGreaterThan(0);
    // Should contain at least one segment marked as collapsed
    const collapsed = result.segments.filter(s => s.shortened === '...');
    expect(collapsed.length).toBeGreaterThan(0);
    expect(collapsed.every(s => s.wasAbbreviated)).toBe(true);
    // Last segment should be filename
    const last = result.segments[result.segments.length - 1];
    expect(last.isFilename).toBe(true);
    expect(last.original).toBe('lib.rs');
  });

  test('segment metadata with hybrid collapse', () => {
    // Hybrid with tight budget triggers collapse — different segment counts
    const result = shrinkDetailed(
      '/home/john/projects/rust/myapp/src/deep/nested/lib.rs',
      { maxLen: 25 },
    );
    expect(result.wasTruncated).toBe(true);
    expect(result.shortenedLen).toBeLessThanOrEqual(25);
    const last = result.segments[result.segments.length - 1];
    expect(last.isFilename).toBe(true);
  });

  test('segment metadata different counts no ellipsis marker fallback', () => {
    // Unicode ellipsis bypasses the ASCII '...'/'..'' detection in buildSegmentMetadata
    // causing segment counts to differ without a recognized ellipsis marker
    const result = shrinkDetailed('/home/john/a/b/c/d/e/src/lib.rs', {
      maxLen: 25,
      strategy: 'ellipsis',
      ellipsis: '\u2026', // Unicode ellipsis '…', not ASCII '...'
    });
    expect(result.wasTruncated).toBe(true);
    expect(result.segments.length).toBeGreaterThan(0);
    const last = result.segments[result.segments.length - 1];
    expect(last.isFilename).toBe(true);
    expect(last.original).toBe('lib.rs');
  });
});

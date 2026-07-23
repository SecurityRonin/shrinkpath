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

  test('phase2 fish context segments', () => {
    // Path where expendable fishing isn't enough, context must be fished too
    const info = PathInfo.parse('/home/john/projects/rust/lib.rs');
    const result = shrinkHybridStrategy(info, 18, '...', []);
    expect(result.length).toBeLessThanOrEqual(18);
    expect(result).toMatch(/lib\.rs$/);
  });

  test('phase4 fish identity last resort', () => {
    // Very tight budget forces identity abbreviation
    const info = PathInfo.parse('/home/john/projects/rust/myapp/src/lib.rs');
    const result = shrinkHybridStrategy(info, 20, '...', []);
    expect(result.length).toBeLessThanOrEqual(20);
    expect(result).toMatch(/lib\.rs$/);
  });

  test('collapse middle fallback to minimal', () => {
    // So tight that collapse can't fit anything — falls through to prefix+ellipsis+filename
    const info = PathInfo.parse('/home/john/projects/rust/myapp/src/lib.rs');
    const result = shrinkHybridStrategy(info, 15, '...', []);
    expect(result.length).toBeLessThanOrEqual(15);
    expect(result).toMatch(/lib\.rs$/);
  });

  test('no segments returns reassembled', () => {
    const info = PathInfo.parse('file.txt');
    expect(shrinkHybridStrategy(info, 5, '...', [])).toBe('file.txt');
  });

  test('collapse middle with empty prefix', () => {
    // Relative path — no prefix
    const info = PathInfo.parse('a/b/c/d/e/f/file.txt');
    const result = shrinkHybridStrategy(info, 15, '...', []);
    expect(result.length).toBeLessThanOrEqual(15);
    expect(result).toMatch(/file\.txt$/);
  });

  test('two segments not enough for collapse', () => {
    // Only 2 segments — collapse requires > 2
    const info = PathInfo.parse('/home/john/lib.rs');
    const result = shrinkHybridStrategy(info, 12, '...', []);
    expect(result.length).toBeLessThanOrEqual(12);
    expect(result).toMatch(/lib\.rs$/);
  });

  test('tight budget exhausts collapseMiddle and post-phase4 path', () => {
    // maxLen so tight that collapseMiddle loop minimum (/.../f.rs = 8) > maxLen
    // Forces: collapseMiddle fallback (line 98), post-phase4 collapse (lines 60-62),
    // minimal check fail, and filename-only return (line 69)
    const info = PathInfo.parse('/home/john/src/f.rs');
    const result = shrinkHybridStrategy(info, 7, '...', []);
    expect(result).toBe('f.rs');
  });

  test('tight budget with relative path exhausts collapseMiddle', () => {
    // Same exhaustion with empty prefix — tests prefixSep '' branch in
    // collapseMiddle fallback (line 99) and minimal computation (line 65)
    const info = PathInfo.parse('a/b/c/f.rs');
    const result = shrinkHybridStrategy(info, 6, '...', []);
    expect(result).toBe('f.rs');
  });

  test('two segments minimal too long returns filename', () => {
    // Only 2 segments — collapse skipped, minimal '//.../lib.rs' > maxLen, returns filename
    const info = PathInfo.parse('/home/john/lib.rs');
    const result = shrinkHybridStrategy(info, 11, '...', []);
    expect(result).toBe('lib.rs');
  });
});

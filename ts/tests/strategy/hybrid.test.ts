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

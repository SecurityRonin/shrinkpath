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

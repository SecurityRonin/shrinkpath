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

  test('shorter sibling during prefix comparison', () => {
    // "ab" vs "a" — when prefix len=2, "a" is shorter than len, so it's automatically unique
    const info = PathInfo.parse('/a/ab/file.txt');
    const result = shrinkUniqueStrategy(info, []);
    expect(result).toBe('/a/ab/file.txt'); // identical prefix "a" — "a" can't be shortened, "ab" needs full
  });

  test('segments with varying lengths', () => {
    // Tests the otherChars.length < len branch explicitly
    const info = PathInfo.parse('/x/xy/xyz/file.txt');
    const result = shrinkUniqueStrategy(info, []);
    expect(result).toBe('/x/xy/xyz/file.txt'); // all share prefix "x"
  });
});

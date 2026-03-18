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

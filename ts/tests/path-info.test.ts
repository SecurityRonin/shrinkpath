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

describe('PathInfo.parse edge cases', () => {
  test('parse unc without trailing path', () => {
    // UNC with server only, no backslash after server
    const info = PathInfo.parse('\\\\server');
    expect(info.prefix).toBe('\\\\server');
    expect(info.filename).toBe('');
    expect(info.segments).toHaveLength(0);
  });

  test('parse bare backslash prefix', () => {
    const info = PathInfo.parse('\\dir\\file.txt');
    expect(info.prefix).toBe('\\');
    expect(info.filename).toBe('file.txt');
    expect(info.segments[0].text).toBe('dir');
  });

  test('parse bare tilde', () => {
    const info = PathInfo.parse('~');
    expect(info.prefix).toBe('~');
    expect(info.filename).toBe('');
    expect(info.segments).toHaveLength(0);
  });

  test('parse relative windows path', () => {
    // No prefix at all in windows mode
    const info = PathInfo.parse('Users\\Admin\\file.txt');
    expect(info.prefix).toBe('');
    expect(info.style).toBe('windows');
    expect(info.filename).toBe('file.txt');
  });

  test('parse unc with share only no trailing content', () => {
    // \\server\share with nothing after
    const info = PathInfo.parse('\\\\server\\share');
    expect(info.prefix).toBe('\\\\server\\share');
    expect(info.filename).toBe('');
  });
});

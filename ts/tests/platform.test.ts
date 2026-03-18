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

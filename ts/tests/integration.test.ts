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

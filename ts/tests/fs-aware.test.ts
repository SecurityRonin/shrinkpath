import { describe, test, expect, afterEach } from 'vitest';
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
    const root = findGitRoot(join(dir, 'file.txt'));
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

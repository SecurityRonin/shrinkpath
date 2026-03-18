import { readdirSync, statSync, existsSync } from 'node:fs';
import { join, dirname, basename } from 'node:path';

export function findGitRoot(path: string): string | null {
  let current: string;

  try {
    const stat = statSync(path);
    current = stat.isFile() ? dirname(path) : path;
  } catch {
    current = dirname(path);
  }

  while (true) {
    if (existsSync(join(current, '.git'))) {
      return basename(current);
    }
    const parent = dirname(current);
    if (parent === current) {
      return null;
    }
    current = parent;
  }
}

export function disambiguateSegment(
  parentPath: string,
  segment: string,
): string {
  let siblings: string[];

  try {
    const entries = readdirSync(parentPath, { withFileTypes: true });
    siblings = entries
      .filter(e => e.isDirectory())
      .map(e => e.name)
      .filter(name => name !== segment);
  } catch {
    return segment;
  }

  if (siblings.length === 0) {
    if (segment === '') return '';
    return segment[0];
  }

  for (let len = 1; len <= segment.length; len++) {
    const prefix = [...segment].slice(0, len).join('');
    const isUnique = siblings.every(s => {
      const sPrefix = [...s].slice(0, len).join('');
      return sPrefix !== prefix;
    });
    if (isUnique) {
      return prefix;
    }
  }

  return segment;
}

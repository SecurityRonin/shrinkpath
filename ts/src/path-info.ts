import { detectStyle, separator, UNIX_HOME_ROOTS, WIN_HOME_ROOTS } from './platform';
import type { PathStyle } from './platform';

export type SegmentPriority = 'sacred' | 'identity' | 'context' | 'expendable';

export interface Segment {
  text: string;
  priority: SegmentPriority;
}

export class PathInfo {
  constructor(
    public readonly prefix: string,
    public readonly segments: Segment[],
    public readonly filename: string,
    public readonly style: PathStyle,
  ) {}

  static parse(path: string, forceStyle?: PathStyle): PathInfo {
    const style = forceStyle ?? detectStyle(path);
    const sep = separator(style);

    const normalized =
      style === 'unix' ? path.replaceAll('\\', '/') : path.replaceAll('/', '\\');

    const [prefix, remainder] = extractPrefix(normalized, style);

    const parts = remainder.split(sep).filter(s => s !== '');

    if (parts.length === 0) {
      return new PathInfo(prefix, [], '', style);
    }

    const filename = parts[parts.length - 1];
    const dirParts = parts.slice(0, -1);

    const segments = classifySegments(dirParts, prefix, style);

    return new PathInfo(prefix, segments, filename, style);
  }

  reassemble(segmentTexts: string[]): string {
    const sep = separator(this.style);
    let result = this.prefix;

    for (let i = 0; i < segmentTexts.length; i++) {
      if (i > 0 || (result !== '' && !result.endsWith(sep))) {
        result += sep;
      }
      result += segmentTexts[i];
    }

    if (this.filename !== '') {
      if (result !== '' && !result.endsWith(sep)) {
        result += sep;
      }
      result += this.filename;
    }

    return result;
  }
}

function extractPrefix(path: string, style: PathStyle): [string, string] {
  if (style === 'windows') {
    if (path.startsWith('\\\\')) {
      const afterSlashes = path.slice(2);
      const serverEnd = afterSlashes.indexOf('\\');
      if (serverEnd !== -1) {
        const afterServer = afterSlashes.slice(serverEnd + 1);
        const shareEnd = afterServer.indexOf('\\');
        const actualShareEnd = shareEnd === -1 ? afterServer.length : shareEnd;
        const prefixEnd = 2 + serverEnd + 1 + actualShareEnd;
        const prefix = path.slice(0, prefixEnd);
        const remainder = prefixEnd < path.length ? path.slice(prefixEnd + 1) : '';
        return [prefix, remainder];
      }
      return [path, ''];
    }

    if (
      path.length >= 3 &&
      /^[a-zA-Z]$/.test(path[0]) &&
      path[1] === ':' &&
      path[2] === '\\'
    ) {
      return [path.slice(0, 3), path.slice(3)];
    }

    if (path.startsWith('.\\')) {
      return ['.', path.slice(2)];
    }

    if (path.startsWith('\\')) {
      return ['\\', path.slice(1)];
    }

    return ['', path];
  }

  if (path.startsWith('/')) {
    return ['/', path.slice(1)];
  }

  if (path.startsWith('~/')) {
    return ['~', path.slice(2)];
  }

  if (path === '~') {
    return ['~', ''];
  }

  return ['', path];
}

function classifySegments(
  parts: string[],
  prefix: string,
  style: PathStyle,
): Segment[] {
  const homeRoots = style === 'unix' ? UNIX_HOME_ROOTS : WIN_HOME_ROOTS;

  let identityIdx: number | null = null;
  if (
    parts.length > 0 &&
    homeRoots.some(root => parts[0].toLowerCase() === root.toLowerCase())
  ) {
    if (parts.length > 1) {
      identityIdx = 1;
    }
  }

  return parts.map((text, i) => {
    let priority: SegmentPriority;

    if (i === identityIdx) {
      priority = 'identity';
    } else if (
      i === 0 &&
      homeRoots.some(root => text.toLowerCase() === root.toLowerCase())
    ) {
      priority = 'context';
    } else {
      priority = 'expendable';
    }

    return { text, priority };
  });
}

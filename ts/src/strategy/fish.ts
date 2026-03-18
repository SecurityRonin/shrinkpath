import { separator } from '../platform';
import type { PathInfo } from '../path-info';

export function abbreviateSegment(
  text: string,
  len: number,
  anchors: string[],
): string {
  if (text === '') return '';

  if (anchors.includes(text)) return text;

  const chars = [...text];
  const first = chars[0];

  if (first === '.') {
    const afterDot = chars.slice(1, 1 + len).join('');
    if (afterDot === '') return '.';
    return '.' + afterDot;
  }

  return chars.slice(0, len).join('');
}

export function shrinkFishStrategy(
  info: PathInfo,
  dirLength: number,
  fullLengthDirs: number,
  anchors: string[],
): string {
  const segCount = info.segments.length;
  const abbreviated = info.segments.map((s, i) => {
    if (fullLengthDirs > 0 && i >= segCount - fullLengthDirs) {
      return s.text;
    }
    return abbreviateSegment(s.text, dirLength, anchors);
  });

  const sep = separator(info.style);
  let result = info.prefix;

  for (let i = 0; i < abbreviated.length; i++) {
    if (i > 0 || (result !== '' && !result.endsWith(sep))) {
      result += sep;
    }
    result += abbreviated[i];
  }

  if (info.filename !== '') {
    if (result !== '' && !result.endsWith(sep)) {
      result += sep;
    }
    result += info.filename;
  }

  return result;
}

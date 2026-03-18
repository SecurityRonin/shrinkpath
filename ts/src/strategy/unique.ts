import type { PathInfo } from '../path-info';

function comparableText(text: string): string {
  if (text.startsWith('.') && text.length > 1) {
    return text.slice(1);
  }
  return text;
}

function uniquePrefixLen(target: string, others: string[]): number | null {
  const targetCmp = comparableText(target);
  const otherCmps = others.map(comparableText);

  if (otherCmps.includes(targetCmp)) {
    return null;
  }

  const targetChars = [...targetCmp];

  for (let len = 1; len <= targetChars.length; len++) {
    const prefix = targetChars.slice(0, len).join('');
    const isUnique = otherCmps.every(other => {
      const otherChars = [...other];
      if (otherChars.length < len) {
        return true;
      }
      const otherPrefix = otherChars.slice(0, len).join('');
      return prefix !== otherPrefix;
    });
    if (isUnique) {
      return len;
    }
  }

  return targetChars.length;
}

function abbreviateWithLen(text: string, prefixLen: number): string {
  if (text.startsWith('.') && text.length > 1) {
    const afterDot = [...text.slice(1)].slice(0, prefixLen).join('');
    return '.' + afterDot;
  }
  return [...text].slice(0, prefixLen).join('');
}

export function shrinkUniqueStrategy(
  info: PathInfo,
  anchors: string[],
): string {
  const texts = info.segments.map(s => s.text);

  const abbreviated = texts.map((text, i) => {
    if (anchors.includes(text)) {
      return text;
    }

    const others = texts.filter((_, j) => j !== i);

    const len = uniquePrefixLen(text, others);
    if (len === null) {
      return text;
    }
    return abbreviateWithLen(text, len);
  });

  return info.reassemble(abbreviated);
}

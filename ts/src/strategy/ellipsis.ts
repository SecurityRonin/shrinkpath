import { separator } from '../platform';
import type { PathInfo } from '../path-info';

export function shrinkEllipsisStrategy(
  info: PathInfo,
  maxLen: number,
  ellipsis: string,
): string {
  if (info.segments.length === 0) {
    return info.reassemble([]);
  }

  const sep = separator(info.style);
  const sepLen = 1;

  const texts = info.segments.map(s => s.text);
  const full = info.reassemble(texts);
  if (full.length <= maxLen) {
    return full;
  }

  const baseLen =
    info.prefix.length +
    (info.prefix !== '' && info.filename !== '' ? sepLen : 0) +
    info.filename.length;

  if (info.filename.length >= maxLen) {
    return info.filename;
  }

  let identityEnd = 0;
  for (let i = 0; i < info.segments.length; i++) {
    if (info.segments[i].priority === 'identity') {
      identityEnd = i + 1;
    }
  }

  const head = info.segments.slice(0, identityEnd);
  const tailCandidates = info.segments.slice(identityEnd);

  const headCost = head.reduce((sum, s) => sum + s.text.length + sepLen, 0);
  const ellipsisCost = ellipsis.length + sepLen;
  const fixedCost = baseLen + headCost + ellipsisCost;

  if (fixedCost >= maxLen) {
    const prefixSep = info.prefix !== '' ? sep : '';
    const minimal = `${info.prefix}${prefixSep}${ellipsis}${sep}${info.filename}`;
    if (minimal.length <= maxLen) {
      return minimal;
    }
    return info.filename;
  }

  const budget = maxLen - fixedCost;

  let tailCount = 0;
  let tailLen = 0;
  for (let i = tailCandidates.length - 1; i >= 0; i--) {
    const cost = tailCandidates[i].text.length + sepLen;
    if (tailLen + cost <= budget) {
      tailLen += cost;
      tailCount++;
    } else {
      break;
    }
  }

  const parts: string[] = [];
  for (const seg of head) {
    parts.push(seg.text);
  }
  parts.push(ellipsis);
  const tailStart = tailCandidates.length - tailCount;
  for (let i = tailStart; i < tailCandidates.length; i++) {
    parts.push(tailCandidates[i].text);
  }

  return info.reassemble(parts);
}

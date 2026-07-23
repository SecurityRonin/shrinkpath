import { separator } from '../platform';
import type { PathInfo } from '../path-info';
import { abbreviateSegment } from './fish';

export function shrinkHybridStrategy(
  info: PathInfo,
  maxLen: number,
  ellipsis: string,
  anchors: string[],
): string {
  if (info.segments.length === 0) {
    return info.reassemble([]);
  }

  const texts = info.segments.map(s => s.text);
  const full = info.reassemble(texts);
  if (full.length <= maxLen) {
    return full;
  }

  if (info.filename.length >= maxLen) {
    return info.filename;
  }

  const working = [...texts];
  const priorities = info.segments.map(s => s.priority);

  // Phase 1: Fish expendable segments
  for (let i = 0; i < priorities.length; i++) {
    if (priorities[i] === 'expendable') {
      working[i] = abbreviateSegment(info.segments[i].text, 1, anchors);
    }
  }
  let result = info.reassemble(working);
  if (result.length <= maxLen) return result;

  // Phase 2: Fish context segments
  for (let i = 0; i < priorities.length; i++) {
    if (priorities[i] === 'context') {
      working[i] = abbreviateSegment(info.segments[i].text, 1, anchors);
    }
  }
  result = info.reassemble(working);
  if (result.length <= maxLen) return result;

  // Phase 3: Collapse middle
  if (working.length > 2) {
    const collapsed = collapseMiddle(working, info, maxLen, ellipsis);
    if (collapsed.length <= maxLen) return collapsed;
  }

  // Phase 4: Fish identity segments
  for (let i = 0; i < priorities.length; i++) {
    if (priorities[i] === 'identity') {
      working[i] = abbreviateSegment(info.segments[i].text, 1, anchors);
    }
  }

  // Note: post-phase4 collapse attempt is omitted because collapseMiddle's
  // loop minimum (keepHead=0/keepTail=0) doesn't use working segments, so
  // identity fishing cannot improve it — if phase 3 failed, this would too.

  const sep = separator(info.style);
  const prefixSep = info.prefix !== '' ? sep : '';
  const minimal = `${info.prefix}${prefixSep}${ellipsis}${sep}${info.filename}`;
  if (minimal.length <= maxLen) return minimal;

  return info.filename;
}

function collapseMiddle(
  working: string[],
  info: PathInfo,
  maxLen: number,
  ellipsis: string,
): string {
  const n = working.length;

  for (let keepTail = Math.min(n, 3); keepTail >= 0; keepTail--) {
    for (let keepHead = Math.min(n, 3); keepHead >= 0; keepHead--) {
      if (keepHead + keepTail >= n) continue;

      const parts: string[] = [];
      for (let i = 0; i < keepHead; i++) {
        parts.push(working[i]);
      }
      parts.push(ellipsis);
      for (let i = n - keepTail; i < n; i++) {
        parts.push(working[i]);
      }

      const result = info.reassemble(parts);
      if (result.length <= maxLen) return result;
    }
  }

  const sep = separator(info.style);
  const prefixSep = info.prefix !== '' ? sep : '';
  return `${info.prefix}${prefixSep}${ellipsis}${sep}${info.filename}`;
}

import { PathInfo } from './path-info';
import { shrinkFishStrategy } from './strategy/fish';
import { shrinkEllipsisStrategy } from './strategy/ellipsis';
import { shrinkHybridStrategy } from './strategy/hybrid';
import { shrinkUniqueStrategy } from './strategy/unique';
import type { PathStyle } from './platform';

export type Strategy = 'hybrid' | 'fish' | 'ellipsis' | 'unique';

export type { PathStyle } from './platform';

export interface ShrinkOptions {
  maxLen: number;
  strategy?: Strategy;
  pathStyle?: PathStyle;
  ellipsis?: string;
  dirLength?: number;
  fullLengthDirs?: number;
  mappedLocations?: [string, string][];
  anchors?: string[];
}

export interface SegmentInfo {
  original: string;
  shortened: string;
  wasAbbreviated: boolean;
  isFilename: boolean;
}

export interface ShrinkResult {
  shortened: string;
  originalLen: number;
  shortenedLen: number;
  wasTruncated: boolean;
  detectedStyle: PathStyle;
  segments: SegmentInfo[];
}

function applyMappedLocations(
  path: string,
  mappedLocations: [string, string][],
): string {
  if (mappedLocations.length === 0) return path;

  const sorted = [...mappedLocations].sort((a, b) => b[0].length - a[0].length);

  for (const [from, to] of sorted) {
    if (path.startsWith(from)) {
      return to + path.slice(from.length);
    }
  }

  return path;
}

function buildSegmentMetadata(
  original: PathInfo,
  shortened: PathInfo,
): SegmentInfo[] {
  const result: SegmentInfo[] = [];

  const origSegCount = original.segments.length;
  const shortSegCount = shortened.segments.length;

  if (origSegCount === shortSegCount) {
    for (let i = 0; i < origSegCount; i++) {
      result.push({
        original: original.segments[i].text,
        shortened: shortened.segments[i].text,
        wasAbbreviated: original.segments[i].text !== shortened.segments[i].text,
        isFilename: false,
      });
    }
  } else {
    const shortTexts = shortened.segments.map(s => s.text);
    const ellipsisPos = shortTexts.findIndex(
      t => t.includes('...') || t.includes('..'),
    );

    if (ellipsisPos !== -1) {
      for (let i = 0; i < ellipsisPos; i++) {
        if (i < origSegCount) {
          result.push({
            original: original.segments[i].text,
            shortened: shortened.segments[i].text,
            wasAbbreviated: original.segments[i].text !== shortened.segments[i].text,
            isFilename: false,
          });
        }
      }

      const tailCount = shortSegCount - ellipsisPos - 1;
      const collapsedStart = ellipsisPos;
      const collapsedEnd = Math.max(0, origSegCount - tailCount);
      for (let i = collapsedStart; i < collapsedEnd; i++) {
        result.push({
          original: original.segments[i].text,
          shortened: '...',
          wasAbbreviated: true,
          isFilename: false,
        });
      }

      for (let i = ellipsisPos + 1; i < shortSegCount; i++) {
        const origIdx = origSegCount - (shortSegCount - i);
        if (origIdx >= 0 && origIdx < origSegCount) {
          result.push({
            original: original.segments[origIdx].text,
            shortened: shortened.segments[i].text,
            wasAbbreviated:
              original.segments[origIdx].text !== shortened.segments[i].text,
            isFilename: false,
          });
        }
      }
    } else {
      for (const seg of shortened.segments) {
        result.push({
          original: seg.text,
          shortened: seg.text,
          wasAbbreviated: false,
          isFilename: false,
        });
      }
    }
  }

  if (original.filename !== '') {
    result.push({
      original: original.filename,
      shortened: shortened.filename,
      wasAbbreviated: original.filename !== shortened.filename,
      isFilename: true,
    });
  }

  return result;
}

export function shrink(path: string, opts: ShrinkOptions): string {
  if (path === '') return '';

  const strategy = opts.strategy ?? 'hybrid';
  const ellipsis = opts.ellipsis ?? '...';
  const dirLength = opts.dirLength ?? 1;
  const fullLengthDirs = opts.fullLengthDirs ?? 0;
  const anchors = opts.anchors ?? [];
  const mappedLocations = opts.mappedLocations ?? [];

  const mapped = applyMappedLocations(path, mappedLocations);
  const info = PathInfo.parse(mapped, opts.pathStyle);

  switch (strategy) {
    case 'fish':
      return shrinkFishStrategy(info, dirLength, fullLengthDirs, anchors);
    case 'ellipsis':
      return shrinkEllipsisStrategy(info, opts.maxLen, ellipsis);
    case 'hybrid':
      return shrinkHybridStrategy(info, opts.maxLen, ellipsis, anchors);
    case 'unique':
      return shrinkUniqueStrategy(info, anchors);
  }
}

export function shrinkDetailed(path: string, opts: ShrinkOptions): ShrinkResult {
  if (path === '') {
    return {
      shortened: '',
      originalLen: 0,
      shortenedLen: 0,
      wasTruncated: false,
      detectedStyle: 'unix',
      segments: [],
    };
  }

  const mappedLocations = opts.mappedLocations ?? [];
  const mappedPath = applyMappedLocations(path, mappedLocations);
  const originalInfo = PathInfo.parse(mappedPath, opts.pathStyle);

  const shortened = shrink(path, opts);
  const shortenedInfo = PathInfo.parse(shortened, opts.pathStyle);

  const segments = buildSegmentMetadata(originalInfo, shortenedInfo);

  return {
    shortened,
    originalLen: path.length,
    shortenedLen: shortened.length,
    wasTruncated: shortened !== path,
    detectedStyle: originalInfo.style,
    segments,
  };
}

export function shrinkTo(path: string, maxLen: number): string {
  return shrink(path, { maxLen });
}

export function shrinkFish(path: string): string {
  const info = PathInfo.parse(path);
  return shrinkFishStrategy(info, 1, 0, []);
}

export function shrinkEllipsis(path: string, maxLen: number): string {
  return shrink(path, { maxLen, strategy: 'ellipsis' });
}

export function shrinkUnique(path: string): string {
  return shrink(path, { maxLen: Infinity, strategy: 'unique' });
}

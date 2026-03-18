export type PathStyle = 'unix' | 'windows';

export function detectStyle(path: string): PathStyle {
  if (path.startsWith('\\\\')) {
    return 'windows';
  }

  if (
    path.length >= 3 &&
    /^[a-zA-Z]$/.test(path[0]) &&
    path[1] === ':' &&
    (path[2] === '\\' || path[2] === '/')
  ) {
    return 'windows';
  }

  if (path.startsWith('.\\')) {
    return 'windows';
  }

  if (path.includes('\\') && !path.includes('/')) {
    return 'windows';
  }

  return 'unix';
}

export const UNIX_HOME_ROOTS: readonly string[] = ['home', 'Users'];
export const WIN_HOME_ROOTS: readonly string[] = ['Users'];

export function separator(style: PathStyle): string {
  return style === 'unix' ? '/' : '\\';
}

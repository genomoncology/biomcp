/**
 * Append a relative path to a base URL.
 *
 * Normalizes both trailing slash on base and leading slash on path to ensure
 * the path is always appended rather than replacing the base path.
 *
 * @example
 * appendPath('https://api.example.com/v1', 'users')
 * // → URL for 'https://api.example.com/v1/users'
 *
 * @example
 * appendPath('https://api.example.com/v1', '/users')
 * // → URL for 'https://api.example.com/v1/users' (leading slash stripped)
 *
 * @remarks
 * Empty path returns the base URL with a trailing slash:
 * `appendPath('https://example.com/v1', '')` → `'https://example.com/v1/'`
 */
export function appendPath(base: string, path: string): URL {
  const normalizedBase = base.endsWith('/') ? base : `${base}/`;
  const normalizedPath = path.startsWith('/') ? path.slice(1) : path;
  return new URL(normalizedPath, normalizedBase);
}

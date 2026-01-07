/**
 * Shared helpers for fetchMock from cloudflare:test.
 */

/**
 * Convert undici's header format to a Headers object.
 * Undici may return headers as Headers or Record<string, string>.
 */
export function toHeaders(input: Headers | Record<string, string>): Headers {
  if (input instanceof Headers) {
    return input;
  }
  const headers = new Headers();
  for (const [key, value] of Object.entries(input)) {
    headers.set(key, value);
  }
  return headers;
}

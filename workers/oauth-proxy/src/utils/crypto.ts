/**
 * Convert an ArrayBuffer to a base64url-encoded string.
 */
export function toBase64Url(buffer: ArrayBuffer): string {
  return btoa(String.fromCharCode(...new Uint8Array(buffer)))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=/g, '');
}

/**
 * Generate a cryptographically secure token for auth codes and refresh tokens.
 *
 * Uses UUID v4 which provides 122 bits of entropy. This is sufficient because:
 * - Tokens are hashed (SHA-256) before storage, not stored in plaintext
 * - Tokens have TTLs (auth codes: 10min, refresh tokens: 30 days)
 * - Refresh tokens are single-use (rotated on each use)
 * - 122 bits requires ~10^24 years to brute-force at 10^12 guesses/sec
 *
 * If stronger entropy is ever needed, we can swap the implementation here.
 */
export function generateSecureToken(): string {
  return crypto.randomUUID();
}

/**
 * Compute SHA-256 hash of input string and return as base64url.
 * Returns full 43-character hash (256 bits / 43 bytes ASCII).
 *
 * Note: Cloudflare KV keys have a 512-byte limit. Since this function
 * returns a fixed 43-byte output, it's safe for use as KV keys even
 * with prefixes (e.g., "token:{hash}" = ~49 bytes).
 */
export async function sha256Base64Url(input: string): Promise<string> {
  const encoder = new TextEncoder();
  const hash = await crypto.subtle.digest('SHA-256', encoder.encode(input));
  return toBase64Url(hash);
}

/**
 * Constant-time string comparison to prevent timing attacks.
 *
 * Uses the platform's built-in crypto.subtle.timingSafeEqual for optimal
 * constant-time guarantees. This prevents attackers from using timing
 * differences to deduce secret values byte-by-byte.
 *
 * Note: While not strictly necessary for PKCE (single-use codes, hash comparison),
 * constant-time comparison is good security hygiene for any credential validation.
 */
export function timingSafeEqual(a: string, b: string): boolean {
  const encoder = new TextEncoder();
  const aBytes = encoder.encode(a);
  const bBytes = encoder.encode(b);

  // The built-in requires equal-length buffers
  if (aBytes.length !== bBytes.length) {
    return false;
  }

  return crypto.subtle.timingSafeEqual(aBytes, bBytes);
}

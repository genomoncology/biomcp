/**
 * Crypto Utility Tests
 *
 * Tests for cryptographic utility functions including constant-time comparison.
 */

import { describe, it, expect } from 'vitest';
import { timingSafeEqual, sha256Base64Url, toBase64Url, generateSecureToken } from '../../src/utils/crypto';

describe('timingSafeEqual', () => {
  it('returns true for equal strings', () => {
    expect(timingSafeEqual('hello', 'hello')).toBe(true);
    expect(timingSafeEqual('test123', 'test123')).toBe(true);
  });

  it('returns false for different strings of the same length', () => {
    expect(timingSafeEqual('hello', 'world')).toBe(false);
    expect(timingSafeEqual('aaaaa', 'aaaab')).toBe(false);
    expect(timingSafeEqual('test1', 'test2')).toBe(false);
  });

  it('returns false for strings of different lengths', () => {
    expect(timingSafeEqual('short', 'longer')).toBe(false);
    expect(timingSafeEqual('abc', 'abcd')).toBe(false);
    expect(timingSafeEqual('', 'nonempty')).toBe(false);
  });

  it('handles empty strings', () => {
    expect(timingSafeEqual('', '')).toBe(true);
    expect(timingSafeEqual('', 'a')).toBe(false);
    expect(timingSafeEqual('a', '')).toBe(false);
  });

  it('handles unicode strings', () => {
    expect(timingSafeEqual('héllo', 'héllo')).toBe(true);
    expect(timingSafeEqual('héllo', 'hello')).toBe(false);
    expect(timingSafeEqual('🔐', '🔐')).toBe(true);
    expect(timingSafeEqual('🔐', '🔑')).toBe(false);
  });

  it('works with base64url-encoded hashes (PKCE use case)', async () => {
    // Simulate PKCE verification: comparing computed challenge against stored challenge
    const codeVerifier = 'dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk';
    const storedChallenge = 'E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM';

    const computedChallenge = await sha256Base64Url(codeVerifier);

    expect(timingSafeEqual(computedChallenge, storedChallenge)).toBe(true);
    expect(timingSafeEqual(computedChallenge, 'wrong-challenge-value-here-xxxxx')).toBe(false);
  });
});

describe('sha256Base64Url', () => {
  it('produces consistent output for same input', async () => {
    const input = 'test-string';
    const hash1 = await sha256Base64Url(input);
    const hash2 = await sha256Base64Url(input);
    expect(hash1).toBe(hash2);
  });

  it('produces 43-character base64url output', async () => {
    const hash = await sha256Base64Url('any input');
    expect(hash).toHaveLength(43);
  });

  it('produces valid base64url characters only', async () => {
    const hash = await sha256Base64Url('test');
    expect(hash).toMatch(/^[A-Za-z0-9_-]+$/);
  });

  it('produces known output for RFC 7636 test vector', async () => {
    // Test vector from RFC 7636 Appendix B
    const codeVerifier = 'dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk';
    const expectedChallenge = 'E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM';
    const hash = await sha256Base64Url(codeVerifier);
    expect(hash).toBe(expectedChallenge);
  });
});

describe('toBase64Url', () => {
  it('converts empty buffer', () => {
    const buffer = new ArrayBuffer(0);
    expect(toBase64Url(buffer)).toBe('');
  });

  it('produces url-safe characters (no +, /, or =)', () => {
    // Create a buffer that would produce + and / in standard base64
    const buffer = new Uint8Array([251, 239, 254]).buffer;
    const result = toBase64Url(buffer);
    expect(result).not.toContain('+');
    expect(result).not.toContain('/');
    expect(result).not.toContain('=');
  });
});

describe('generateSecureToken', () => {
  it('generates valid UUID format', () => {
    const token = generateSecureToken();
    expect(token).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/);
  });

  it('generates unique tokens', () => {
    const tokens = new Set<string>();
    for (let i = 0; i < 100; i++) {
      tokens.add(generateSecureToken());
    }
    expect(tokens.size).toBe(100);
  });
});

/**
 * Environment Parsing Tests
 *
 * Tests for parseEnv function, focusing on REMOTE_MCP_AUTH_TOKEN validation
 * and HTTPS enforcement.
 */

import { describe, expect, it } from 'vitest';
import { env } from 'cloudflare:test';
import { parseTestEnv, createTestEnv } from './helpers/env';
import { parseEnv } from '../src/env';

/**
 * Smoke test to verify .dev.vars.test is loaded correctly.
 * If this fails, check that vitest.config.mts has `environment: 'test'`.
 */
describe('test environment loading', () => {
  it('should load .dev.vars.test values via cloudflare:test', () => {
    // These values must match .dev.vars.test exactly
    expect(env.JWT_SECRET).toBe('test-jwt-secret-for-ci-minimum-32-chars-long');
    expect(env.IDENTITY_PROVIDER).toBe('disabled');
  });

  it('should parse loaded env values successfully', () => {
    const parsed = parseTestEnv();
    expect(parsed.JWT_SECRET).toBe('test-jwt-secret-for-ci-minimum-32-chars-long');
    expect(parsed.IDENTITY_PROVIDER).toBe('disabled');
  });
});

describe('parseEnv', () => {
  describe('IDENTITY_PROVIDER validation', () => {
    it('should throw when not set', () => {
      const env = createTestEnv({ IDENTITY_PROVIDER: undefined });
      expect(() => parseEnv(env)).toThrow('IDENTITY_PROVIDER is required');
    });

    it('should throw with helpful message when not set', () => {
      const env = createTestEnv({ IDENTITY_PROVIDER: undefined });
      expect(() => parseEnv(env)).toThrow('Set to "disabled" to run without identity features');
    });

    it('should accept stytch provider', () => {
      const parsed = parseTestEnv({ IDENTITY_PROVIDER: 'stytch' });
      expect(parsed.IDENTITY_PROVIDER).toBe('stytch');
    });

    it('should accept oauth provider', () => {
      const parsed = parseTestEnv({ IDENTITY_PROVIDER: 'oauth' });
      expect(parsed.IDENTITY_PROVIDER).toBe('oauth');
    });

    it('should accept disabled provider', () => {
      const parsed = parseTestEnv({ IDENTITY_PROVIDER: 'disabled' });
      expect(parsed.IDENTITY_PROVIDER).toBe('disabled');
    });

    it('should reject invalid provider', () => {
      const env = createTestEnv({ IDENTITY_PROVIDER: 'invalid' as any });
      expect(() => parseEnv(env)).toThrow();
    });
  });

  describe('ANALYTICS_PROVIDER validation', () => {
    it('should default to cloudflare when not set', () => {
      const parsed = parseTestEnv({ ANALYTICS_PROVIDER: undefined });
      expect(parsed.ANALYTICS_PROVIDER).toBe('cloudflare');
    });

    it('should accept cloudflare provider', () => {
      const parsed = parseTestEnv({ ANALYTICS_PROVIDER: 'cloudflare' });
      expect(parsed.ANALYTICS_PROVIDER).toBe('cloudflare');
    });

    it('should accept bigquery provider', () => {
      const parsed = parseTestEnv({ ANALYTICS_PROVIDER: 'bigquery' });
      expect(parsed.ANALYTICS_PROVIDER).toBe('bigquery');
    });

    it('should accept none provider', () => {
      const parsed = parseTestEnv({ ANALYTICS_PROVIDER: 'none' });
      expect(parsed.ANALYTICS_PROVIDER).toBe('none');
    });

    it('should reject invalid provider', () => {
      const env = createTestEnv({ ANALYTICS_PROVIDER: 'invalid' as any });
      expect(() => parseEnv(env)).toThrow();
    });
  });

  describe('REMOTE_MCP_AUTH_TOKEN validation', () => {
    it('should accept valid token', () => {
      const parsed = parseTestEnv({
        REMOTE_MCP_AUTH_TOKEN: 'valid-token-that-is-at-least-32-chars-long',
      });

      expect(parsed.REMOTE_MCP_AUTH_TOKEN).toBe('valid-token-that-is-at-least-32-chars-long');
    });

    it('should treat empty string as undefined', () => {
      const parsed = parseTestEnv({
        REMOTE_MCP_AUTH_TOKEN: '',
      });

      expect(parsed.REMOTE_MCP_AUTH_TOKEN).toBeUndefined();
    });

    it('should treat whitespace-only string as undefined', () => {
      const parsed = parseTestEnv({
        REMOTE_MCP_AUTH_TOKEN: '   ',
      });

      expect(parsed.REMOTE_MCP_AUTH_TOKEN).toBeUndefined();
    });

    it('should throw for token with leading whitespace', () => {
      const env = createTestEnv({
        REMOTE_MCP_AUTH_TOKEN: '  token-with-leading-space',
      });

      expect(() => parseEnv(env)).toThrow('REMOTE_MCP_AUTH_TOKEN contains leading/trailing whitespace');
    });

    it('should throw for token with trailing whitespace', () => {
      const env = createTestEnv({
        REMOTE_MCP_AUTH_TOKEN: 'token-with-trailing-space  ',
      });

      expect(() => parseEnv(env)).toThrow('REMOTE_MCP_AUTH_TOKEN contains leading/trailing whitespace');
    });

    it('should throw for tokens shorter than 32 characters', () => {
      const env = createTestEnv({
        REMOTE_MCP_AUTH_TOKEN: 'short-token-only-25-chars', // 25 chars
      });

      expect(() => parseEnv(env)).toThrow('REMOTE_MCP_AUTH_TOKEN must be at least 32 characters long');
    });

    it('should accept tokens with exactly 32 characters', () => {
      const parsed = parseTestEnv({
        REMOTE_MCP_AUTH_TOKEN: '12345678901234567890123456789012', // exactly 32 chars
      });

      expect(parsed.REMOTE_MCP_AUTH_TOKEN).toBe('12345678901234567890123456789012');
    });

    it('should allow undefined token', () => {
      // Note: vitest.config.mts sets REMOTE_MCP_AUTH_TOKEN in miniflare bindings,
      // so we must explicitly override to undefined to test this case
      const parsed = parseTestEnv({
        REMOTE_MCP_AUTH_TOKEN: undefined,
      });
      expect(parsed.REMOTE_MCP_AUTH_TOKEN).toBeUndefined();
    });
  });

  describe('OAUTH_ISSUER validation', () => {
    const oauthBaseConfig = {
      IDENTITY_PROVIDER: 'oauth' as const,
      OAUTH_CLIENT_ID: 'test-client-id',
      OAUTH_CLIENT_SECRET: 'test-client-secret',
      OAUTH_AUTHORIZATION_URL: 'https://auth.example.com/authorize',
      OAUTH_TOKEN_URL: 'https://auth.example.com/token',
    };

    it('should accept OAUTH_ISSUER without trailing slash', () => {
      const parsed = parseTestEnv({
        ...oauthBaseConfig,
        OAUTH_ISSUER: 'https://auth.example.com',
      });
      expect(parsed.OAUTH_ISSUER).toBe('https://auth.example.com');
    });

    it('should reject OAUTH_ISSUER with trailing slash', () => {
      const env = createTestEnv({
        ...oauthBaseConfig,
        OAUTH_ISSUER: 'https://auth.example.com/',
      });
      expect(() => parseEnv(env)).toThrow('OAUTH_ISSUER must not end with a trailing slash');
    });

    it('should accept OAUTH_ISSUER with path but no trailing slash', () => {
      const parsed = parseTestEnv({
        ...oauthBaseConfig,
        OAUTH_ISSUER: 'https://auth.example.com/realms/myrealm',
      });
      expect(parsed.OAUTH_ISSUER).toBe('https://auth.example.com/realms/myrealm');
    });

    it('should reject OAUTH_ISSUER with path and trailing slash', () => {
      const env = createTestEnv({
        ...oauthBaseConfig,
        OAUTH_ISSUER: 'https://auth.example.com/realms/myrealm/',
      });
      expect(() => parseEnv(env)).toThrow('OAUTH_ISSUER must not end with a trailing slash');
    });
  });

  describe('HTTPS enforcement', () => {
    const validToken = 'my-secret-token-that-is-32-chars!';

    it('should throw when using HTTP with auth token', () => {
      const env = createTestEnv({
        REMOTE_MCP_SERVER_URL: 'http://api.example.com',
        REMOTE_MCP_AUTH_TOKEN: validToken,
      });

      expect(() => parseEnv(env)).toThrow('REMOTE_MCP_AUTH_TOKEN requires HTTPS for REMOTE_MCP_SERVER_URL to prevent token leakage');
    });

    it('should allow HTTPS with auth token', () => {
      const parsed = parseTestEnv({
        REMOTE_MCP_SERVER_URL: 'https://api.example.com',
        REMOTE_MCP_AUTH_TOKEN: validToken,
      });

      expect(parsed.REMOTE_MCP_AUTH_TOKEN).toBe(validToken);
    });

    it('should allow http://localhost with auth token (local dev)', () => {
      const parsed = parseTestEnv({
        REMOTE_MCP_SERVER_URL: 'http://localhost:8000',
        REMOTE_MCP_AUTH_TOKEN: validToken,
      });

      expect(parsed.REMOTE_MCP_AUTH_TOKEN).toBe(validToken);
    });

    it('should allow http://127.0.0.1 with auth token (local dev)', () => {
      const parsed = parseTestEnv({
        REMOTE_MCP_SERVER_URL: 'http://127.0.0.1:8000',
        REMOTE_MCP_AUTH_TOKEN: validToken,
      });

      expect(parsed.REMOTE_MCP_AUTH_TOKEN).toBe(validToken);
    });

    it('should allow HTTP without auth token', () => {
      const parsed = parseTestEnv({
        REMOTE_MCP_SERVER_URL: 'http://api.example.com',
        REMOTE_MCP_AUTH_TOKEN: undefined,
      });

      expect(parsed.REMOTE_MCP_AUTH_TOKEN).toBeUndefined();
    });
  });
});

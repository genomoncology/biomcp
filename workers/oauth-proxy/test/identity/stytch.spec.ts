/**
 * StytchProvider Unit Tests
 *
 * Tests for the Stytch identity provider implementation.
 * Uses fetchMock from cloudflare:test for mocking outbound fetch.
 */

import { fetchMock } from 'cloudflare:test';
import { describe, it, expect, beforeAll, beforeEach, afterEach } from 'vitest';
import type { StytchEnv } from '../../src/env';
import { StytchProvider } from '../../src/identity/providers/stytch';
import { createLogger, type Logger } from '../../src/utils/logger';

// Test environment
const testEnv: StytchEnv = {
  STYTCH_PROJECT_ID: 'project-test-123',
  STYTCH_SECRET: 'secret-test-456',
  STYTCH_PUBLIC_TOKEN: 'public-token-test-789',
  STYTCH_OAUTH_URL: 'https://test.stytch.com/v1/public/oauth/google/start',
  STYTCH_API_URL: 'https://test.stytch.com/v1',
};

/**
 * Captured request details from fetchMock.
 */
interface CapturedRequest {
  url: string;
  method: string;
  headers: Record<string, string>;
  body?: string;
}

describe('StytchProvider', () => {
  let provider: StytchProvider;
  let logger: Logger;
  let capturedRequest: CapturedRequest | null = null;

  beforeAll(() => {
    fetchMock.activate();
    fetchMock.disableNetConnect();
  });

  beforeEach(() => {
    logger = createLogger(false, 'test');
    provider = new StytchProvider(testEnv, logger);
    capturedRequest = null;
  });

  afterEach(() => {
    fetchMock.assertNoPendingInterceptors();
  });

  // Helper to set up Stytch API mock
  const setupStytchMock = (response: { ok: boolean; data?: object; errorText?: string }) => {
    fetchMock
      .get('https://test.stytch.com')
      .intercept({
        method: 'POST',
        path: '/v1/oauth/authenticate',
      })
      .reply((opts) => {
        capturedRequest = {
          url: `https://test.stytch.com${opts.path}`,
          method: opts.method,
          headers: opts.headers as Record<string, string>,
          body: opts.body?.toString(),
        };
        if (response.ok) {
          return {
            statusCode: 200,
            data: JSON.stringify(response.data),
            responseOptions: { headers: { 'content-type': 'application/json' } },
          };
        }
        return {
          statusCode: 401,
          data: response.errorText || 'Unauthorized',
        };
      });
  };

  describe('buildAuthorizationUrl', () => {
    it('embeds tx in login_redirect_url query param', async () => {
      const url = await provider.buildAuthorizationUrl({
        callbackUrl: 'https://gateway.example.com/callback',
        tx: 'test-tx-123',
      });

      // Should be Stytch OAuth URL
      expect(url.origin).toBe('https://test.stytch.com');
      expect(url.pathname).toBe('/v1/public/oauth/google/start');

      // Should have public_token
      expect(url.searchParams.get('public_token')).toBe('public-token-test-789');

      // Should have login_redirect_url with tx embedded
      const loginRedirectUrl = url.searchParams.get('login_redirect_url');
      expect(loginRedirectUrl).toBeDefined();

      const redirectUrl = new URL(loginRedirectUrl!);
      expect(redirectUrl.origin).toBe('https://gateway.example.com');
      expect(redirectUrl.pathname).toBe('/callback');
      expect(redirectUrl.searchParams.get('tx')).toBe('test-tx-123');
    });

    it('preserves existing query params in callback URL', async () => {
      const url = await provider.buildAuthorizationUrl({
        callbackUrl: 'https://gateway.example.com/callback?existing=param',
        tx: 'test-tx-456',
      });

      const loginRedirectUrl = new URL(url.searchParams.get('login_redirect_url')!);
      expect(loginRedirectUrl.searchParams.get('existing')).toBe('param');
      expect(loginRedirectUrl.searchParams.get('tx')).toBe('test-tx-456');
    });

    it('ignores scopes (Stytch handles scopes internally)', async () => {
      const url = await provider.buildAuthorizationUrl({
        callbackUrl: 'https://gateway.example.com/callback',
        tx: 'test-tx-789',
        scopes: ['openid', 'email', 'profile'],
      });

      // Scopes should not appear in URL (Stytch manages OAuth scopes internally)
      expect(url.searchParams.get('scope')).toBeNull();
    });
  });

  describe('parseCallback', () => {
    it('extracts token and tx from stytch_token_type=oauth callback', () => {
      const url = new URL('https://gateway.example.com/callback?stytch_token_type=oauth&token=oauth-token-123&tx=test-tx-abc');

      const result = provider.parseCallback(url);

      expect(result).not.toBeNull();
      expect(result!.token).toBe('oauth-token-123');
      expect(result!.tokenType).toBe('oauth');
      expect(result!.tx).toBe('test-tx-abc');
    });

    it('extracts token from stytch_token param when no token_type', () => {
      const url = new URL('https://gateway.example.com/callback?stytch_token=stytch-token-456&tx=test-tx-def');

      const result = provider.parseCallback(url);

      expect(result).not.toBeNull();
      expect(result!.token).toBe('stytch-token-456');
      expect(result!.tokenType).toBeUndefined();
      expect(result!.tx).toBe('test-tx-def');
    });

    it('extracts token from token param when no stytch_token', () => {
      const url = new URL('https://gateway.example.com/callback?token=generic-token-789&tx=test-tx-ghi');

      const result = provider.parseCallback(url);

      expect(result).not.toBeNull();
      expect(result!.token).toBe('generic-token-789');
      expect(result!.tx).toBe('test-tx-ghi');
    });

    it('returns null when token is missing', () => {
      const url = new URL('https://gateway.example.com/callback?tx=test-tx-jkl');

      const result = provider.parseCallback(url);

      expect(result).toBeNull();
    });

    it('returns null when tx is missing', () => {
      const url = new URL('https://gateway.example.com/callback?token=some-token');

      const result = provider.parseCallback(url);

      expect(result).toBeNull();
    });

    it('returns null for empty callback URL', () => {
      const url = new URL('https://gateway.example.com/callback');

      const result = provider.parseCallback(url);

      expect(result).toBeNull();
    });

    it('parses Stytch error response with stytch_error', () => {
      const url = new URL(
        'https://gateway.example.com/callback?stytch_error=access_denied&stytch_error_description=User%20cancelled&tx=test-tx',
      );

      const result = provider.parseCallback(url);

      expect(result).not.toBeNull();
      expect(result!.tx).toBe('test-tx');
      expect(result!.token).toBeUndefined();
      expect(result!.idpError).toBeDefined();
      expect(result!.idpError!.error).toBe('access_denied');
      expect(result!.idpError!.error_description).toBe('User cancelled');
    });

    it('parses standard OAuth error params from Stytch', () => {
      // Stytch may also use standard OAuth error params
      const url = new URL('https://gateway.example.com/callback?error=server_error&error_description=Internal%20error&tx=test-tx');

      const result = provider.parseCallback(url);

      expect(result).not.toBeNull();
      expect(result!.tx).toBe('test-tx');
      expect(result!.idpError).toBeDefined();
      expect(result!.idpError!.error).toBe('server_error');
      expect(result!.idpError!.error_description).toBe('Internal error');
    });

    it('prefers stytch_error over standard error param', () => {
      const url = new URL('https://gateway.example.com/callback?stytch_error=stytch_specific&error=generic&tx=test-tx');

      const result = provider.parseCallback(url);

      expect(result).not.toBeNull();
      expect(result!.idpError!.error).toBe('stytch_specific');
    });

    it('returns null for error without tx', () => {
      const url = new URL('https://gateway.example.com/callback?stytch_error=access_denied');

      const result = provider.parseCallback(url);

      expect(result).toBeNull();
    });

    it('treats callback with both error and token as error (prioritizes error)', () => {
      // RFC 6749 violation, but we defensively handle it by prioritizing error
      const url = new URL('https://gateway.example.com/callback?error=access_denied&token=oauth-token-123&tx=test-tx');

      const result = provider.parseCallback(url);

      expect(result).not.toBeNull();
      expect(result!.idpError).toBeDefined();
      expect(result!.idpError!.error).toBe('access_denied');
      expect(result!.token).toBeUndefined();
    });
  });

  describe('authenticate', () => {
    it('calls Stytch API with correct headers and body', async () => {
      setupStytchMock({
        ok: true,
        data: {
          user_id: 'user-123',
          user: {
            emails: [{ email: 'test@example.com' }],
          },
        },
      });

      await provider.authenticate({
        token: 'test-token',
        redirectUri: 'https://gateway.example.com/callback',
      });

      expect(capturedRequest).not.toBeNull();
      expect(capturedRequest!.url).toBe('https://test.stytch.com/v1/oauth/authenticate');
      expect(capturedRequest!.method).toBe('POST');
      expect(capturedRequest!.headers['content-type']).toBe('application/json');

      // Verify Basic auth header
      const expectedAuth = btoa('project-test-123:secret-test-456');
      expect(capturedRequest!.headers['authorization']).toBe(`Basic ${expectedAuth}`);

      // Verify body
      const body = JSON.parse(capturedRequest!.body!);
      expect(body.token).toBe('test-token');
    });

    it('returns authenticated user with sub and email', async () => {
      setupStytchMock({
        ok: true,
        data: {
          user_id: 'user-456',
          user: {
            emails: [{ email: 'user@example.com' }],
          },
        },
      });

      const result = await provider.authenticate({
        token: 'test-token',
        redirectUri: 'https://gateway.example.com/callback',
      });

      expect(result.sub).toBe('user-456');
      expect(result.email).toBe('user@example.com');
    });

    it('handles user without email', async () => {
      setupStytchMock({
        ok: true,
        data: {
          user_id: 'user-789',
          user: {},
        },
      });

      const result = await provider.authenticate({
        token: 'test-token',
        redirectUri: 'https://gateway.example.com/callback',
      });

      expect(result.sub).toBe('user-789');
      expect(result.email).toBeUndefined();
    });

    it('handles user with empty emails array', async () => {
      setupStytchMock({
        ok: true,
        data: {
          user_id: 'user-000',
          user: {
            emails: [],
          },
        },
      });

      const result = await provider.authenticate({
        token: 'test-token',
        redirectUri: 'https://gateway.example.com/callback',
      });

      expect(result.sub).toBe('user-000');
      expect(result.email).toBeUndefined();
    });

    it('throws on authentication failure', async () => {
      setupStytchMock({
        ok: false,
        errorText: 'Invalid token',
      });

      await expect(
        provider.authenticate({
          token: 'invalid-token',
          redirectUri: 'https://gateway.example.com/callback',
        }),
      ).rejects.toThrow('Authentication failed');
    });

    it('throws on invalid response structure (missing user_id)', async () => {
      setupStytchMock({
        ok: true,
        data: {
          // Missing required user_id field
          user: {
            emails: [{ email: 'test@example.com' }],
          },
        },
      });

      await expect(
        provider.authenticate({
          token: 'test-token',
          redirectUri: 'https://gateway.example.com/callback',
        }),
      ).rejects.toThrow('Invalid Stytch response: missing required fields');
    });

    it('throws on invalid response structure (wrong user_id type)', async () => {
      setupStytchMock({
        ok: true,
        data: {
          user_id: 12345, // Wrong type - should be string
          user: {},
        },
      });

      await expect(
        provider.authenticate({
          token: 'test-token',
          redirectUri: 'https://gateway.example.com/callback',
        }),
      ).rejects.toThrow('Invalid Stytch response: missing required fields');
    });

    it('does not require redirectUri for Stytch (accepts but ignores)', async () => {
      setupStytchMock({
        ok: true,
        data: {
          user_id: 'user-no-redirect',
          user: {},
        },
      });

      // Stytch doesn't need redirect_uri for token auth, but we pass it for interface consistency
      const result = await provider.authenticate({
        token: 'test-token',
        redirectUri: 'https://different.example.com/callback',
      });

      expect(result.sub).toBe('user-no-redirect');

      // Verify redirect_uri is NOT sent to Stytch
      const body = JSON.parse(capturedRequest!.body!);
      expect(body.redirect_uri).toBeUndefined();
    });
  });

  describe('name property', () => {
    it('returns "stytch"', () => {
      expect(provider.name).toBe('stytch');
    });
  });
});

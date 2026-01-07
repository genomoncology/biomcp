/**
 * StandardOAuthProvider Unit Tests
 *
 * Tests for the standard OAuth provider implementation.
 * Works with Auth0, Okta, Google, and other standard OAuth 2.0 providers.
 * Uses fetchMock from cloudflare:test for mocking outbound fetch.
 */

import { fetchMock } from 'cloudflare:test';
import { exportJWK, generateKeyPair, SignJWT } from 'jose';
import { describe, it, expect, beforeAll, beforeEach, afterEach } from 'vitest';
import type { StandardOAuthEnv } from '../../src/env';
import { StandardOAuthProvider } from '../../src/identity/providers/oauth';
import { createLogger, type Logger } from '../../src/utils/logger';

// Test environment
const testEnv: StandardOAuthEnv = {
  OAUTH_CLIENT_ID: 'test-client-id',
  OAUTH_CLIENT_SECRET: 'test-client-secret',
  OAUTH_AUTHORIZATION_URL: 'https://idp.example.com/authorize',
  OAUTH_TOKEN_URL: 'https://idp.example.com/oauth/token',
  OAUTH_USERINFO_URL: 'https://idp.example.com/userinfo',
  OAUTH_SCOPES: ['openid', 'email', 'profile'],
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

describe('StandardOAuthProvider', () => {
  let provider: StandardOAuthProvider;
  let logger: Logger;
  let capturedTokenRequest: CapturedRequest | null = null;
  let capturedUserInfoRequest: CapturedRequest | null = null;

  beforeAll(() => {
    fetchMock.activate();
    fetchMock.disableNetConnect();
  });

  beforeEach(() => {
    logger = createLogger(false, 'test');
    provider = new StandardOAuthProvider(testEnv, logger);
    capturedTokenRequest = null;
    capturedUserInfoRequest = null;
  });

  afterEach(() => {
    fetchMock.assertNoPendingInterceptors();
  });

  // Helper to set up OAuth token endpoint mock
  const setupTokenMock = (response: { ok: boolean; data?: object; errorText?: string }) => {
    fetchMock
      .get('https://idp.example.com')
      .intercept({
        method: 'POST',
        path: '/oauth/token',
      })
      .reply((opts) => {
        capturedTokenRequest = {
          url: `https://idp.example.com${opts.path}`,
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
          data: response.errorText || 'Invalid code',
        };
      });
  };

  // Helper to set up OAuth userinfo endpoint mock
  const setupUserInfoMock = (response: { ok: boolean; data?: object; errorText?: string }) => {
    fetchMock
      .get('https://idp.example.com')
      .intercept({
        method: 'GET',
        path: '/userinfo',
      })
      .reply((opts) => {
        capturedUserInfoRequest = {
          url: `https://idp.example.com${opts.path}`,
          method: opts.method,
          headers: opts.headers as Record<string, string>,
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

  // Helper to set up both token and userinfo mocks
  const setupFullAuthMock = (options: {
    tokenResponse: { ok: boolean; data?: object; errorText?: string };
    userInfoResponse?: { ok: boolean; data?: object; errorText?: string };
  }) => {
    setupTokenMock(options.tokenResponse);
    if (options.userInfoResponse) {
      setupUserInfoMock(options.userInfoResponse);
    }
  };

  describe('buildAuthorizationUrl', () => {
    it('places tx in state parameter (standard OAuth behavior)', async () => {
      const url = await provider.buildAuthorizationUrl({
        callbackUrl: 'https://gateway.example.com/callback',
        tx: 'test-tx-123',
      });

      expect(url.origin).toBe('https://idp.example.com');
      expect(url.pathname).toBe('/authorize');
      expect(url.searchParams.get('client_id')).toBe('test-client-id');
      expect(url.searchParams.get('redirect_uri')).toBe('https://gateway.example.com/callback');
      expect(url.searchParams.get('response_type')).toBe('code');
      expect(url.searchParams.get('state')).toBe('test-tx-123');
    });

    it('keeps redirect_uri clean (no tx embedded)', async () => {
      const url = await provider.buildAuthorizationUrl({
        callbackUrl: 'https://gateway.example.com/callback',
        tx: 'test-tx-456',
      });

      const redirectUri = url.searchParams.get('redirect_uri')!;
      // redirect_uri should be clean - no tx query param
      expect(redirectUri).toBe('https://gateway.example.com/callback');
      expect(redirectUri).not.toContain('tx=');
    });

    it('includes scopes when provided', async () => {
      const url = await provider.buildAuthorizationUrl({
        callbackUrl: 'https://gateway.example.com/callback',
        tx: 'test-tx-789',
        scopes: ['openid', 'email', 'profile'],
      });

      expect(url.searchParams.get('scope')).toBe('openid email profile');
    });

    it('uses default OIDC scopes when no scopes provided', async () => {
      const url = await provider.buildAuthorizationUrl({
        callbackUrl: 'https://gateway.example.com/callback',
        tx: 'test-tx-abc',
      });

      // Defaults to standard OIDC scopes for user identity
      expect(url.searchParams.get('scope')).toBe('openid email profile');
    });

    it('uses default OIDC scopes for empty scopes array', async () => {
      const url = await provider.buildAuthorizationUrl({
        callbackUrl: 'https://gateway.example.com/callback',
        tx: 'test-tx-def',
        scopes: [],
      });

      // Empty array falls back to default scopes
      expect(url.searchParams.get('scope')).toBe('openid email profile');
    });
  });

  describe('parseCallback', () => {
    it('extracts code and tx from state parameter', () => {
      const url = new URL('https://gateway.example.com/callback?code=auth-code-123&state=test-tx-ghi');

      const result = provider.parseCallback(url);

      expect(result).not.toBeNull();
      expect(result!.token).toBe('auth-code-123');
      expect(result!.tx).toBe('test-tx-ghi');
      expect(result!.tokenType).toBeUndefined();
    });

    it('returns null when code is missing', () => {
      const url = new URL('https://gateway.example.com/callback?state=test-tx-jkl');

      const result = provider.parseCallback(url);

      expect(result).toBeNull();
    });

    it('returns null when state (tx) is missing', () => {
      const url = new URL('https://gateway.example.com/callback?code=some-code');

      const result = provider.parseCallback(url);

      expect(result).toBeNull();
    });

    it('returns null for empty callback URL', () => {
      const url = new URL('https://gateway.example.com/callback');

      const result = provider.parseCallback(url);

      expect(result).toBeNull();
    });

    it('parses IdP error response with error and description', () => {
      // RFC 6749 §4.1.2.1: IdP error responses include error, error_description, and state
      const url = new URL(
        'https://gateway.example.com/callback?error=access_denied&error_description=User%20denied%20access&state=test-tx',
      );

      const result = provider.parseCallback(url);

      expect(result).not.toBeNull();
      expect(result!.tx).toBe('test-tx');
      expect(result!.token).toBeUndefined();
      expect(result!.idpError).toBeDefined();
      expect(result!.idpError!.error).toBe('access_denied');
      expect(result!.idpError!.error_description).toBe('User denied access');
    });

    it('parses IdP error response without description', () => {
      const url = new URL('https://gateway.example.com/callback?error=server_error&state=test-tx');

      const result = provider.parseCallback(url);

      expect(result).not.toBeNull();
      expect(result!.tx).toBe('test-tx');
      expect(result!.idpError).toBeDefined();
      expect(result!.idpError!.error).toBe('server_error');
      expect(result!.idpError!.error_description).toBeUndefined();
    });

    it('returns null for IdP error without state', () => {
      // If there's no state (tx), we can't look up the redirect_uri
      const url = new URL('https://gateway.example.com/callback?error=access_denied');

      const result = provider.parseCallback(url);

      expect(result).toBeNull();
    });

    it('treats callback with both error and code as error (prioritizes error)', () => {
      // RFC 6749 violation, but we defensively handle it by prioritizing error
      const url = new URL('https://gateway.example.com/callback?error=access_denied&code=abc123&state=test-tx');

      const result = provider.parseCallback(url);

      expect(result).not.toBeNull();
      expect(result!.idpError).toBeDefined();
      expect(result!.idpError!.error).toBe('access_denied');
      expect(result!.token).toBeUndefined();
    });
  });

  describe('authenticate', () => {
    it('exchanges code for tokens with correct parameters', async () => {
      setupFullAuthMock({
        tokenResponse: {
          ok: true,
          data: {
            access_token: 'access-token-123',
            token_type: 'Bearer',
            expires_in: 3600,
          },
        },
        userInfoResponse: {
          ok: true,
          data: {
            sub: 'user-123',
            email: 'user@example.com',
          },
        },
      });

      await provider.authenticate({
        token: 'auth-code-456',
        redirectUri: 'https://gateway.example.com/callback',
      });

      // Verify token exchange request
      expect(capturedTokenRequest).not.toBeNull();
      expect(capturedTokenRequest!.url).toBe('https://idp.example.com/oauth/token');
      expect(capturedTokenRequest!.method).toBe('POST');
      expect(capturedTokenRequest!.headers['content-type']).toBe('application/x-www-form-urlencoded');

      const tokenBody = new URLSearchParams(capturedTokenRequest!.body!);
      expect(tokenBody.get('grant_type')).toBe('authorization_code');
      expect(tokenBody.get('code')).toBe('auth-code-456');
      expect(tokenBody.get('redirect_uri')).toBe('https://gateway.example.com/callback');
      expect(tokenBody.get('client_id')).toBe('test-client-id');
      expect(tokenBody.get('client_secret')).toBe('test-client-secret');
    });

    it('fetches user info with access token', async () => {
      setupFullAuthMock({
        tokenResponse: {
          ok: true,
          data: {
            access_token: 'access-token-789',
            token_type: 'Bearer',
          },
        },
        userInfoResponse: {
          ok: true,
          data: {
            sub: 'user-456',
            email: 'another@example.com',
          },
        },
      });

      await provider.authenticate({
        token: 'auth-code-xyz',
        redirectUri: 'https://gateway.example.com/callback',
      });

      // Verify userinfo request
      expect(capturedUserInfoRequest).not.toBeNull();
      expect(capturedUserInfoRequest!.url).toBe('https://idp.example.com/userinfo');
      expect(capturedUserInfoRequest!.headers['authorization']).toBe('Bearer access-token-789');
    });

    /**
     * RFC 6749 §4.1.3: The redirect_uri in the token request MUST be identical
     * to the redirect_uri used in the authorization request.
     *
     * For this gateway, that means the gateway's /callback URL (not the client's redirect_uri).
     * The callback handler reconstructs this from: new URL('/callback', url.origin)
     */
    it('sends gateway callback URL in token exchange, not client redirect_uri (RFC 6749 §4.1.3)', async () => {
      setupFullAuthMock({
        tokenResponse: {
          ok: true,
          data: {
            access_token: 'access-token-rfc',
            token_type: 'Bearer',
          },
        },
        userInfoResponse: {
          ok: true,
          data: {
            sub: 'user-rfc',
            email: 'rfc@example.com',
          },
        },
      });

      // The redirectUri here represents the gateway's callback URL,
      // which the callback handler constructs from new URL('/callback', url.origin).
      // This is distinct from the client's redirect_uri stored in oauthTxData.
      const gatewayCallbackUrl = 'https://gateway.example.com/callback';

      await provider.authenticate({
        token: 'auth-code-rfc',
        redirectUri: gatewayCallbackUrl,
      });

      // The redirect_uri sent to IdP must match what was used in /authorize
      const tokenBody = new URLSearchParams(capturedTokenRequest!.body!);
      expect(tokenBody.get('redirect_uri')).toBe(gatewayCallbackUrl);
    });

    it('returns authenticated user with sub and email', async () => {
      setupFullAuthMock({
        tokenResponse: {
          ok: true,
          data: {
            access_token: 'access-token-abc',
            token_type: 'Bearer',
            refresh_token: 'refresh-token-abc',
          },
        },
        userInfoResponse: {
          ok: true,
          data: {
            sub: 'user-789',
            email: 'test@example.com',
            name: 'Test User',
          },
        },
      });

      const result = await provider.authenticate({
        token: 'auth-code-def',
        redirectUri: 'https://gateway.example.com/callback',
      });

      expect(result.sub).toBe('user-789');
      expect(result.email).toBe('test@example.com');
    });

    it('throws on token exchange failure', async () => {
      setupTokenMock({
        ok: false,
        errorText: 'Invalid code',
      });

      await expect(
        provider.authenticate({
          token: 'invalid-code',
          redirectUri: 'https://gateway.example.com/callback',
        }),
      ).rejects.toThrow('Token exchange failed');
    });

    it('throws on userinfo fetch failure', async () => {
      setupFullAuthMock({
        tokenResponse: {
          ok: true,
          data: {
            access_token: 'access-token-valid',
            token_type: 'Bearer',
          },
        },
        userInfoResponse: {
          ok: false,
          errorText: 'Unauthorized',
        },
      });

      await expect(
        provider.authenticate({
          token: 'valid-code',
          redirectUri: 'https://gateway.example.com/callback',
        }),
      ).rejects.toThrow('Failed to fetch user info');
    });

    it('throws on invalid token response (missing access_token)', async () => {
      setupTokenMock({
        ok: true,
        data: {
          // Missing access_token
          token_type: 'Bearer',
        },
      });

      await expect(
        provider.authenticate({
          token: 'some-code',
          redirectUri: 'https://gateway.example.com/callback',
        }),
      ).rejects.toThrow('Invalid token response');
    });

    it('throws on invalid userinfo response (missing sub)', async () => {
      setupFullAuthMock({
        tokenResponse: {
          ok: true,
          data: {
            access_token: 'access-token-valid',
            token_type: 'Bearer',
          },
        },
        userInfoResponse: {
          ok: true,
          data: {
            // Missing sub
            email: 'user@example.com',
          },
        },
      });

      await expect(
        provider.authenticate({
          token: 'valid-code',
          redirectUri: 'https://gateway.example.com/callback',
        }),
      ).rejects.toThrow('Invalid userinfo response');
    });

    it('requires redirect_uri for token exchange (per OAuth spec)', async () => {
      setupFullAuthMock({
        tokenResponse: {
          ok: true,
          data: {
            access_token: 'access-token-redirect',
            token_type: 'Bearer',
          },
        },
        userInfoResponse: {
          ok: true,
          data: {
            sub: 'user-redirect',
            email: 'redirect@example.com',
          },
        },
      });

      await provider.authenticate({
        token: 'auth-code-redirect',
        redirectUri: 'https://gateway.example.com/callback',
      });

      const tokenBody = new URLSearchParams(capturedTokenRequest!.body!);
      expect(tokenBody.get('redirect_uri')).toBe('https://gateway.example.com/callback');
    });
  });

  describe('name property', () => {
    it('returns "oauth"', () => {
      expect(provider.name).toBe('oauth');
    });
  });

  describe('authenticate without userinfo URL', () => {
    let providerNoUserInfo: StandardOAuthProvider;

    beforeEach(() => {
      providerNoUserInfo = new StandardOAuthProvider(
        {
          ...testEnv,
          OAUTH_USERINFO_URL: undefined,
        },
        logger,
      );
    });

    it('uses sub from token response when available', async () => {
      setupTokenMock({
        ok: true,
        data: {
          access_token: 'access-token-sub',
          token_type: 'Bearer',
          sub: 'user-from-token',
          email: 'token@example.com',
        },
      });

      const result = await providerNoUserInfo.authenticate({
        token: 'auth-code-sub',
        redirectUri: 'https://gateway.example.com/callback',
      });

      expect(result.sub).toBe('user-from-token');
      expect(result.email).toBe('token@example.com');
    });

    it('throws when no sub in token response and no userinfo URL', async () => {
      setupTokenMock({
        ok: true,
        data: {
          access_token: 'access-token-nosub',
          token_type: 'Bearer',
          // No sub - userinfo URL required
        },
      });

      await expect(
        providerNoUserInfo.authenticate({
          token: 'auth-code-nosub',
          redirectUri: 'https://gateway.example.com/callback',
        }),
      ).rejects.toThrow('Cannot determine user identity: configure OAUTH_USERINFO_URL');
    });
  });

  describe('id_token extraction with OIDC discovery', () => {
    let providerWithIssuer: StandardOAuthProvider;

    // Generate RSA key pair for signing test JWTs
    let privateKey: CryptoKey;
    let publicJwk: object;

    beforeAll(async () => {
      const keyPair = await generateKeyPair('RS256');
      privateKey = keyPair.privateKey;
      const jwk = await exportJWK(keyPair.publicKey);
      publicJwk = { ...jwk, kid: 'test-key-id', use: 'sig', alg: 'RS256' };
    });

    beforeEach(() => {
      providerWithIssuer = new StandardOAuthProvider(
        {
          ...testEnv,
          OAUTH_USERINFO_URL: undefined,
          OAUTH_ISSUER: 'https://idp.example.com',
        },
        logger,
      );
    });

    // Helper to create a signed id_token
    const createSignedIdToken = async (claims: { sub: string; email?: string }) => {
      return new SignJWT(claims)
        .setProtectedHeader({ alg: 'RS256', kid: 'test-key-id' })
        .setIssuer('https://idp.example.com')
        .setAudience('test-client-id')
        .setIssuedAt()
        .setExpirationTime('1h')
        .sign(privateKey);
    };

    // Helper to mock OIDC discovery and JWKS endpoints
    const setupOidcMocks = () => {
      // Mock OIDC discovery endpoint
      fetchMock
        .get('https://idp.example.com')
        .intercept({
          method: 'GET',
          path: '/.well-known/openid-configuration',
        })
        .reply(() => ({
          statusCode: 200,
          data: JSON.stringify({
            issuer: 'https://idp.example.com',
            jwks_uri: 'https://idp.example.com/.well-known/jwks.json',
          }),
          responseOptions: { headers: { 'content-type': 'application/json' } },
        }));

      // Mock JWKS endpoint
      fetchMock
        .get('https://idp.example.com')
        .intercept({
          method: 'GET',
          path: '/.well-known/jwks.json',
        })
        .reply(() => ({
          statusCode: 200,
          data: JSON.stringify({ keys: [publicJwk] }),
          responseOptions: { headers: { 'content-type': 'application/json' } },
        }));
    };

    it('extracts sub from id_token with valid signature', async () => {
      const idToken = await createSignedIdToken({ sub: 'user-from-jwt', email: 'jwt@example.com' });

      setupTokenMock({
        ok: true,
        data: {
          access_token: 'access-token-jwt',
          token_type: 'Bearer',
          id_token: idToken,
          email: 'jwt@example.com',
        },
      });
      setupOidcMocks();

      const result = await providerWithIssuer.authenticate({
        token: 'auth-code-jwt',
        redirectUri: 'https://gateway.example.com/callback',
      });

      expect(result.sub).toBe('user-from-jwt');
      expect(result.email).toBe('jwt@example.com');
    });

    it('ignores id_token when OAUTH_ISSUER is not configured', async () => {
      const providerNoIssuer = new StandardOAuthProvider(
        {
          ...testEnv,
          OAUTH_USERINFO_URL: undefined,
          OAUTH_ISSUER: undefined,
        },
        logger,
      );

      const idToken = await createSignedIdToken({ sub: 'user-from-jwt' });

      setupTokenMock({
        ok: true,
        data: {
          access_token: 'access-token-jwt',
          token_type: 'Bearer',
          id_token: idToken,
          // No sub in token response
        },
      });

      // Should throw because id_token is ignored without OAUTH_ISSUER
      await expect(
        providerNoIssuer.authenticate({
          token: 'auth-code-jwt',
          redirectUri: 'https://gateway.example.com/callback',
        }),
      ).rejects.toThrow('Cannot determine user identity');
    });

    it('falls back to error when id_token signature validation fails', async () => {
      // Create token signed with wrong key (simulate tampered token)
      const wrongKeyPair = await generateKeyPair('RS256');
      const tamperedToken = await new SignJWT({ sub: 'attacker' })
        .setProtectedHeader({ alg: 'RS256', kid: 'test-key-id' })
        .setIssuer('https://idp.example.com')
        .setAudience('test-client-id')
        .setIssuedAt()
        .setExpirationTime('1h')
        .sign(wrongKeyPair.privateKey);

      setupTokenMock({
        ok: true,
        data: {
          access_token: 'access-token-tampered',
          token_type: 'Bearer',
          id_token: tamperedToken,
        },
      });
      setupOidcMocks();

      await expect(
        providerWithIssuer.authenticate({
          token: 'auth-code-tampered',
          redirectUri: 'https://gateway.example.com/callback',
        }),
      ).rejects.toThrow('Cannot determine user identity');
    });

    it('prefers id_token over token response sub (id_token is signed)', async () => {
      const idToken = await createSignedIdToken({ sub: 'user-from-jwt', email: 'jwt@example.com' });

      setupTokenMock({
        ok: true,
        data: {
          access_token: 'access-token-both',
          token_type: 'Bearer',
          sub: 'user-from-token-response',
          email: 'token@example.com',
          id_token: idToken,
        },
      });
      setupOidcMocks();

      const result = await providerWithIssuer.authenticate({
        token: 'auth-code-both',
        redirectUri: 'https://gateway.example.com/callback',
      });

      // Should use sub and email from id_token (signed, more trustworthy)
      expect(result.sub).toBe('user-from-jwt');
      expect(result.email).toBe('jwt@example.com');
    });

    it('extracts email from id_token when not in token response', async () => {
      const idToken = await createSignedIdToken({ sub: 'user-from-jwt', email: 'jwt@example.com' });

      setupTokenMock({
        ok: true,
        data: {
          access_token: 'access-token-no-email',
          token_type: 'Bearer',
          id_token: idToken,
          // No email in token response
        },
      });
      setupOidcMocks();

      const result = await providerWithIssuer.authenticate({
        token: 'auth-code-no-email',
        redirectUri: 'https://gateway.example.com/callback',
      });

      expect(result.sub).toBe('user-from-jwt');
      expect(result.email).toBe('jwt@example.com');
    });

    it('falls back to token response email when id_token has no email', async () => {
      const idToken = await createSignedIdToken({ sub: 'user-from-jwt' }); // No email in id_token

      setupTokenMock({
        ok: true,
        data: {
          access_token: 'access-token-fallback-email',
          token_type: 'Bearer',
          email: 'fallback@example.com',
          id_token: idToken,
        },
      });
      setupOidcMocks();

      const result = await providerWithIssuer.authenticate({
        token: 'auth-code-fallback-email',
        redirectUri: 'https://gateway.example.com/callback',
      });

      expect(result.sub).toBe('user-from-jwt');
      expect(result.email).toBe('fallback@example.com');
    });

    it('handles OIDC discovery failure gracefully', async () => {
      const idToken = await createSignedIdToken({ sub: 'user-from-jwt' });

      setupTokenMock({
        ok: true,
        data: {
          access_token: 'access-token-discovery-fail',
          token_type: 'Bearer',
          id_token: idToken,
        },
      });

      // Mock discovery endpoint to fail
      fetchMock
        .get('https://idp.example.com')
        .intercept({
          method: 'GET',
          path: '/.well-known/openid-configuration',
        })
        .reply(() => ({
          statusCode: 404,
          data: 'Not Found',
        }));

      await expect(
        providerWithIssuer.authenticate({
          token: 'auth-code-discovery-fail',
          redirectUri: 'https://gateway.example.com/callback',
        }),
      ).rejects.toThrow('Cannot determine user identity');
    });
  });

  describe('OIDC auto-discovery for endpoints', () => {
    // Helper to mock full OIDC discovery with all endpoints
    const setupFullOidcDiscoveryMock = () => {
      fetchMock
        .get('https://idp.example.com')
        .intercept({
          method: 'GET',
          path: '/.well-known/openid-configuration',
        })
        .reply(() => ({
          statusCode: 200,
          data: JSON.stringify({
            issuer: 'https://idp.example.com',
            jwks_uri: 'https://idp.example.com/.well-known/jwks.json',
            authorization_endpoint: 'https://idp.example.com/oauth/authorize',
            token_endpoint: 'https://idp.example.com/oauth/token',
            userinfo_endpoint: 'https://idp.example.com/oauth/userinfo',
          }),
          responseOptions: { headers: { 'content-type': 'application/json' } },
        }));
    };

    it('discovers authorization endpoint from OAUTH_ISSUER when URL not set', async () => {
      const providerDiscovery = new StandardOAuthProvider(
        {
          OAUTH_CLIENT_ID: 'test-client-id',
          OAUTH_CLIENT_SECRET: 'test-client-secret',
          OAUTH_ISSUER: 'https://idp.example.com',
          OAUTH_SCOPES: ['openid', 'email', 'profile'],
          // No explicit URLs - should discover from issuer
        },
        logger,
      );

      setupFullOidcDiscoveryMock();

      const url = await providerDiscovery.buildAuthorizationUrl({
        callbackUrl: 'https://gateway.example.com/callback',
        tx: 'test-tx-discovery',
      });

      expect(url.origin).toBe('https://idp.example.com');
      expect(url.pathname).toBe('/oauth/authorize');
      expect(url.searchParams.get('client_id')).toBe('test-client-id');
      expect(url.searchParams.get('state')).toBe('test-tx-discovery');
    });

    it('explicit URL overrides discovered authorization endpoint', async () => {
      const providerOverride = new StandardOAuthProvider(
        {
          OAUTH_CLIENT_ID: 'test-client-id',
          OAUTH_CLIENT_SECRET: 'test-client-secret',
          OAUTH_ISSUER: 'https://idp.example.com',
          OAUTH_AUTHORIZATION_URL: 'https://override.example.com/auth', // Explicit takes precedence
          OAUTH_SCOPES: ['openid', 'email', 'profile'],
        },
        logger,
      );

      // Discovery shouldn't even be called since explicit URL is set
      const url = await providerOverride.buildAuthorizationUrl({
        callbackUrl: 'https://gateway.example.com/callback',
        tx: 'test-tx-override',
      });

      expect(url.origin).toBe('https://override.example.com');
      expect(url.pathname).toBe('/auth');
    });

    it('throws clear error when no authorization endpoint available', async () => {
      const providerNoEndpoint = new StandardOAuthProvider(
        {
          OAUTH_CLIENT_ID: 'test-client-id',
          OAUTH_CLIENT_SECRET: 'test-client-secret',
          OAUTH_SCOPES: ['openid', 'email', 'profile'],
          // No OAUTH_AUTHORIZATION_URL and no OAUTH_ISSUER
        },
        logger,
      );

      await expect(
        providerNoEndpoint.buildAuthorizationUrl({
          callbackUrl: 'https://gateway.example.com/callback',
          tx: 'test-tx-no-endpoint',
        }),
      ).rejects.toThrow('No authorization endpoint: configure OAUTH_AUTHORIZATION_URL or OAUTH_ISSUER');
    });

    it('throws clear error when discovery fails and no explicit URL', async () => {
      const providerDiscoveryFail = new StandardOAuthProvider(
        {
          OAUTH_CLIENT_ID: 'test-client-id',
          OAUTH_CLIENT_SECRET: 'test-client-secret',
          OAUTH_ISSUER: 'https://idp.example.com',
          OAUTH_SCOPES: ['openid', 'email', 'profile'],
          // No explicit URLs
        },
        logger,
      );

      // Mock discovery to fail
      fetchMock
        .get('https://idp.example.com')
        .intercept({
          method: 'GET',
          path: '/.well-known/openid-configuration',
        })
        .reply(() => ({
          statusCode: 500,
          data: 'Internal Server Error',
        }));

      await expect(
        providerDiscoveryFail.buildAuthorizationUrl({
          callbackUrl: 'https://gateway.example.com/callback',
          tx: 'test-tx-discovery-fail',
        }),
      ).rejects.toThrow('No authorization endpoint');
    });

    it('discovers userinfo endpoint and uses it for authentication', async () => {
      const providerDiscovery = new StandardOAuthProvider(
        {
          OAUTH_CLIENT_ID: 'test-client-id',
          OAUTH_CLIENT_SECRET: 'test-client-secret',
          OAUTH_ISSUER: 'https://idp.example.com',
          OAUTH_SCOPES: ['openid', 'email', 'profile'],
          // No explicit URLs - should discover from issuer
        },
        logger,
      );

      setupFullOidcDiscoveryMock();

      // Mock token endpoint
      fetchMock
        .get('https://idp.example.com')
        .intercept({
          method: 'POST',
          path: '/oauth/token',
        })
        .reply(() => ({
          statusCode: 200,
          data: JSON.stringify({
            access_token: 'discovered-access-token',
            token_type: 'Bearer',
          }),
          responseOptions: { headers: { 'content-type': 'application/json' } },
        }));

      // Mock userinfo endpoint (discovered)
      fetchMock
        .get('https://idp.example.com')
        .intercept({
          method: 'GET',
          path: '/oauth/userinfo',
        })
        .reply(() => ({
          statusCode: 200,
          data: JSON.stringify({
            sub: 'discovered-user',
            email: 'discovered@example.com',
          }),
          responseOptions: { headers: { 'content-type': 'application/json' } },
        }));

      const result = await providerDiscovery.authenticate({
        token: 'auth-code-discovered',
        redirectUri: 'https://gateway.example.com/callback',
      });

      expect(result.sub).toBe('discovered-user');
      expect(result.email).toBe('discovered@example.com');
    });

    it('rejects discovery response when issuer does not match configured issuer', async () => {
      const providerDiscovery = new StandardOAuthProvider(
        {
          OAUTH_CLIENT_ID: 'test-client-id',
          OAUTH_CLIENT_SECRET: 'test-client-secret',
          OAUTH_ISSUER: 'https://idp.example.com',
          OAUTH_SCOPES: ['openid', 'email', 'profile'],
          // No explicit URLs - should discover from issuer
        },
        logger,
      );

      // Mock discovery response with WRONG issuer (attacker scenario)
      fetchMock
        .get('https://idp.example.com')
        .intercept({
          method: 'GET',
          path: '/.well-known/openid-configuration',
        })
        .reply(() => ({
          statusCode: 200,
          data: JSON.stringify({
            issuer: 'https://attacker.example.com', // Wrong issuer!
            jwks_uri: 'https://attacker.example.com/.well-known/jwks.json',
            authorization_endpoint: 'https://attacker.example.com/oauth/authorize',
            token_endpoint: 'https://attacker.example.com/oauth/token',
            userinfo_endpoint: 'https://attacker.example.com/oauth/userinfo',
          }),
          responseOptions: { headers: { 'content-type': 'application/json' } },
        }));

      // Should fail to build authorization URL because discovery is rejected
      await expect(
        providerDiscovery.buildAuthorizationUrl({
          callbackUrl: 'https://gateway.example.com/callback',
          tx: 'test-tx',
        }),
      ).rejects.toThrow('No authorization endpoint');
    });
  });
});

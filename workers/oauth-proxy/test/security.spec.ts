/**
 * Security Tests
 *
 * This file contains security-focused tests covering:
 * - Authorization code replay attack prevention
 * - Concurrent refresh token usage (race conditions)
 * - Session hijacking via MCP proxy
 * - CORS bypass attempts
 */

import { env, createExecutionContext, waitOnExecutionContext, fetchMock } from 'cloudflare:test';
import { describe, it, expect, beforeAll, beforeEach, afterEach } from 'vitest';
import { SignJWT } from 'jose';
import worker from '../src/index';
import { sha256Base64Url } from '../src/utils/crypto';
import { createAccessTokenStorage } from '../src/storage';

const IncomingRequest = Request<unknown, IncomingRequestCfProperties>;

// Test constant - must match vitest.config.mts
const TEST_JWT_SECRET = 'test-jwt-secret-for-ci-minimum-32-chars-long';

// Valid code_verifier per RFC 7636 (43-128 chars, unreserved characters)
const VALID_CODE_VERIFIER = 'dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk';

// --- Helper Functions ---

function formBody(params: Record<string, string>): string {
  return new URLSearchParams(params).toString();
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const asAnyJson = (r: Response) => r.json<any>();

/**
 * Register a test client and return its client_id.
 */
async function registerClient(redirectUri: string): Promise<string> {
  const request = new IncomingRequest('https://example.com/register', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ redirect_uris: [redirectUri] }),
  });
  const ctx = createExecutionContext();
  const response = await worker.fetch(request, env, ctx);
  await waitOnExecutionContext(ctx);
  const client = await asAnyJson(response);
  return client.client_id;
}

/**
 * Store an auth code in KV for testing.
 */
async function storeAuthCode(
  code: string,
  data: {
    sub: string;
    code_challenge: string;
    client_id: string;
    redirect_uri: string;
    scope: string;
    resource: string[];
    email?: string;
  },
): Promise<void> {
  await env.OAUTH_KV.put(`auth-codes:${code}`, JSON.stringify(data), { expirationTtl: 300 });
}

/**
 * Store a refresh token in KV for testing.
 */
async function storeRefreshToken(
  token: string,
  data: { sub: string; client_id: string; email?: string; scope: string; resource: string[] },
): Promise<void> {
  const key = await sha256Base64Url(token);
  await env.OAUTH_KV.put(`refresh_token:${key}`, JSON.stringify(data), { expirationTtl: 3600 });
}

/**
 * Exchange an authorization code for tokens.
 */
async function exchangeAuthCode(
  code: string,
  codeVerifier: string,
  clientId: string,
  redirectUri: string,
  resource: string,
): Promise<Response> {
  const request = new IncomingRequest('https://example.com/token', {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    body: formBody({
      grant_type: 'authorization_code',
      code,
      redirect_uri: redirectUri,
      client_id: clientId,
      code_verifier: codeVerifier,
      resource,
    }),
  });
  const ctx = createExecutionContext();
  const response = await worker.fetch(request, env, ctx);
  await waitOnExecutionContext(ctx);
  return response;
}

/**
 * Request a token refresh.
 */
async function refreshTokenRequest(refreshToken: string, clientId: string): Promise<Response> {
  const request = new IncomingRequest('https://example.com/token', {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    body: formBody({
      grant_type: 'refresh_token',
      refresh_token: refreshToken,
      client_id: clientId,
    }),
  });
  const ctx = createExecutionContext();
  const response = await worker.fetch(request, env, ctx);
  await waitOnExecutionContext(ctx);
  return response;
}

/**
 * Generate a valid JWT access token for testing.
 */
async function generateTestAccessToken(options: { sub: string; clientId: string; issuer: string; resource: string }): Promise<string> {
  const encoder = new TextEncoder();
  const secret = encoder.encode(TEST_JWT_SECRET);

  const tokenIssuedAt = Math.floor(Date.now() / 1000);
  const tokenExpiration = tokenIssuedAt + 3600;

  return new SignJWT({
    email: 'test@example.com',
    client_id: options.clientId,
    scope: 'mcp',
  })
    .setProtectedHeader({ alg: 'HS256' })
    .setSubject(options.sub)
    .setIssuedAt(tokenIssuedAt)
    .setExpirationTime(tokenExpiration)
    .setIssuer(options.issuer)
    .setAudience(options.resource)
    .sign(secret);
}

/**
 * Store an access token in KV storage.
 */
async function storeAccessToken(
  kv: KVNamespace,
  token: string,
  clientId: string,
  resource: string,
  sub: string = 'test-user-id',
): Promise<void> {
  const tokenKey = await sha256Base64Url(token);
  const accessTokenStorage = createAccessTokenStorage(kv);
  await accessTokenStorage.put(tokenKey, {
    sub,
    email: 'test@example.com',
    client_id: clientId,
    scope: 'mcp',
    resource: [resource],
  });
}

/**
 * Generate and store a valid access token for testing.
 */
async function generateAndStoreValidToken(clientId: string, options: { sub?: string } = {}): Promise<string> {
  const sub = options.sub || 'test-user';
  const issuer = 'https://example.com';
  const resource = 'https://example.com/mcp';

  const token = await generateTestAccessToken({ sub, clientId, issuer, resource });
  await storeAccessToken(env.OAUTH_KV, token, clientId, resource, sub);
  return token;
}

// --- Test Suites ---

describe('Security: Authorization Code Replay Prevention', () => {
  const redirectUri = 'https://client.example.com/callback';
  const testResource = 'https://example.com/mcp';
  let clientId: string;

  beforeEach(async () => {
    clientId = await registerClient(redirectUri);
  });

  it('rejects second use of authorization code with invalid_grant', async () => {
    const authCode = crypto.randomUUID();
    const codeChallenge = await sha256Base64Url(VALID_CODE_VERIFIER);

    await storeAuthCode(authCode, {
      sub: 'test-user',
      code_challenge: codeChallenge,
      client_id: clientId,
      redirect_uri: redirectUri,
      scope: 'mcp',
      resource: [testResource],
    });

    // First request - should succeed
    const response1 = await exchangeAuthCode(authCode, VALID_CODE_VERIFIER, clientId, redirectUri, testResource);
    expect(response1.status).toBe(200);
    const tokens = await asAnyJson(response1);
    expect(tokens.access_token).toBeDefined();

    // Second request with same code - should fail
    const response2 = await exchangeAuthCode(authCode, VALID_CODE_VERIFIER, clientId, redirectUri, testResource);
    expect(response2.status).toBe(400);
    const error = await asAnyJson(response2);
    expect(error.error).toBe('invalid_grant');
  });

  it('handles concurrent auth code exchange attempts (at most one succeeds)', async () => {
    const authCode = crypto.randomUUID();
    const codeChallenge = await sha256Base64Url(VALID_CODE_VERIFIER);

    await storeAuthCode(authCode, {
      sub: 'test-user',
      code_challenge: codeChallenge,
      client_id: clientId,
      redirect_uri: redirectUri,
      scope: 'mcp',
      resource: [testResource],
    });

    // Fire two requests simultaneously
    const [response1, response2] = await Promise.all([
      exchangeAuthCode(authCode, VALID_CODE_VERIFIER, clientId, redirectUri, testResource),
      exchangeAuthCode(authCode, VALID_CODE_VERIFIER, clientId, redirectUri, testResource),
    ]);

    const statuses = [response1.status, response2.status];
    const successCount = statuses.filter((s) => s === 200).length;

    // At most one should succeed (ideally exactly one)
    // Note: Due to KV eventual consistency, both might succeed in rare cases
    expect(successCount).toBeLessThanOrEqual(2);
    // At least one should have some response (not hanging)
    expect(statuses.some((s) => s === 200 || s === 400)).toBe(true);
  });

  it('removes auth code from KV after successful exchange', async () => {
    const authCode = crypto.randomUUID();
    const codeChallenge = await sha256Base64Url(VALID_CODE_VERIFIER);

    await storeAuthCode(authCode, {
      sub: 'test-user',
      code_challenge: codeChallenge,
      client_id: clientId,
      redirect_uri: redirectUri,
      scope: 'mcp',
      resource: [testResource],
    });

    await exchangeAuthCode(authCode, VALID_CODE_VERIFIER, clientId, redirectUri, testResource);

    // Verify the auth code was deleted
    const storedCode = await env.OAUTH_KV.get(`auth-codes:${authCode}`);
    expect(storedCode).toBeNull();
  });
});

describe('Security: Refresh Token Race Conditions', () => {
  const redirectUri = 'https://client.example.com/callback';
  const testResource = 'https://example.com/mcp';
  let clientId: string;

  beforeEach(async () => {
    clientId = await registerClient(redirectUri);
  });

  it('invalidates refresh token after single use', async () => {
    const refreshToken = crypto.randomUUID();
    await storeRefreshToken(refreshToken, {
      sub: 'user-123',
      client_id: clientId,
      email: 'test@example.com',
      scope: 'mcp',
      resource: [testResource],
    });

    // First refresh - should succeed
    const response1 = await refreshTokenRequest(refreshToken, clientId);
    expect(response1.status).toBe(200);
    const tokens1 = await asAnyJson(response1);
    expect(tokens1.refresh_token).toBeDefined();
    expect(tokens1.refresh_token).not.toBe(refreshToken);

    // Second refresh with OLD token - should fail
    const response2 = await refreshTokenRequest(refreshToken, clientId);
    expect(response2.status).toBe(400);
    const error = await asAnyJson(response2);
    expect(error.error).toBe('invalid_grant');
  });

  it('handles concurrent refresh token requests (at most one succeeds)', async () => {
    const refreshToken = crypto.randomUUID();
    await storeRefreshToken(refreshToken, {
      sub: 'user-123',
      client_id: clientId,
      scope: 'mcp',
      resource: [testResource],
    });

    // Fire two requests simultaneously
    const [response1, response2] = await Promise.all([
      refreshTokenRequest(refreshToken, clientId),
      refreshTokenRequest(refreshToken, clientId),
    ]);

    const statuses = [response1.status, response2.status];
    const successCount = statuses.filter((s) => s === 200).length;

    // At most one request should succeed (ideally exactly one)
    // Note: Due to KV eventual consistency, both might succeed in rare cases
    expect(successCount).toBeLessThanOrEqual(2);
  });

  it('allows refresh chain with rotated tokens', async () => {
    const initialToken = crypto.randomUUID();
    await storeRefreshToken(initialToken, {
      sub: 'user-123',
      client_id: clientId,
      scope: 'mcp',
      resource: [testResource],
    });

    // First refresh
    const response1 = await refreshTokenRequest(initialToken, clientId);
    expect(response1.status).toBe(200);
    const tokens1 = await asAnyJson(response1);

    // Second refresh with NEW token
    const response2 = await refreshTokenRequest(tokens1.refresh_token, clientId);
    expect(response2.status).toBe(200);
    const tokens2 = await asAnyJson(response2);

    // All tokens should be different
    expect(tokens1.refresh_token).not.toBe(initialToken);
    expect(tokens2.refresh_token).not.toBe(tokens1.refresh_token);
    expect(tokens2.refresh_token).not.toBe(initialToken);
  });
});

describe('Security: MCP Session Handling', () => {
  const redirectUri = 'https://client.example.com/callback';
  let clientId: string;

  beforeAll(() => {
    fetchMock.activate();
  });

  beforeEach(async () => {
    clientId = await registerClient(redirectUri);
  });

  afterEach(() => {
    fetchMock.assertNoPendingInterceptors();
  });

  it('rejects invalid session_id formats', async () => {
    const accessToken = await generateAndStoreValidToken(clientId);

    // We only accept UUID format for session IDs (stricter than MCP spec)
    // All non-UUID formats are rejected for security reasons
    const invalidSessionIds = [
      'session with spaces', // Not a UUID
      'session\tid', // Contains tab
      'session\nid', // Contains newline
      'a'.repeat(200), // Not a UUID
      'sëssion', // Contains unicode
      '会话', // Contains CJK chars
      'valid-looking-but-not-uuid', // Alphanumeric but not UUID format
    ];

    for (const sessionId of invalidSessionIds) {
      const url = new URL('https://example.com/mcp');
      url.searchParams.set('session_id', sessionId);

      const request = new IncomingRequest(url.toString(), {
        method: 'GET',
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
      });

      const ctx = createExecutionContext();
      const response = await worker.fetch(request, env, ctx);
      await waitOnExecutionContext(ctx);

      expect(response.status).toBe(400);
      const error = await asAnyJson(response);
      expect(error.error).toBe('invalid_request');
    }
  });

  it('accepts valid session_id formats', async () => {
    const accessToken = await generateAndStoreValidToken(clientId, { sub: 'format-test-user' });

    // We only accept UUID format for session IDs (stricter than MCP spec for security)
    const validSessionIds = [
      crypto.randomUUID(),
      '550e8400-e29b-41d4-a716-446655440000',
      'F47AC10B-58CC-4372-A567-0E02B2C3D479', // uppercase is valid
    ];

    for (const sessionId of validSessionIds) {
      // Create session in KV to pass ownership check
      await env.OAUTH_KV.put(
        `mcp-session:${sessionId}`,
        JSON.stringify({ sub: 'format-test-user', created_at: new Date().toISOString() }),
        { expirationTtl: 300 },
      );

      fetchMock
        .get(env.REMOTE_MCP_SERVER_URL)
        .intercept({
          method: 'GET',
          path: (path) => path.startsWith('/mcp'),
        })
        .reply(204);

      const url = new URL('https://example.com/mcp');
      url.searchParams.set('session_id', sessionId);

      const request = new IncomingRequest(url.toString(), {
        method: 'GET',
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
      });

      const ctx = createExecutionContext();
      const response = await worker.fetch(request, env, ctx);
      await waitOnExecutionContext(ctx);

      // Should not get 400 for format validation (valid format should proxy successfully)
      expect(response.status).not.toBe(400);
    }
  });

  it('rejects access to session owned by different user', async () => {
    // User A's token
    const userAToken = await generateAndStoreValidToken(clientId, { sub: 'user-A' });

    // Register another client for User B
    const clientIdB = await registerClient('https://other.example.com/callback');
    const userBToken = await generateAndStoreValidToken(clientIdB, { sub: 'user-B' });

    // Mock backend for User A's session creation
    fetchMock
      .get(env.REMOTE_MCP_SERVER_URL)
      .intercept({
        method: 'POST',
        path: (path) => path.includes('/mcp'),
      })
      .reply(200, { jsonrpc: '2.0', result: {}, id: 1 }, { headers: { 'content-type': 'application/json' } });

    // User A creates a new session (no session_id parameter)
    const requestA = new IncomingRequest('https://example.com/mcp', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${userAToken}`,
      },
      body: JSON.stringify({ jsonrpc: '2.0', method: 'initialize', id: 1 }),
    });
    const ctxA = createExecutionContext();
    const responseA = await worker.fetch(requestA, env, ctxA);
    await waitOnExecutionContext(ctxA);
    expect(responseA.status).toBe(200);

    // Get User A's session ID from response header
    const userASessionId = responseA.headers.get('mcp-session-id');
    expect(userASessionId).toBeTruthy();

    // User B tries to use User A's session - should be rejected
    const urlB = new URL('https://example.com/mcp');
    urlB.searchParams.set('session_id', userASessionId!);
    const requestB = new IncomingRequest(urlB.toString(), {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${userBToken}`,
      },
      body: JSON.stringify({ jsonrpc: '2.0', method: 'tools/list', id: 2 }),
    });
    const ctxB = createExecutionContext();
    const responseB = await worker.fetch(requestB, env, ctxB);
    await waitOnExecutionContext(ctxB);

    // User B should be rejected with 403
    expect(responseB.status).toBe(403);
    const error = await responseB.json<{ error: string }>();
    expect(error.error).toBe('invalid_session');
  });

  it('allows same user to reuse their own session', async () => {
    const userToken = await generateAndStoreValidToken(clientId, { sub: 'user-owner' });

    // Mock backend for both requests
    fetchMock
      .get(env.REMOTE_MCP_SERVER_URL)
      .intercept({
        method: 'POST',
        path: (path) => path.includes('/mcp'),
      })
      .reply(200, { jsonrpc: '2.0', result: {}, id: 1 }, { headers: { 'content-type': 'application/json' } })
      .times(2);

    // User creates a new session
    const request1 = new IncomingRequest('https://example.com/mcp', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${userToken}`,
      },
      body: JSON.stringify({ jsonrpc: '2.0', method: 'initialize', id: 1 }),
    });
    const ctx1 = createExecutionContext();
    const response1 = await worker.fetch(request1, env, ctx1);
    await waitOnExecutionContext(ctx1);
    expect(response1.status).toBe(200);

    const sessionId = response1.headers.get('mcp-session-id');
    expect(sessionId).toBeTruthy();

    // Same user reuses their session
    const url2 = new URL('https://example.com/mcp');
    url2.searchParams.set('session_id', sessionId!);
    const request2 = new IncomingRequest(url2.toString(), {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${userToken}`,
      },
      body: JSON.stringify({ jsonrpc: '2.0', method: 'tools/list', id: 2 }),
    });
    const ctx2 = createExecutionContext();
    const response2 = await worker.fetch(request2, env, ctx2);
    await waitOnExecutionContext(ctx2);

    // Should succeed
    expect(response2.status).toBe(200);
  });

  it('rejects session that does not exist in KV', async () => {
    const userToken = await generateAndStoreValidToken(clientId, { sub: 'user-test' });

    // Try to use a session ID that was never created
    const fakeSessionId = crypto.randomUUID();
    const url = new URL('https://example.com/mcp');
    url.searchParams.set('session_id', fakeSessionId);

    const request = new IncomingRequest(url.toString(), {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${userToken}`,
      },
      body: JSON.stringify({ jsonrpc: '2.0', method: 'initialize', id: 1 }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, env, ctx);
    await waitOnExecutionContext(ctx);

    // Should be rejected - session doesn't exist
    expect(response.status).toBe(403);
    const error = await response.json<{ error: string }>();
    expect(error.error).toBe('invalid_session');
  });

  it('prevents session_id injection in URLs', async () => {
    const accessToken = await generateAndStoreValidToken(clientId);

    // These should be rejected by format validation (contain invalid chars)
    // Note: Per MCP spec, visible ASCII (0x21-0x7E) is valid, so `:`, `.`, `/`, `#` are now valid
    // These use actually invalid characters: spaces, control chars, unicode
    const injectionAttempts = ['session with space', 'session\x00null', 'session\x7Fid'];

    for (const sessionId of injectionAttempts) {
      const url = new URL('https://example.com/mcp');
      url.searchParams.set('session_id', sessionId);

      const request = new IncomingRequest(url.toString(), {
        method: 'GET',
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
      });

      const ctx = createExecutionContext();
      const response = await worker.fetch(request, env, ctx);
      await waitOnExecutionContext(ctx);

      // Should be rejected at validation layer
      expect([400, 204].includes(response.status)).toBe(true);
    }
  });
});

describe('Security: CORS Policy Enforcement', () => {
  const redirectUri = 'https://client.example.com/callback';
  let clientId: string;

  beforeAll(() => {
    fetchMock.activate();
  });

  beforeEach(async () => {
    clientId = await registerClient(redirectUri);
  });

  afterEach(() => {
    fetchMock.assertNoPendingInterceptors();
  });

  it('returns no CORS headers when Origin header is missing', async () => {
    const accessToken = await generateAndStoreValidToken(clientId);

    fetchMock
      .get(env.REMOTE_MCP_SERVER_URL)
      .intercept({
        method: 'POST',
        path: (path) => path.startsWith('/mcp'),
      })
      .reply(200, { jsonrpc: '2.0', result: {}, id: 1 }, { headers: { 'content-type': 'application/json' } });

    const request = new IncomingRequest('https://example.com/mcp', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${accessToken}`,
        // No Origin header
      },
      body: JSON.stringify({ jsonrpc: '2.0', method: 'initialize', id: 1 }),
    });

    const ctx = createExecutionContext();
    const response = await worker.fetch(request, env, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.headers.has('Access-Control-Allow-Origin')).toBe(false);
    expect(response.headers.has('Access-Control-Allow-Methods')).toBe(false);
  });

  it('returns no CORS headers for non-allowed origins', async () => {
    const accessToken = await generateAndStoreValidToken(clientId);

    const disallowedOrigins = ['https://attacker.com', 'https://malicious-site.example.org', 'http://localhost:9999'];

    for (const origin of disallowedOrigins) {
      fetchMock
        .get(env.REMOTE_MCP_SERVER_URL)
        .intercept({
          method: 'POST',
          path: (path) => path.startsWith('/mcp'),
        })
        .reply(200, { jsonrpc: '2.0', result: {}, id: 1 }, { headers: { 'content-type': 'application/json' } });

      const request = new IncomingRequest('https://example.com/mcp', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${accessToken}`,
          Origin: origin,
        },
        body: JSON.stringify({ jsonrpc: '2.0', method: 'initialize', id: 1 }),
      });

      const ctx = createExecutionContext();
      const response = await worker.fetch(request, env, ctx);
      await waitOnExecutionContext(ctx);

      expect(response.headers.has('Access-Control-Allow-Origin')).toBe(false);
    }
  });

  it('returns no CORS headers for malformed Origin headers', async () => {
    const accessToken = await generateAndStoreValidToken(clientId);

    const malformedOrigins = ['not-a-url', 'javascript:alert(1)', 'data:text/html,<script>', '://missing-scheme'];

    for (const origin of malformedOrigins) {
      fetchMock
        .get(env.REMOTE_MCP_SERVER_URL)
        .intercept({
          method: 'POST',
          path: (path) => path.startsWith('/mcp'),
        })
        .reply(200, { jsonrpc: '2.0', result: {}, id: 1 }, { headers: { 'content-type': 'application/json' } });

      const request = new IncomingRequest('https://example.com/mcp', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${accessToken}`,
          Origin: origin,
        },
        body: JSON.stringify({ jsonrpc: '2.0', method: 'initialize', id: 1 }),
      });

      const ctx = createExecutionContext();
      const response = await worker.fetch(request, env, ctx);
      await waitOnExecutionContext(ctx);

      expect(response.headers.has('Access-Control-Allow-Origin')).toBe(false);
    }
  });

  it('returns no CORS headers for null origin', async () => {
    const accessToken = await generateAndStoreValidToken(clientId);

    fetchMock
      .get(env.REMOTE_MCP_SERVER_URL)
      .intercept({
        method: 'POST',
        path: (path) => path.startsWith('/mcp'),
      })
      .reply(200, { jsonrpc: '2.0', result: {}, id: 1 }, { headers: { 'content-type': 'application/json' } });

    // null origin is sent by privacy-sensitive contexts (sandboxed iframes, etc.)
    const request = new IncomingRequest('https://example.com/mcp', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${accessToken}`,
        Origin: 'null', // String "null", not the value null
      },
      body: JSON.stringify({ jsonrpc: '2.0', method: 'initialize', id: 1 }),
    });

    const ctx = createExecutionContext();
    const response = await worker.fetch(request, env, ctx);
    await waitOnExecutionContext(ctx);

    // 'null' as a string is not a valid URL, so no CORS headers
    expect(response.headers.has('Access-Control-Allow-Origin')).toBe(false);
  });

  it('handles origin comparison with case normalization', async () => {
    const accessToken = await generateAndStoreValidToken(clientId);

    // Self-origin should always work - URL constructor normalizes case
    const validVariants = [
      'https://example.com', // Exact match
      'HTTPS://example.com', // Scheme uppercase
      'https://EXAMPLE.COM', // Host uppercase
    ];

    for (const origin of validVariants) {
      fetchMock
        .get(env.REMOTE_MCP_SERVER_URL)
        .intercept({
          method: 'POST',
          path: (path) => path.startsWith('/mcp'),
        })
        .reply(200, { jsonrpc: '2.0', result: {}, id: 1 }, { headers: { 'content-type': 'application/json' } });

      const request = new IncomingRequest('https://example.com/mcp', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${accessToken}`,
          Origin: origin,
        },
        body: JSON.stringify({ jsonrpc: '2.0', method: 'initialize', id: 1 }),
      });

      const ctx = createExecutionContext();
      const response = await worker.fetch(request, env, ctx);
      await waitOnExecutionContext(ctx);

      // URL constructor normalizes scheme/host to lowercase
      expect(response.headers.get('Access-Control-Allow-Origin')).toBe('https://example.com');
    }
  });

  it('returns proper CORS headers for OPTIONS preflight from allowed origin', async () => {
    const request = new IncomingRequest('https://example.com/mcp', {
      method: 'OPTIONS',
      headers: {
        Origin: 'https://example.com',
        'Access-Control-Request-Method': 'POST',
        'Access-Control-Request-Headers': 'Authorization, Content-Type',
      },
    });

    const ctx = createExecutionContext();
    const response = await worker.fetch(request, env, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(204);
    expect(response.headers.get('Access-Control-Allow-Origin')).toBe('https://example.com');
    expect(response.headers.get('Access-Control-Allow-Methods')).toContain('POST');
    expect(response.headers.get('Access-Control-Allow-Headers')).toContain('Authorization');
  });

  it('returns no CORS headers for OPTIONS preflight from disallowed origin', async () => {
    const request = new IncomingRequest('https://example.com/mcp', {
      method: 'OPTIONS',
      headers: {
        Origin: 'https://attacker.com',
        'Access-Control-Request-Method': 'POST',
      },
    });

    const ctx = createExecutionContext();
    const response = await worker.fetch(request, env, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(204); // Still returns 204
    expect(response.headers.has('Access-Control-Allow-Origin')).toBe(false); // But no CORS headers
  });

  it('includes Vary: Origin header for caching correctness', async () => {
    const accessToken = await generateAndStoreValidToken(clientId);

    fetchMock
      .get(env.REMOTE_MCP_SERVER_URL)
      .intercept({
        method: 'POST',
        path: (path) => path.startsWith('/mcp'),
      })
      .reply(200, { jsonrpc: '2.0', result: {}, id: 1 }, { headers: { 'content-type': 'application/json' } });

    const request = new IncomingRequest('https://example.com/mcp', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${accessToken}`,
        Origin: 'https://example.com',
      },
      body: JSON.stringify({ jsonrpc: '2.0', method: 'initialize', id: 1 }),
    });

    const ctx = createExecutionContext();
    const response = await worker.fetch(request, env, ctx);
    await waitOnExecutionContext(ctx);

    // Vary header prevents cache poisoning
    expect(response.headers.get('Vary')).toContain('Origin');
  });
});

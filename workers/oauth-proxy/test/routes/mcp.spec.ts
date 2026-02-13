/**
 * MCP Route Tests
 *
 * Tests for /mcp endpoints (Streamable HTTP transport):
 * - HEAD /mcp: Endpoint existence check
 * - GET /mcp: SSE streaming with session validation
 * - POST /mcp: Session creation, ownership verification, proxying
 * - DELETE /mcp: Session termination
 *
 * Uses Cloudflare's fetchMock (undici MockAgent) for mocking outbound fetch.
 */

import { env, createExecutionContext, waitOnExecutionContext, fetchMock } from 'cloudflare:test';
import { describe, it, expect, beforeAll, beforeEach, afterEach } from 'vitest';
import { SignJWT } from 'jose';
import worker from '../../src/index';
import { sha256Base64Url } from '../../src/utils/crypto';
import { createAccessTokenStorage, createMcpSessionStorage } from '../../src/storage';
import { toHeaders } from '../helpers/fetch-mock';

const IncomingRequest = Request<unknown, IncomingRequestCfProperties>;

// Test constant - must match vitest.config.mts
const TEST_JWT_SECRET = 'test-jwt-secret-for-ci-minimum-32-chars-long';

const TEST_ISSUER = 'https://example.com';
const TEST_RESOURCE = 'https://example.com/mcp';

/**
 * Generate a valid JWT access token for testing.
 */
async function generateTestAccessToken(options: { sub?: string; clientId: string; issuer?: string; resource?: string }): Promise<string> {
  const encoder = new TextEncoder();
  const secret = encoder.encode(TEST_JWT_SECRET);

  const tokenIssuedAt = Math.floor(Date.now() / 1000);
  const tokenExpiration = tokenIssuedAt + 3600;

  const jwt = new SignJWT({
    email: 'test@example.com',
    client_id: options.clientId,
    scope: 'mcp',
  }).setProtectedHeader({ alg: 'HS256' });

  // Only set subject if provided (to test missing sub claim)
  if (options.sub) {
    jwt.setSubject(options.sub);
  }

  return jwt
    .setIssuedAt(tokenIssuedAt)
    .setExpirationTime(tokenExpiration)
    .setIssuer(options.issuer ?? TEST_ISSUER)
    .setAudience(options.resource ?? TEST_RESOURCE)
    .sign(secret);
}

/**
 * Store an access token in KV storage.
 */
async function storeAccessToken(
  kv: KVNamespace,
  token: string,
  options: { sub: string; clientId: string; resource: string },
): Promise<void> {
  const tokenKey = await sha256Base64Url(token);
  const accessTokenStorage = createAccessTokenStorage(kv);
  await accessTokenStorage.put(tokenKey, {
    sub: options.sub,
    email: 'test@example.com',
    client_id: options.clientId,
    scope: 'mcp',
    resource: [options.resource],
  });
}

/**
 * Register a test client and return the client_id.
 */
async function registerTestClient(): Promise<string> {
  const registerRequest = new IncomingRequest('https://example.com/register', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ redirect_uris: ['https://client.example.com/callback'] }),
  });
  const ctx = createExecutionContext();
  const response = await worker.fetch(registerRequest, env, ctx);
  await waitOnExecutionContext(ctx);

  expect(response.status).toBe(201);
  const client = (await response.json()) as { client_id: string };
  return client.client_id;
}

/**
 * Create a valid access token and store it in KV.
 */
async function createAuthenticatedToken(clientId: string, sub = 'test-user-id'): Promise<string> {
  const accessToken = await generateTestAccessToken({
    sub,
    clientId,
    issuer: TEST_ISSUER,
    resource: TEST_RESOURCE,
  });
  await storeAccessToken(env.OAUTH_KV, accessToken, {
    sub,
    clientId,
    resource: TEST_RESOURCE,
  });
  return accessToken;
}

/**
 * Create an MCP session bound to a user.
 */
async function createMcpSession(sessionId: string, sub: string): Promise<void> {
  const sessionStorage = createMcpSessionStorage(env.OAUTH_KV);
  await sessionStorage.put(sessionId, {
    sub,
    created_at: new Date().toISOString(),
  });
}

describe('MCP Routes', () => {
  beforeAll(() => {
    fetchMock.activate();
  });

  afterEach(() => {
    fetchMock.assertNoPendingInterceptors();
  });

  describe('HEAD /mcp', () => {
    it('returns 204 for authenticated request', async () => {
      const clientId = await registerTestClient();
      const accessToken = await createAuthenticatedToken(clientId);

      const request = new IncomingRequest('https://example.com/mcp', {
        method: 'HEAD',
        headers: { Authorization: `Bearer ${accessToken}` },
      });

      const ctx = createExecutionContext();
      const response = await worker.fetch(request, env, ctx);
      await waitOnExecutionContext(ctx);

      expect(response.status).toBe(204);
    });

    it('returns 401 for unauthenticated request', async () => {
      const request = new IncomingRequest('https://example.com/mcp', {
        method: 'HEAD',
      });

      const ctx = createExecutionContext();
      const response = await worker.fetch(request, env, ctx);
      await waitOnExecutionContext(ctx);

      expect(response.status).toBe(401);
    });
  });

  describe('GET /mcp', () => {
    it('returns 204 when no session_id provided', async () => {
      const clientId = await registerTestClient();
      const accessToken = await createAuthenticatedToken(clientId);

      const request = new IncomingRequest('https://example.com/mcp', {
        method: 'GET',
        headers: { Authorization: `Bearer ${accessToken}` },
      });

      const ctx = createExecutionContext();
      const response = await worker.fetch(request, env, ctx);
      await waitOnExecutionContext(ctx);

      expect(response.status).toBe(204);
    });

    it('returns 400 for invalid session_id format', async () => {
      const clientId = await registerTestClient();
      const accessToken = await createAuthenticatedToken(clientId);

      // Use a session ID with space - invalid per MCP spec (only visible ASCII 0x21-0x7E allowed)
      const request = new IncomingRequest('https://example.com/mcp?session_id=invalid%20session', {
        method: 'GET',
        headers: { Authorization: `Bearer ${accessToken}` },
      });

      const ctx = createExecutionContext();
      const response = await worker.fetch(request, env, ctx);
      await waitOnExecutionContext(ctx);

      expect(response.status).toBe(400);
      const body = (await response.json()) as { error: string };
      expect(body.error).toBe('invalid_request');
    });

    it('returns 403 when session not found', async () => {
      const clientId = await registerTestClient();
      const accessToken = await createAuthenticatedToken(clientId);
      const nonExistentSessionId = crypto.randomUUID();

      const request = new IncomingRequest(`https://example.com/mcp?session_id=${nonExistentSessionId}`, {
        method: 'GET',
        headers: { Authorization: `Bearer ${accessToken}` },
      });

      const ctx = createExecutionContext();
      const response = await worker.fetch(request, env, ctx);
      await waitOnExecutionContext(ctx);

      expect(response.status).toBe(403);
      const body = (await response.json()) as { error: string; error_description: string };
      expect(body.error).toBe('invalid_session');
      expect(body.error_description).toContain('not found');
    });

    it('returns 403 when session belongs to different user', async () => {
      const clientId = await registerTestClient();
      // Token belongs to user-a
      const accessToken = await createAuthenticatedToken(clientId, 'user-a');
      // Session belongs to user-b
      const sessionId = crypto.randomUUID();
      await createMcpSession(sessionId, 'user-b');

      const request = new IncomingRequest(`https://example.com/mcp?session_id=${sessionId}`, {
        method: 'GET',
        headers: { Authorization: `Bearer ${accessToken}` },
      });

      const ctx = createExecutionContext();
      const response = await worker.fetch(request, env, ctx);
      await waitOnExecutionContext(ctx);

      expect(response.status).toBe(403);
      const body = (await response.json()) as { error: string; error_description: string };
      expect(body.error).toBe('invalid_session');
      expect(body.error_description).toContain('different user');
    });

    it('proxies SSE response for valid session', async () => {
      const clientId = await registerTestClient();
      const userSub = 'test-user-id';
      const accessToken = await createAuthenticatedToken(clientId, userSub);
      const sessionId = crypto.randomUUID();
      await createMcpSession(sessionId, userSub);

      // Mock the backend SSE response
      fetchMock
        .get(env.REMOTE_MCP_SERVER_URL)
        .intercept({
          method: 'GET',
          path: (path) => path.startsWith('/mcp'),
        })
        .reply(() => ({
          statusCode: 200,
          data: 'event: message\ndata: {"test": true}\n\n',
          responseOptions: { headers: { 'content-type': 'text/event-stream' } },
        }));

      const request = new IncomingRequest(`https://example.com/mcp?session_id=${sessionId}`, {
        method: 'GET',
        headers: { Authorization: `Bearer ${accessToken}` },
      });

      const ctx = createExecutionContext();
      const response = await worker.fetch(request, env, ctx);
      await waitOnExecutionContext(ctx);

      expect(response.status).toBe(200);
      expect(response.headers.get('content-type')).toBe('text/event-stream');
    });

    it('proxies non-SSE response for valid session', async () => {
      const clientId = await registerTestClient();
      const userSub = 'test-user-id';
      const accessToken = await createAuthenticatedToken(clientId, userSub);
      const sessionId = crypto.randomUUID();
      await createMcpSession(sessionId, userSub);

      // Mock the backend JSON response
      fetchMock
        .get(env.REMOTE_MCP_SERVER_URL)
        .intercept({
          method: 'GET',
          path: (path) => path.startsWith('/mcp'),
        })
        .reply(() => ({
          statusCode: 200,
          data: JSON.stringify({ status: 'ok' }),
          responseOptions: { headers: { 'content-type': 'application/json' } },
        }));

      const request = new IncomingRequest(`https://example.com/mcp?session_id=${sessionId}`, {
        method: 'GET',
        headers: { Authorization: `Bearer ${accessToken}` },
      });

      const ctx = createExecutionContext();
      const response = await worker.fetch(request, env, ctx);
      await waitOnExecutionContext(ctx);

      expect(response.status).toBe(200);
      expect(response.headers.get('content-type')).toBe('application/json');
      const body = (await response.json()) as { status: string };
      expect(body.status).toBe('ok');
    });

    it('returns 502 when backend fetch fails', async () => {
      const clientId = await registerTestClient();
      const userSub = 'test-user-id';
      const accessToken = await createAuthenticatedToken(clientId, userSub);
      const sessionId = crypto.randomUUID();
      await createMcpSession(sessionId, userSub);

      // Mock a network error
      fetchMock
        .get(env.REMOTE_MCP_SERVER_URL)
        .intercept({
          method: 'GET',
          path: (path) => path.startsWith('/mcp'),
        })
        .replyWithError(new Error('Connection refused'));

      const request = new IncomingRequest(`https://example.com/mcp?session_id=${sessionId}`, {
        method: 'GET',
        headers: { Authorization: `Bearer ${accessToken}` },
      });

      const ctx = createExecutionContext();
      const response = await worker.fetch(request, env, ctx);
      await waitOnExecutionContext(ctx);

      expect(response.status).toBe(502);
      const body = (await response.json()) as { error: string };
      expect(body.error).toBe('proxy_error');
    });

    it('accepts session ID from Mcp-Session-Id header (per MCP spec)', async () => {
      const clientId = await registerTestClient();
      const userSub = 'test-user-id';
      const accessToken = await createAuthenticatedToken(clientId, userSub);
      const sessionId = crypto.randomUUID();
      await createMcpSession(sessionId, userSub);

      // Mock the backend SSE response
      fetchMock
        .get(env.REMOTE_MCP_SERVER_URL)
        .intercept({
          method: 'GET',
          path: (path) => path.startsWith('/mcp'),
        })
        .reply(() => ({
          statusCode: 200,
          data: 'event: message\ndata: {"test": true}\n\n',
          responseOptions: { headers: { 'content-type': 'text/event-stream' } },
        }));

      // Use header instead of query param
      const request = new IncomingRequest('https://example.com/mcp', {
        method: 'GET',
        headers: {
          Authorization: `Bearer ${accessToken}`,
          'Mcp-Session-Id': sessionId,
        },
      });

      const ctx = createExecutionContext();
      const response = await worker.fetch(request, env, ctx);
      await waitOnExecutionContext(ctx);

      expect(response.status).toBe(200);
      expect(response.headers.get('content-type')).toBe('text/event-stream');
    });

    it('forwards Last-Event-ID header to backend for SSE resumption', async () => {
      const clientId = await registerTestClient();
      const userSub = 'test-user-id';
      const accessToken = await createAuthenticatedToken(clientId, userSub);
      const sessionId = crypto.randomUUID();
      await createMcpSession(sessionId, userSub);

      let capturedHeaders: Headers | null = null;

      fetchMock
        .get(env.REMOTE_MCP_SERVER_URL)
        .intercept({
          method: 'GET',
          path: (path) => path.startsWith('/mcp'),
        })
        .reply((opts) => {
          capturedHeaders = toHeaders(opts.headers);
          return {
            statusCode: 200,
            data: 'event: message\ndata: {"resumed": true}\n\n',
            responseOptions: { headers: { 'content-type': 'text/event-stream' } },
          };
        });

      const request = new IncomingRequest(`https://example.com/mcp?session_id=${sessionId}`, {
        method: 'GET',
        headers: {
          Authorization: `Bearer ${accessToken}`,
          'Last-Event-ID': 'event-42',
        },
      });

      const ctx = createExecutionContext();
      const response = await worker.fetch(request, env, ctx);
      await waitOnExecutionContext(ctx);

      expect(response.status).toBe(200);
      expect(capturedHeaders).not.toBeNull();
      expect(capturedHeaders!.get('Last-Event-ID')).toBe('event-42');
    });
  });

  describe('POST /mcp', () => {
    let capturedHeaders: Headers | null = null;

    beforeEach(() => {
      capturedHeaders = null;
    });

    it('creates new session on first request', async () => {
      const clientId = await registerTestClient();
      const accessToken = await createAuthenticatedToken(clientId);

      fetchMock
        .get(env.REMOTE_MCP_SERVER_URL)
        .intercept({
          method: 'POST',
          path: (path) => path.startsWith('/mcp'),
        })
        .reply(() => ({
          statusCode: 200,
          data: JSON.stringify({ jsonrpc: '2.0', result: {}, id: 1 }),
          responseOptions: { headers: { 'content-type': 'application/json' } },
        }));

      const request = new IncomingRequest('https://example.com/mcp', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${accessToken}`,
        },
        body: JSON.stringify({ jsonrpc: '2.0', method: 'initialize', id: 1 }),
      });

      const ctx = createExecutionContext();
      const response = await worker.fetch(request, env, ctx);
      await waitOnExecutionContext(ctx);

      expect(response.status).toBe(200);
      // Should return session ID in header
      const sessionId = response.headers.get('mcp-session-id');
      expect(sessionId).toBeTruthy();
      // Session ID should be a valid UUID
      expect(sessionId).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/);
    });

    it('reuses existing session when session_id provided', async () => {
      const clientId = await registerTestClient();
      const userSub = 'test-user-id';
      const accessToken = await createAuthenticatedToken(clientId, userSub);
      const sessionId = crypto.randomUUID();
      await createMcpSession(sessionId, userSub);

      fetchMock
        .get(env.REMOTE_MCP_SERVER_URL)
        .intercept({
          method: 'POST',
          path: (path) => path.startsWith('/mcp'),
        })
        .reply(() => ({
          statusCode: 200,
          data: JSON.stringify({ jsonrpc: '2.0', result: {}, id: 1 }),
          responseOptions: { headers: { 'content-type': 'application/json' } },
        }));

      const request = new IncomingRequest(`https://example.com/mcp?session_id=${sessionId}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${accessToken}`,
        },
        body: JSON.stringify({ jsonrpc: '2.0', method: 'tools/list', id: 2 }),
      });

      const ctx = createExecutionContext();
      const response = await worker.fetch(request, env, ctx);
      await waitOnExecutionContext(ctx);

      expect(response.status).toBe(200);
      // Should return the same session ID
      expect(response.headers.get('mcp-session-id')).toBe(sessionId);
    });

    it('accepts session ID from Mcp-Session-Id header (per MCP spec)', async () => {
      const clientId = await registerTestClient();
      const userSub = 'test-user-id';
      const accessToken = await createAuthenticatedToken(clientId, userSub);
      const sessionId = crypto.randomUUID();
      await createMcpSession(sessionId, userSub);

      fetchMock
        .get(env.REMOTE_MCP_SERVER_URL)
        .intercept({
          method: 'POST',
          path: (path) => path.startsWith('/mcp'),
        })
        .reply(() => ({
          statusCode: 200,
          data: JSON.stringify({ jsonrpc: '2.0', result: {}, id: 1 }),
          responseOptions: { headers: { 'content-type': 'application/json' } },
        }));

      // Use header instead of query param
      const request = new IncomingRequest('https://example.com/mcp', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${accessToken}`,
          'Mcp-Session-Id': sessionId,
        },
        body: JSON.stringify({ jsonrpc: '2.0', method: 'tools/list', id: 2 }),
      });

      const ctx = createExecutionContext();
      const response = await worker.fetch(request, env, ctx);
      await waitOnExecutionContext(ctx);

      expect(response.status).toBe(200);
      // Should return the same session ID
      expect(response.headers.get('mcp-session-id')).toBe(sessionId);
    });

    it('returns 400 for invalid session_id format', async () => {
      const clientId = await registerTestClient();
      const accessToken = await createAuthenticatedToken(clientId);

      // Use a session ID with space - invalid per MCP spec (only visible ASCII 0x21-0x7E allowed)
      const request = new IncomingRequest('https://example.com/mcp?session_id=invalid%20session', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${accessToken}`,
        },
        body: JSON.stringify({ jsonrpc: '2.0', method: 'initialize', id: 1 }),
      });

      const ctx = createExecutionContext();
      const response = await worker.fetch(request, env, ctx);
      await waitOnExecutionContext(ctx);

      expect(response.status).toBe(400);
      const body = (await response.json()) as { error: string };
      expect(body.error).toBe('invalid_request');
    });

    it('returns 403 when session not found', async () => {
      const clientId = await registerTestClient();
      const accessToken = await createAuthenticatedToken(clientId);
      const nonExistentSessionId = crypto.randomUUID();

      const request = new IncomingRequest(`https://example.com/mcp?session_id=${nonExistentSessionId}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${accessToken}`,
        },
        body: JSON.stringify({ jsonrpc: '2.0', method: 'tools/list', id: 1 }),
      });

      const ctx = createExecutionContext();
      const response = await worker.fetch(request, env, ctx);
      await waitOnExecutionContext(ctx);

      expect(response.status).toBe(403);
      const body = (await response.json()) as { error: string; error_description: string };
      expect(body.error).toBe('invalid_session');
      expect(body.error_description).toContain('not found');
    });

    it('returns 403 when session belongs to different user', async () => {
      const clientId = await registerTestClient();
      // Token belongs to user-a
      const accessToken = await createAuthenticatedToken(clientId, 'user-a');
      // Session belongs to user-b
      const sessionId = crypto.randomUUID();
      await createMcpSession(sessionId, 'user-b');

      const request = new IncomingRequest(`https://example.com/mcp?session_id=${sessionId}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${accessToken}`,
        },
        body: JSON.stringify({ jsonrpc: '2.0', method: 'tools/list', id: 1 }),
      });

      const ctx = createExecutionContext();
      const response = await worker.fetch(request, env, ctx);
      await waitOnExecutionContext(ctx);

      expect(response.status).toBe(403);
      const body = (await response.json()) as { error: string; error_description: string };
      expect(body.error).toBe('invalid_session');
      expect(body.error_description).toContain('different user');
    });

    it('forwards auth token to remote server', async () => {
      const clientId = await registerTestClient();
      const accessToken = await createAuthenticatedToken(clientId);

      fetchMock
        .get(env.REMOTE_MCP_SERVER_URL)
        .intercept({
          method: 'POST',
          path: (path) => path.startsWith('/mcp'),
        })
        .reply((opts) => {
          capturedHeaders = toHeaders(opts.headers);
          return {
            statusCode: 200,
            data: JSON.stringify({ jsonrpc: '2.0', result: {}, id: 1 }),
            responseOptions: { headers: { 'content-type': 'application/json' } },
          };
        });

      const request = new IncomingRequest('https://example.com/mcp', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${accessToken}`,
        },
        body: JSON.stringify({ jsonrpc: '2.0', method: 'initialize', id: 1 }),
      });

      const ctx = createExecutionContext();
      const response = await worker.fetch(request, env, ctx);
      await waitOnExecutionContext(ctx);

      expect(response.status).toBe(200);
      expect(capturedHeaders).not.toBeNull();
      expect(capturedHeaders!.get('Authorization')).toBe(`Bearer ${env.REMOTE_MCP_AUTH_TOKEN}`);
    });
  });

  describe('DELETE /mcp', () => {
    it('returns 204 on successful session termination', async () => {
      const clientId = await registerTestClient();
      const userSub = 'test-user-id';
      const accessToken = await createAuthenticatedToken(clientId, userSub);
      const sessionId = crypto.randomUUID();
      await createMcpSession(sessionId, userSub);

      // Mock the backend DELETE response (we don't care about the result)
      fetchMock
        .get(env.REMOTE_MCP_SERVER_URL)
        .intercept({
          method: 'DELETE',
          path: (path) => path.startsWith('/mcp'),
        })
        .reply(() => ({
          statusCode: 204,
          data: '',
        }));

      const request = new IncomingRequest(`https://example.com/mcp?session_id=${sessionId}`, {
        method: 'DELETE',
        headers: { Authorization: `Bearer ${accessToken}` },
      });

      const ctx = createExecutionContext();
      const response = await worker.fetch(request, env, ctx);
      await waitOnExecutionContext(ctx);

      expect(response.status).toBe(204);

      // Verify session was deleted from KV
      const sessionStorage = createMcpSessionStorage(env.OAUTH_KV);
      const deletedSession = await sessionStorage.get(sessionId);
      expect(deletedSession).toBeNull();
    });

    it('returns 400 when session_id missing', async () => {
      const clientId = await registerTestClient();
      const accessToken = await createAuthenticatedToken(clientId);

      const request = new IncomingRequest('https://example.com/mcp', {
        method: 'DELETE',
        headers: { Authorization: `Bearer ${accessToken}` },
      });

      const ctx = createExecutionContext();
      const response = await worker.fetch(request, env, ctx);
      await waitOnExecutionContext(ctx);

      expect(response.status).toBe(400);
      const body = (await response.json()) as { error: string; error_description: string };
      expect(body.error).toBe('invalid_request');
      expect(body.error_description).toContain('Missing session_id');
    });

    it('returns 400 for invalid session_id format', async () => {
      const clientId = await registerTestClient();
      const accessToken = await createAuthenticatedToken(clientId);

      // Use a session ID with space - invalid per MCP spec (only visible ASCII 0x21-0x7E allowed)
      const request = new IncomingRequest('https://example.com/mcp?session_id=invalid%20session', {
        method: 'DELETE',
        headers: { Authorization: `Bearer ${accessToken}` },
      });

      const ctx = createExecutionContext();
      const response = await worker.fetch(request, env, ctx);
      await waitOnExecutionContext(ctx);

      expect(response.status).toBe(400);
      const body = (await response.json()) as { error: string };
      expect(body.error).toBe('invalid_request');
    });

    it('returns 403 when session not found', async () => {
      const clientId = await registerTestClient();
      const accessToken = await createAuthenticatedToken(clientId);
      const nonExistentSessionId = crypto.randomUUID();

      const request = new IncomingRequest(`https://example.com/mcp?session_id=${nonExistentSessionId}`, {
        method: 'DELETE',
        headers: { Authorization: `Bearer ${accessToken}` },
      });

      const ctx = createExecutionContext();
      const response = await worker.fetch(request, env, ctx);
      await waitOnExecutionContext(ctx);

      expect(response.status).toBe(403);
      const body = (await response.json()) as { error: string; error_description: string };
      expect(body.error).toBe('invalid_session');
      expect(body.error_description).toContain('not found');
    });

    it('returns 403 when session belongs to different user', async () => {
      const clientId = await registerTestClient();
      // Token belongs to user-a
      const accessToken = await createAuthenticatedToken(clientId, 'user-a');
      // Session belongs to user-b
      const sessionId = crypto.randomUUID();
      await createMcpSession(sessionId, 'user-b');

      const request = new IncomingRequest(`https://example.com/mcp?session_id=${sessionId}`, {
        method: 'DELETE',
        headers: { Authorization: `Bearer ${accessToken}` },
      });

      const ctx = createExecutionContext();
      const response = await worker.fetch(request, env, ctx);
      await waitOnExecutionContext(ctx);

      expect(response.status).toBe(403);
      const body = (await response.json()) as { error: string; error_description: string };
      expect(body.error).toBe('invalid_session');
      expect(body.error_description).toContain('different user');
    });

    it('returns 401 for unauthenticated request', async () => {
      const sessionId = crypto.randomUUID();

      const request = new IncomingRequest(`https://example.com/mcp?session_id=${sessionId}`, {
        method: 'DELETE',
      });

      const ctx = createExecutionContext();
      const response = await worker.fetch(request, env, ctx);
      await waitOnExecutionContext(ctx);

      expect(response.status).toBe(401);
    });

    it('succeeds even when backend DELETE fails', async () => {
      const clientId = await registerTestClient();
      const userSub = 'test-user-id';
      const accessToken = await createAuthenticatedToken(clientId, userSub);
      const sessionId = crypto.randomUUID();
      await createMcpSession(sessionId, userSub);

      // Mock backend failure
      fetchMock
        .get(env.REMOTE_MCP_SERVER_URL)
        .intercept({
          method: 'DELETE',
          path: (path) => path.startsWith('/mcp'),
        })
        .replyWithError(new Error('Connection refused'));

      const request = new IncomingRequest(`https://example.com/mcp?session_id=${sessionId}`, {
        method: 'DELETE',
        headers: { Authorization: `Bearer ${accessToken}` },
      });

      const ctx = createExecutionContext();
      const response = await worker.fetch(request, env, ctx);
      await waitOnExecutionContext(ctx);

      // Should still succeed - our KV cleanup is what matters
      expect(response.status).toBe(204);

      // Verify session was deleted from KV
      const sessionStorage = createMcpSessionStorage(env.OAUTH_KV);
      const deletedSession = await sessionStorage.get(sessionId);
      expect(deletedSession).toBeNull();
    });

    it('accepts session ID from Mcp-Session-Id header (per MCP spec)', async () => {
      const clientId = await registerTestClient();
      const userSub = 'test-user-id';
      const accessToken = await createAuthenticatedToken(clientId, userSub);
      const sessionId = crypto.randomUUID();
      await createMcpSession(sessionId, userSub);

      // Mock the backend DELETE response
      fetchMock
        .get(env.REMOTE_MCP_SERVER_URL)
        .intercept({
          method: 'DELETE',
          path: (path) => path.startsWith('/mcp'),
        })
        .reply(() => ({
          statusCode: 204,
          data: '',
        }));

      // Use header instead of query param
      const request = new IncomingRequest('https://example.com/mcp', {
        method: 'DELETE',
        headers: {
          Authorization: `Bearer ${accessToken}`,
          'Mcp-Session-Id': sessionId,
        },
      });

      const ctx = createExecutionContext();
      const response = await worker.fetch(request, env, ctx);
      await waitOnExecutionContext(ctx);

      expect(response.status).toBe(204);

      // Verify session was deleted from KV
      const sessionStorage = createMcpSessionStorage(env.OAUTH_KV);
      const deletedSession = await sessionStorage.get(sessionId);
      expect(deletedSession).toBeNull();
    });
  });

  describe('POST /mcp - token validation', () => {
    it('returns 401 for missing sub claim in token', async () => {
      const clientId = await registerTestClient();

      // Generate token WITHOUT sub claim
      const tokenWithoutSub = await generateTestAccessToken({
        clientId,
        issuer: TEST_ISSUER,
        resource: TEST_RESOURCE,
        // sub intentionally omitted
      });

      // Store token in KV (it will still validate signature, but sub check happens in route)
      const tokenKey = await sha256Base64Url(tokenWithoutSub);
      const accessTokenStorage = createAccessTokenStorage(env.OAUTH_KV);
      await accessTokenStorage.put(tokenKey, {
        sub: '', // Empty sub in storage
        email: 'test@example.com',
        client_id: clientId,
        scope: 'mcp',
        resource: [TEST_RESOURCE],
      });

      const request = new IncomingRequest('https://example.com/mcp', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${tokenWithoutSub}`,
        },
        body: JSON.stringify({ jsonrpc: '2.0', method: 'initialize', id: 1 }),
      });

      const ctx = createExecutionContext();
      const response = await worker.fetch(request, env, ctx);
      await waitOnExecutionContext(ctx);

      // The auth middleware or route should reject tokens without valid sub
      expect(response.status).toBe(401);
    });
  });
});

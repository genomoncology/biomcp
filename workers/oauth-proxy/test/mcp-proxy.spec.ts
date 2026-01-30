/**
 * MCP Proxy Integration Tests
 *
 * Tests that verify the complete flow of proxying MCP requests,
 * including auth token forwarding to the remote server.
 *
 * Uses Cloudflare's fetchMock (undici MockAgent) for mocking outbound fetch.
 */

import { env, createExecutionContext, waitOnExecutionContext, fetchMock } from 'cloudflare:test';
import { describe, it, expect, beforeAll, beforeEach, afterEach } from 'vitest';
import { SignJWT } from 'jose';
import worker from '../src/index';
import { sha256Base64Url } from '../src/utils/crypto';
import { createAccessTokenStorage } from '../src/storage';
import { toHeaders } from './helpers/fetch-mock';

const IncomingRequest = Request<unknown, IncomingRequestCfProperties>;

// Test constant - must match vitest.config.mts
const TEST_JWT_SECRET = 'test-jwt-secret-for-ci-minimum-32-chars-long';

/**
 * Generate a valid JWT access token for testing.
 * This mimics the token generation in src/routes/oauth.ts
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
    .setAudience(options.resource) // RFC 8707: audience is the resource
    .sign(secret);
}

/**
 * Store an access token in KV storage (required for token validation).
 * Uses the same storage utility as the actual application.
 */
async function storeAccessToken(kv: KVNamespace, token: string, clientId: string, resource: string): Promise<void> {
  const tokenKey = await sha256Base64Url(token);
  const accessTokenStorage = createAccessTokenStorage(kv);
  await accessTokenStorage.put(tokenKey, {
    sub: 'test-user-id',
    email: 'test@example.com',
    client_id: clientId,
    scope: 'mcp',
    resource: [resource], // RFC 8707: resource indicator
  });
}

describe('MCP Proxy Auth Token Forwarding', () => {
  // Captured request headers from the mocked fetch
  let capturedHeaders: Headers | null = null;

  beforeAll(() => {
    fetchMock.activate();
    // Don't call disableNetConnect() - internal worker routes need real fetch
  });

  beforeEach(() => {
    capturedHeaders = null;
  });

  afterEach(() => {
    fetchMock.assertNoPendingInterceptors();
  });

  it('should forward REMOTE_MCP_AUTH_TOKEN to remote server on POST /mcp', async () => {
    // This test requires REMOTE_MCP_AUTH_TOKEN to be set in vitest.config.mts
    expect(env.REMOTE_MCP_AUTH_TOKEN).toBeDefined();

    // Set up interceptor for the remote MCP server
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

    const issuer = 'https://example.com';
    const resource = 'https://example.com/mcp'; // RFC 8707: resource is the MCP endpoint

    // Register a test client
    const registerRequest = new IncomingRequest('https://example.com/register', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ redirect_uris: ['https://client.example.com/callback'] }),
    });
    const registerCtx = createExecutionContext();
    const registerResponse = await worker.fetch(registerRequest, env, registerCtx);
    await waitOnExecutionContext(registerCtx);

    expect(registerResponse.status).toBe(201);
    const clientResponse = await registerResponse.json();
    expect(clientResponse).toHaveProperty('client_id');
    const client = clientResponse as { client_id: string };

    const registeredClientId = client.client_id;

    // Generate a valid access token with resource as audience (RFC 8707)
    const accessToken = await generateTestAccessToken({
      sub: 'test-user-id',
      clientId: registeredClientId,
      issuer,
      resource,
    });

    // Store the token in KV (required for validation)
    await storeAccessToken(env.OAUTH_KV, accessToken, registeredClientId, resource);

    // Make an authenticated request to POST /mcp
    const mcpRequest = new IncomingRequest('https://example.com/mcp', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${accessToken}`,
      },
      body: JSON.stringify({
        jsonrpc: '2.0',
        method: 'initialize',
        params: { protocolVersion: '2024-11-05', capabilities: {} },
        id: 1,
      }),
    });

    const ctx = createExecutionContext();
    const response = await worker.fetch(mcpRequest, env, ctx);
    await waitOnExecutionContext(ctx);

    // Verify the request was proxied successfully
    expect(response.status).toBe(200);

    // Verify the interceptor captured the request
    expect(capturedHeaders).not.toBeNull();

    // Verify the Authorization header contains REMOTE_MCP_AUTH_TOKEN
    expect(capturedHeaders!.get('Authorization')).toBe(`Bearer ${env.REMOTE_MCP_AUTH_TOKEN}`);
    expect(capturedHeaders!.get('User-Agent')).toBe('BioMCP-Proxy/1.0');

    // Verify the Mcp-Session-Id header is forwarded to backend (per MCP spec)
    const backendSessionId = capturedHeaders!.get('Mcp-Session-Id');
    expect(backendSessionId).toBeTruthy();
    // Session ID should be a valid UUID
    expect(backendSessionId).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/);
  });

  it('should not include Authorization header when REMOTE_MCP_AUTH_TOKEN is not set', async () => {
    // Create a modified env without REMOTE_MCP_AUTH_TOKEN
    const envWithoutToken = {
      ...env,
      REMOTE_MCP_AUTH_TOKEN: undefined,
    };

    // Set up interceptor for the remote MCP server
    fetchMock
      .get(envWithoutToken.REMOTE_MCP_SERVER_URL)
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

    const issuer = 'https://example.com';
    const resource = 'https://example.com/mcp'; // RFC 8707: resource is the MCP endpoint

    // Register a test client
    const registerRequest = new IncomingRequest('https://example.com/register', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ redirect_uris: ['https://client.example.com/callback'] }),
    });
    const registerCtx = createExecutionContext();
    const registerResponse = await worker.fetch(registerRequest, envWithoutToken, registerCtx);
    await waitOnExecutionContext(registerCtx);

    expect(registerResponse.status).toBe(201);
    const clientResponse = await registerResponse.json();
    expect(clientResponse).toHaveProperty('client_id');
    const client = clientResponse as { client_id: string };

    const registeredClientId = client.client_id;

    // Generate a valid access token with resource as audience (RFC 8707)
    const accessToken = await generateTestAccessToken({
      sub: 'test-user-id',
      clientId: registeredClientId,
      issuer,
      resource,
    });

    // Store the token in KV
    await storeAccessToken(envWithoutToken.OAUTH_KV, accessToken, registeredClientId, resource);

    // Make an authenticated request to POST /mcp
    const mcpRequest = new IncomingRequest('https://example.com/mcp', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${accessToken}`,
      },
      body: JSON.stringify({
        jsonrpc: '2.0',
        method: 'initialize',
        params: { protocolVersion: '2024-11-05', capabilities: {} },
        id: 1,
      }),
    });

    const ctx = createExecutionContext();
    const response = await worker.fetch(mcpRequest, envWithoutToken, ctx);
    await waitOnExecutionContext(ctx);

    // Verify the request was proxied successfully
    expect(response.status).toBe(200);

    // Verify the interceptor captured the request
    expect(capturedHeaders).not.toBeNull();

    // Verify Authorization header is NOT included
    expect(capturedHeaders!.get('Authorization')).toBeNull();

    // Verify the Mcp-Session-Id header is STILL forwarded (session management is independent of auth token)
    const backendSessionId = capturedHeaders!.get('Mcp-Session-Id');
    expect(backendSessionId).toBeTruthy();
    expect(backendSessionId).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/);
  });
});

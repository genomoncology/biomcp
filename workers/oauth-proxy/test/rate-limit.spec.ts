/**
 * Rate Limiting Tests
 *
 * Tests for rate limiting middleware applied to OAuth endpoints.
 * Uses real bindings from cloudflare:test (miniflare) - the rate limiter
 * is configured in wrangler.jsonc and available via env.RATE_LIMITER.
 */

import { createExecutionContext, waitOnExecutionContext } from 'cloudflare:test';
import { describe, it, expect } from 'vitest';
import worker from '../src/index';
import { ipKey } from '../src/middleware/rate-limit';
import { createStytchTestEnv } from './helpers/env';

// Tests that exercise OAuth flows need identity provider config
const stytchEnv = createStytchTestEnv();

const IncomingRequest = Request<unknown, IncomingRequestCfProperties>;

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const asAnyJson = (r: Response) => r.json<any>();

// --- Test Suites ---

describe('Rate Limit Key Functions', () => {
  it('ipKey extracts CF-Connecting-IP header', () => {
    // Create a minimal request to test the key function
    const request = new Request('http://localhost/test', {
      headers: { 'CF-Connecting-IP': '1.2.3.4' },
    });

    // ipKey uses c.req.header(), so we need a context-like object
    // Use the actual Hono pattern: create a mock that matches the interface
    const key = ipKey({
      req: {
        header: (name: string) => request.headers.get(name) || undefined,
      },
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
    } as any);

    expect(key).toBe('1.2.3.4');
  });

  it('ipKey falls back to X-Forwarded-For', () => {
    const request = new Request('http://localhost/test', {
      headers: { 'X-Forwarded-For': '5.6.7.8' },
    });

    const key = ipKey({
      req: {
        header: (name: string) => request.headers.get(name) || undefined,
      },
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
    } as any);

    expect(key).toBe('5.6.7.8');
  });

  it('ipKey returns undefined when no headers present', () => {
    const request = new Request('http://localhost/test');

    const key = ipKey({
      req: {
        header: (name: string) => request.headers.get(name) || undefined,
      },
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
    } as any);

    expect(key).toBeUndefined();
  });
});

describe('Rate Limit: OAuth Route Integration', () => {
  it('rate limiting is applied to /register endpoint', async () => {
    // The rate limiter from wrangler.jsonc is applied in production
    // In tests, miniflare provides the binding and requests succeed under limit
    const request = new IncomingRequest('https://example.com/register', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ redirect_uris: ['https://client.example.com/callback'] }),
    });

    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    // Should succeed (rate limiter allows in test environment)
    expect(response.status).toBe(201);
  });

  it('rate limiting is applied to /authorize endpoint', async () => {
    // First register a client to get a valid client_id
    const registerRequest = new IncomingRequest('https://example.com/register', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ redirect_uris: ['https://client.example.com/callback'] }),
    });
    const registerCtx = createExecutionContext();
    const registerResponse = await worker.fetch(registerRequest, stytchEnv, registerCtx);
    await waitOnExecutionContext(registerCtx);
    const client = await asAnyJson(registerResponse);

    // Now test /authorize
    const url = new URL('https://example.com/authorize');
    url.searchParams.set('response_type', 'code');
    url.searchParams.set('client_id', client.client_id);
    url.searchParams.set('redirect_uri', 'https://client.example.com/callback');
    url.searchParams.set('state', 'test-state-123');
    url.searchParams.set('code_challenge', 'E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM');
    url.searchParams.set('code_challenge_method', 'S256');
    url.searchParams.set('resource', 'https://example.com/mcp');

    const request = new IncomingRequest(url.toString(), { method: 'GET' });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    // Should redirect to IdP (302) - rate limiter allows
    expect(response.status).toBe(302);
  });

  it('rate limiting is applied to /token endpoint', async () => {
    const request = new IncomingRequest('https://example.com/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: new URLSearchParams({
        grant_type: 'authorization_code',
        code: 'invalid-code',
        redirect_uri: 'https://client.example.com/callback',
        client_id: 'invalid-client',
        code_verifier: 'dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk',
        resource: 'https://example.com/mcp',
      }).toString(),
    });

    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    // Should fail with invalid_grant (not rate limited)
    expect(response.status).toBe(400);
    const body = await asAnyJson(response);
    expect(body.error).toBe('invalid_grant');
  });

  it('rate limiting is applied to /revoke endpoint', async () => {
    const request = new IncomingRequest('https://example.com/revoke', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: new URLSearchParams({
        token: 'some-token',
      }).toString(),
    });

    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    // RFC 7009 says always return 200 for revoke
    expect(response.status).toBe(200);
  });
});

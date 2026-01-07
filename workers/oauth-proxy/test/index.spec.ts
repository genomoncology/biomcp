// TODO: use SELF to test
// eslint-disable-next-line @typescript-eslint/no-unused-vars
import { env, createExecutionContext, waitOnExecutionContext, SELF } from 'cloudflare:test';
import { describe, it, expect } from 'vitest';
import worker from '../src/index';

const IncomingRequest = Request<unknown, IncomingRequestCfProperties>;

describe('OAuth Discovery', () => {
  it('handles OPTIONS preflight for discovery endpoints', async () => {
    const request = new IncomingRequest('https://example.com/.well-known/oauth-authorization-server', {
      method: 'OPTIONS',
      headers: {
        Origin: 'https://client.example.com',
        'Access-Control-Request-Method': 'GET',
      },
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, env, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(204);
    expect(response.headers.get('Access-Control-Allow-Origin')).toBe('*');
    expect(response.headers.get('Access-Control-Allow-Methods')).toBe('GET, HEAD, OPTIONS');
    expect(response.headers.get('Access-Control-Max-Age')).toBe('86400');
  });

  it('returns CORS headers on GET requests to discovery endpoints', async () => {
    const request = new IncomingRequest('https://example.com/.well-known/oauth-protected-resource', {
      headers: { Origin: 'https://client.example.com' },
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, env, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(200);
    expect(response.headers.get('Access-Control-Allow-Origin')).toBe('*');
  });

  it('returns OAuth authorization server metadata', async () => {
    const request = new IncomingRequest('https://example.com/.well-known/oauth-authorization-server');
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, env, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(200);
    expect(response.headers.get('content-type')).toContain('application/json');

    const metadata = await response.json();
    expect(metadata).toMatchObject({
      issuer: 'https://example.com',
      authorization_endpoint: 'https://example.com/authorize',
      token_endpoint: 'https://example.com/token',
      registration_endpoint: 'https://example.com/register',
      response_types_supported: ['code'],
      grant_types_supported: ['authorization_code', 'refresh_token'],
      code_challenge_methods_supported: ['S256'],
    });
  });

  it('returns protected resource metadata', async () => {
    const request = new IncomingRequest('https://example.com/.well-known/oauth-protected-resource');
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, env, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(200);

    const metadata = await response.json();
    expect(metadata).toMatchObject({
      resource: 'https://example.com/mcp',
      authorization_servers: ['https://example.com'],
      bearer_methods_supported: ['header'],
    });
  });
});

describe('Error Handling', () => {
  it('returns 404 for unknown routes', async () => {
    const request = new IncomingRequest('https://example.com/nonexistent');
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, env, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(404);
    const body = await response.json();
    expect(body).toEqual({ error: 'not_found' });
  });
});

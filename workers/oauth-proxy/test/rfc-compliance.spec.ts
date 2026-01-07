/**
 * RFC Compliance Tests
 *
 * This file contains tests to verify compliance with the following RFCs:
 * - RFC 6749: OAuth 2.0 Authorization Framework
 * - RFC 6750: Bearer Token Usage
 * - RFC 7009: OAuth 2.0 Token Revocation
 * - RFC 7591: OAuth 2.0 Dynamic Client Registration
 * - RFC 7636: Proof Key for Code Exchange (PKCE)
 * - RFC 8414: OAuth 2.0 Authorization Server Metadata
 * - RFC 8707: Resource Indicators for OAuth 2.0
 * - RFC 9728: OAuth 2.0 Protected Resource Metadata
 */

import { env, createExecutionContext, waitOnExecutionContext, fetchMock } from 'cloudflare:test';
import { describe, it, expect, beforeAll, beforeEach, afterEach } from 'vitest';
import { decodeJwt } from 'jose';
import worker from '../src/index';
import { sha256Base64Url } from '../src/utils/crypto';
import { createStytchTestEnv } from './helpers/env';

// Env with Stytch identity provider for tests that exercise OAuth flows
const stytchEnv = createStytchTestEnv();

const IncomingRequest = Request<unknown, IncomingRequestCfProperties>;

// Helper to create a URL-encoded form body
function formBody(params: Record<string, string>): string {
  return new URLSearchParams(params).toString();
}

// Helper to get JSON from response (tests validate shape via assertions)
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const asAnyJson = (r: Response) => r.json<any>();

/**
 * Helper to extract OAuth error parameters from a redirect response.
 * Per RFC 6749 Section 4.1.2.1, errors after redirect_uri validation
 * are returned via redirect with error query parameters.
 */
function getRedirectError(response: Response): { error: string; error_description: string; state?: string } {
  const location = response.headers.get('location');
  if (!location) {
    throw new Error('No location header in redirect response');
  }
  const url = new URL(location);
  const error = url.searchParams.get('error');
  const errorDescription = url.searchParams.get('error_description');
  const state = url.searchParams.get('state');
  if (!error) {
    throw new Error(`No error parameter in redirect URL: ${location}`);
  }
  return {
    error,
    error_description: errorDescription || '',
    state: state || undefined,
  };
}

// Helper to compute code_challenge from code_verifier (S256)
const computeCodeChallenge = sha256Base64Url;

// Valid code_verifier per RFC 7636 (43-128 chars, unreserved characters)
const VALID_CODE_VERIFIER = 'dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk';

describe('RFC 8414: OAuth 2.0 Authorization Server Metadata', () => {
  it('returns required metadata fields', async () => {
    const request = new IncomingRequest('https://example.com/.well-known/oauth-authorization-server');
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(200);
    expect(response.headers.get('content-type')).toContain('application/json');

    const metadata = await asAnyJson(response);

    // RFC 8414 Section 2: Required fields
    expect(metadata.issuer).toBe('https://example.com');
    expect(metadata.authorization_endpoint).toBe('https://example.com/authorize');
    expect(metadata.token_endpoint).toBe('https://example.com/token');
    expect(metadata.response_types_supported).toContain('code');

    // RFC 8414 Section 2: Recommended/optional fields we support
    expect(metadata.registration_endpoint).toBe('https://example.com/register');
    expect(metadata.scopes_supported).toContain('mcp');
    expect(metadata.grant_types_supported).toContain('authorization_code');
    expect(metadata.token_endpoint_auth_methods_supported).toContain('none');
    expect(metadata.code_challenge_methods_supported).toContain('S256');
  });

  it('supports path-specific metadata endpoint', async () => {
    const request = new IncomingRequest('https://example.com/.well-known/oauth-authorization-server/mcp');
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(200);
    const metadata = await asAnyJson(response);
    expect(metadata.issuer).toBe('https://example.com');
  });
});

describe('RFC 9728: OAuth 2.0 Protected Resource Metadata', () => {
  it('returns required metadata fields', async () => {
    const request = new IncomingRequest('https://example.com/.well-known/oauth-protected-resource');
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(200);

    const metadata = await asAnyJson(response);

    // RFC 9728 Section 2: Required fields
    expect(metadata.resource).toBe('https://example.com/mcp');
    expect(metadata.authorization_servers).toContain('https://example.com');

    // RFC 9728 Section 2: Optional fields we support
    expect(metadata.bearer_methods_supported).toContain('header');
    expect(metadata.scopes_supported).toContain('mcp');
  });

  it('supports path-specific metadata endpoint', async () => {
    const request = new IncomingRequest('https://example.com/.well-known/oauth-protected-resource/mcp');
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(200);
    const metadata = await asAnyJson(response);
    expect(metadata.resource).toBe('https://example.com/mcp');
  });
});

describe('RFC 7591: OAuth 2.0 Dynamic Client Registration', () => {
  it('successfully registers a client with minimal parameters', async () => {
    const request = new IncomingRequest('https://example.com/register', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        redirect_uris: ['https://client.example.com/callback'],
      }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    // RFC 7591 Section 3.2.1: 201 Created on success
    expect(response.status).toBe(201);

    const client = await asAnyJson(response);

    // RFC 7591 Section 3.2.1: Required response fields
    expect(client.client_id).toBeDefined();
    expect(typeof client.client_id).toBe('string');

    // RFC 7591 Section 3.2.1: client_id_issued_at is RECOMMENDED
    expect(client.client_id_issued_at).toBeDefined();
    expect(typeof client.client_id_issued_at).toBe('number');

    // Should echo back registered values
    expect(client.redirect_uris).toEqual(['https://client.example.com/callback']);
    expect(client.token_endpoint_auth_method).toBe('none');
    expect(client.grant_types).toContain('authorization_code');
    expect(client.grant_types).toContain('refresh_token');
    expect(client.response_types).toContain('code');
  });

  it('registers a client with all supported parameters', async () => {
    const request = new IncomingRequest('https://example.com/register', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        redirect_uris: ['https://client.example.com/callback', 'https://client.example.com/callback2'],
        client_name: 'Test Client',
        client_uri: 'https://client.example.com',
        logo_uri: 'https://client.example.com/logo.png',
        scope: 'mcp',
        token_endpoint_auth_method: 'none',
      }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(201);

    const client = await asAnyJson(response);
    expect(client.client_name).toBe('Test Client');
    expect(client.client_uri).toBe('https://client.example.com');
    expect(client.logo_uri).toBe('https://client.example.com/logo.png');
    expect(client.scope).toBe('mcp');
  });

  it('rejects registration without redirect_uris', async () => {
    const request = new IncomingRequest('https://example.com/register', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({}),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    // RFC 7591 Section 3.2.2: invalid_client_metadata error
    expect(response.status).toBe(400);
    const error = await asAnyJson(response);
    expect(error.error).toBe('invalid_client_metadata');
  });

  it('rejects registration with empty redirect_uris array', async () => {
    const request = new IncomingRequest('https://example.com/register', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ redirect_uris: [] }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(400);
    const error = await asAnyJson(response);
    expect(error.error).toBe('invalid_client_metadata');
  });

  it('rejects registration with invalid redirect_uri format', async () => {
    const request = new IncomingRequest('https://example.com/register', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ redirect_uris: ['not-a-valid-url'] }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(400);
    const error = await asAnyJson(response);
    expect(error.error).toBe('invalid_client_metadata');
  });

  it('rejects registration with unsupported scope', async () => {
    const request = new IncomingRequest('https://example.com/register', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        redirect_uris: ['https://client.example.com/callback'],
        scope: 'admin', // Only 'mcp' is supported
      }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(400);
    const error = await asAnyJson(response);
    expect(error.error).toBe('invalid_client_metadata');
  });
});

describe('RFC 6749: OAuth 2.0 Authorization Endpoint', () => {
  let clientId: string;
  const redirectUri = 'https://client.example.com/callback';

  beforeEach(async () => {
    // Register a client for authorization tests
    const request = new IncomingRequest('https://example.com/register', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ redirect_uris: [redirectUri] }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);
    const client = await asAnyJson(response);
    clientId = client.client_id;
  });

  it('rejects request without response_type (RFC 6749 Section 4.1.1)', async () => {
    const codeChallenge = await computeCodeChallenge(VALID_CODE_VERIFIER);
    const url = new URL('https://example.com/authorize');
    url.searchParams.set('client_id', clientId);
    url.searchParams.set('redirect_uri', redirectUri);
    url.searchParams.set('state', 'test-state');
    url.searchParams.set('code_challenge', codeChallenge);
    url.searchParams.set('code_challenge_method', 'S256');
    // Missing response_type

    const request = new IncomingRequest(url.toString());
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(400);
    const error = await asAnyJson(response);
    expect(error.error).toBe('invalid_request');
    expect(error.error_description).toContain('response_type');
  });

  it('rejects unsupported response_type (RFC 6749 Section 4.1.2.1)', async () => {
    const codeChallenge = await computeCodeChallenge(VALID_CODE_VERIFIER);
    const url = new URL('https://example.com/authorize');
    url.searchParams.set('response_type', 'token'); // Implicit flow not supported
    url.searchParams.set('client_id', clientId);
    url.searchParams.set('redirect_uri', redirectUri);
    url.searchParams.set('state', 'test-state');
    url.searchParams.set('code_challenge', codeChallenge);
    url.searchParams.set('code_challenge_method', 'S256');

    const request = new IncomingRequest(url.toString());
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(400);
    const error = await asAnyJson(response);
    expect(error.error).toBe('unsupported_response_type');
  });

  it('rejects request without client_id (RFC 6749 Section 4.1.1)', async () => {
    const codeChallenge = await computeCodeChallenge(VALID_CODE_VERIFIER);
    const url = new URL('https://example.com/authorize');
    url.searchParams.set('response_type', 'code');
    url.searchParams.set('redirect_uri', redirectUri);
    url.searchParams.set('state', 'test-state');
    url.searchParams.set('code_challenge', codeChallenge);
    url.searchParams.set('code_challenge_method', 'S256');
    // Missing client_id

    const request = new IncomingRequest(url.toString());
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(400);
    const error = await asAnyJson(response);
    expect(error.error).toBe('invalid_request');
    expect(error.error_description).toContain('client_id');
  });

  it('rejects unknown client_id (RFC 6749 Section 4.1.2.1)', async () => {
    const codeChallenge = await computeCodeChallenge(VALID_CODE_VERIFIER);
    const url = new URL('https://example.com/authorize');
    url.searchParams.set('response_type', 'code');
    url.searchParams.set('client_id', 'unknown-client-id');
    url.searchParams.set('redirect_uri', redirectUri);
    url.searchParams.set('state', 'test-state');
    url.searchParams.set('code_challenge', codeChallenge);
    url.searchParams.set('code_challenge_method', 'S256');

    const request = new IncomingRequest(url.toString());
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(400);
    const error = await asAnyJson(response);
    expect(error.error).toBe('invalid_client');
  });

  it('rejects request without redirect_uri (RFC 6749 Section 4.1.1)', async () => {
    const codeChallenge = await computeCodeChallenge(VALID_CODE_VERIFIER);
    const url = new URL('https://example.com/authorize');
    url.searchParams.set('response_type', 'code');
    url.searchParams.set('client_id', clientId);
    url.searchParams.set('state', 'test-state');
    url.searchParams.set('code_challenge', codeChallenge);
    url.searchParams.set('code_challenge_method', 'S256');
    // Missing redirect_uri

    const request = new IncomingRequest(url.toString());
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(400);
    const error = await asAnyJson(response);
    expect(error.error).toBe('invalid_request');
    expect(error.error_description).toContain('redirect_uri');
  });

  it('rejects mismatched redirect_uri (RFC 6749 Section 3.1.2.3)', async () => {
    const codeChallenge = await computeCodeChallenge(VALID_CODE_VERIFIER);
    const url = new URL('https://example.com/authorize');
    url.searchParams.set('response_type', 'code');
    url.searchParams.set('client_id', clientId);
    url.searchParams.set('redirect_uri', 'https://attacker.com/callback');
    url.searchParams.set('state', 'test-state');
    url.searchParams.set('code_challenge', codeChallenge);
    url.searchParams.set('code_challenge_method', 'S256');

    const request = new IncomingRequest(url.toString());
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(400);
    const error = await asAnyJson(response);
    expect(error.error).toBe('invalid_request');
    expect(error.error_description).toContain('redirect_uri');
  });

  it('rejects request without state parameter', async () => {
    const codeChallenge = await computeCodeChallenge(VALID_CODE_VERIFIER);
    const url = new URL('https://example.com/authorize');
    url.searchParams.set('response_type', 'code');
    url.searchParams.set('client_id', clientId);
    url.searchParams.set('redirect_uri', redirectUri);
    url.searchParams.set('code_challenge', codeChallenge);
    url.searchParams.set('code_challenge_method', 'S256');
    // Missing state

    const request = new IncomingRequest(url.toString());
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    // RFC 6749 Section 4.1.2.1: After redirect_uri is validated, errors MUST be
    // returned via redirect with error query parameters
    expect(response.status).toBe(302);
    const error = getRedirectError(response);
    expect(error.error).toBe('invalid_request');
    expect(error.error_description).toContain('state');
  });

  it('rejects invalid state parameter format', async () => {
    const codeChallenge = await computeCodeChallenge(VALID_CODE_VERIFIER);
    const url = new URL('https://example.com/authorize');
    url.searchParams.set('response_type', 'code');
    url.searchParams.set('client_id', clientId);
    url.searchParams.set('redirect_uri', redirectUri);
    url.searchParams.set('state', 'invalid state with spaces!');
    url.searchParams.set('code_challenge', codeChallenge);
    url.searchParams.set('code_challenge_method', 'S256');

    const request = new IncomingRequest(url.toString());
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    // RFC 6749 Section 4.1.2.1: After redirect_uri is validated, errors MUST be
    // returned via redirect with error query parameters
    expect(response.status).toBe(302);
    const error = getRedirectError(response);
    expect(error.error).toBe('invalid_request');
    expect(error.error_description).toContain('state');
  });

  it('accepts state with base64 and URL-safe characters (.=~)', async () => {
    const codeChallenge = await computeCodeChallenge(VALID_CODE_VERIFIER);
    const url = new URL('https://example.com/authorize');
    url.searchParams.set('response_type', 'code');
    url.searchParams.set('client_id', clientId);
    url.searchParams.set('redirect_uri', redirectUri);
    // State with base64url chars, padding, and safe chars: eyJmb28iOiJiYXIifQ==
    url.searchParams.set('state', 'eyJmb28iOiJiYXIifQ==.test~value_123-abc');
    url.searchParams.set('code_challenge', codeChallenge);
    url.searchParams.set('code_challenge_method', 'S256');
    url.searchParams.set('resource', 'https://example.com/mcp');

    const request = new IncomingRequest(url.toString());
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    // Should redirect to IdP (302) not reject with 400
    expect(response.status).toBe(302);
  });

  it('rejects unsupported scope', async () => {
    const codeChallenge = await computeCodeChallenge(VALID_CODE_VERIFIER);
    const url = new URL('https://example.com/authorize');
    url.searchParams.set('response_type', 'code');
    url.searchParams.set('client_id', clientId);
    url.searchParams.set('redirect_uri', redirectUri);
    url.searchParams.set('state', 'test-state');
    url.searchParams.set('code_challenge', codeChallenge);
    url.searchParams.set('code_challenge_method', 'S256');
    url.searchParams.set('scope', 'admin'); // Only 'mcp' is supported

    const request = new IncomingRequest(url.toString());
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    // RFC 6749 Section 4.1.2.1: After redirect_uri is validated, errors MUST be
    // returned via redirect with error query parameters
    expect(response.status).toBe(302);
    const error = getRedirectError(response);
    expect(error.error).toBe('invalid_scope');
    expect(error.state).toBe('test-state');
  });
});

describe('RFC 7636: Proof Key for Code Exchange (PKCE)', () => {
  let clientId: string;
  const redirectUri = 'https://client.example.com/callback';

  beforeEach(async () => {
    const request = new IncomingRequest('https://example.com/register', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ redirect_uris: [redirectUri] }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);
    const client = await asAnyJson(response);
    clientId = client.client_id;
  });

  it('rejects request without code_challenge (PKCE required)', async () => {
    const url = new URL('https://example.com/authorize');
    url.searchParams.set('response_type', 'code');
    url.searchParams.set('client_id', clientId);
    url.searchParams.set('redirect_uri', redirectUri);
    url.searchParams.set('state', 'test-state');
    // Missing code_challenge

    const request = new IncomingRequest(url.toString());
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    // RFC 6749 Section 4.1.2.1: After redirect_uri is validated, errors MUST be
    // returned via redirect with error query parameters
    expect(response.status).toBe(302);
    const error = getRedirectError(response);
    expect(error.error).toBe('invalid_request');
    expect(error.error_description).toContain('code_challenge');
    expect(error.state).toBe('test-state');
  });

  it('rejects unsupported code_challenge_method (RFC 7636 Section 4.2)', async () => {
    const codeChallenge = await computeCodeChallenge(VALID_CODE_VERIFIER);
    const url = new URL('https://example.com/authorize');
    url.searchParams.set('response_type', 'code');
    url.searchParams.set('client_id', clientId);
    url.searchParams.set('redirect_uri', redirectUri);
    url.searchParams.set('state', 'test-state');
    url.searchParams.set('code_challenge', codeChallenge);
    url.searchParams.set('code_challenge_method', 'plain'); // Only S256 is supported

    const request = new IncomingRequest(url.toString());
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    // RFC 6749 Section 4.1.2.1: After redirect_uri is validated, errors MUST be
    // returned via redirect with error query parameters
    expect(response.status).toBe(302);
    const error = getRedirectError(response);
    expect(error.error).toBe('invalid_request');
    expect(error.error_description).toContain('code_challenge_method');
    expect(error.state).toBe('test-state');
  });

  it('accepts valid S256 code_challenge_method', async () => {
    const codeChallenge = await computeCodeChallenge(VALID_CODE_VERIFIER);
    const url = new URL('https://example.com/authorize');
    url.searchParams.set('response_type', 'code');
    url.searchParams.set('client_id', clientId);
    url.searchParams.set('redirect_uri', redirectUri);
    url.searchParams.set('state', 'test-state');
    url.searchParams.set('code_challenge', codeChallenge);
    url.searchParams.set('code_challenge_method', 'S256');
    url.searchParams.set('resource', 'https://example.com/mcp'); // RFC 8707

    const request = new IncomingRequest(url.toString());
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    // Should redirect to IdP (Stytch), not return error
    expect(response.status).toBe(302);
  });
});

describe('RFC 6749: OAuth 2.0 Token Endpoint', () => {
  let clientId: string;
  const redirectUri = 'https://client.example.com/callback';

  beforeEach(async () => {
    const request = new IncomingRequest('https://example.com/register', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ redirect_uris: [redirectUri] }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);
    const client = await asAnyJson(response);
    clientId = client.client_id;
  });

  it('rejects request with missing grant_type', async () => {
    const request = new IncomingRequest('https://example.com/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formBody({
        code: 'test-code',
        redirect_uri: redirectUri,
        client_id: clientId,
        code_verifier: VALID_CODE_VERIFIER,
      }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(400);
    const error = await asAnyJson(response);
    expect(error.error).toBe('invalid_request');
  });

  it('rejects unsupported grant_type (RFC 6749 Section 5.2)', async () => {
    const request = new IncomingRequest('https://example.com/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formBody({
        grant_type: 'client_credentials',
        code: 'test-code',
        redirect_uri: redirectUri,
        client_id: clientId,
        code_verifier: VALID_CODE_VERIFIER,
      }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(400);
    const error = await asAnyJson(response);
    expect(error.error).toBe('invalid_request');
  });

  it('rejects request with missing code', async () => {
    const request = new IncomingRequest('https://example.com/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formBody({
        grant_type: 'authorization_code',
        redirect_uri: redirectUri,
        client_id: clientId,
        code_verifier: VALID_CODE_VERIFIER,
      }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(400);
    const error = await asAnyJson(response);
    expect(error.error).toBe('invalid_request');
  });

  it('rejects request with missing client_id', async () => {
    const request = new IncomingRequest('https://example.com/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formBody({
        grant_type: 'authorization_code',
        code: 'test-code',
        redirect_uri: redirectUri,
        code_verifier: VALID_CODE_VERIFIER,
      }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(400);
    const error = await asAnyJson(response);
    expect(error.error).toBe('invalid_request');
  });

  it('rejects request with missing redirect_uri', async () => {
    const request = new IncomingRequest('https://example.com/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formBody({
        grant_type: 'authorization_code',
        code: 'test-code',
        client_id: clientId,
        code_verifier: VALID_CODE_VERIFIER,
      }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(400);
    const error = await asAnyJson(response);
    expect(error.error).toBe('invalid_request');
  });

  it('rejects request with invalid/expired authorization code (RFC 6749 Section 5.2)', async () => {
    const request = new IncomingRequest('https://example.com/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formBody({
        grant_type: 'authorization_code',
        code: 'invalid-code',
        redirect_uri: redirectUri,
        client_id: clientId,
        code_verifier: VALID_CODE_VERIFIER,
      }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(400);
    const error = await asAnyJson(response);
    expect(error.error).toBe('invalid_grant');
  });

  it('rejects code_verifier that is too short (RFC 7636 Section 4.1)', async () => {
    const request = new IncomingRequest('https://example.com/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formBody({
        grant_type: 'authorization_code',
        code: 'test-code',
        redirect_uri: redirectUri,
        client_id: clientId,
        code_verifier: 'too-short', // Less than 43 characters
      }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(400);
    const error = await asAnyJson(response);
    expect(error.error).toBe('invalid_request');
  });

  it('rejects code_verifier that is too long (RFC 7636 Section 4.1)', async () => {
    const request = new IncomingRequest('https://example.com/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formBody({
        grant_type: 'authorization_code',
        code: 'test-code',
        redirect_uri: redirectUri,
        client_id: clientId,
        code_verifier: 'a'.repeat(129), // More than 128 characters
      }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(400);
    const error = await asAnyJson(response);
    expect(error.error).toBe('invalid_request');
  });

  it('rejects code_verifier with invalid characters (RFC 7636 Section 4.1)', async () => {
    const request = new IncomingRequest('https://example.com/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formBody({
        grant_type: 'authorization_code',
        code: 'test-code',
        redirect_uri: redirectUri,
        client_id: clientId,
        // Contains invalid characters (spaces, special chars)
        code_verifier: 'invalid verifier with spaces and $pecial chars!!!',
      }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(400);
    const error = await asAnyJson(response);
    expect(error.error).toBe('invalid_request');
  });

  it('accepts code_verifier with valid RFC 7636 characters', async () => {
    // Valid characters: [A-Z] / [a-z] / [0-9] / "-" / "." / "_" / "~"
    const validVerifier = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnop0123456789-._~';
    const request = new IncomingRequest('https://example.com/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formBody({
        grant_type: 'authorization_code',
        code: 'test-code',
        redirect_uri: redirectUri,
        client_id: clientId,
        code_verifier: validVerifier,
      }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    // Will fail with invalid_grant (code doesn't exist), not invalid_request
    expect(response.status).toBe(400);
    const error = await asAnyJson(response);
    expect(error.error).toBe('invalid_grant');
  });
});

describe('RFC 6750: Bearer Token Usage', () => {
  it('returns WWW-Authenticate without error when no token provided (Section 3)', async () => {
    const request = new IncomingRequest('https://example.com/mcp', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({}),
      // No Authorization header
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(401);

    const wwwAuth = response.headers.get('WWW-Authenticate');
    expect(wwwAuth).toBeDefined();
    expect(wwwAuth).toContain('Bearer');
    expect(wwwAuth).toContain('realm=');
    // Should NOT contain error when token is missing
    expect(wwwAuth).not.toContain('error=');

    const body = await asAnyJson(response);
    expect(body.error).toBe('invalid_request');
    expect(body.error_description).toBeDefined();
  });

  it('returns WWW-Authenticate with error for invalid token (Section 3.1)', async () => {
    const request = new IncomingRequest('https://example.com/mcp', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: 'Bearer invalid-token',
      },
      body: JSON.stringify({}),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(401);

    const wwwAuth = response.headers.get('WWW-Authenticate');
    expect(wwwAuth).toBeDefined();
    expect(wwwAuth).toContain('Bearer');
    expect(wwwAuth).toContain('realm=');
    // Should contain error when token is invalid
    expect(wwwAuth).toContain('error="invalid_token"');

    const body = await asAnyJson(response);
    expect(body.error).toBe('invalid_token');
    expect(body.error_description).toBeDefined();
  });

  it('rejects malformed Authorization header', async () => {
    const request = new IncomingRequest('https://example.com/mcp', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: 'Basic dXNlcjpwYXNz', // Not Bearer
      },
      body: JSON.stringify({}),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(401);
    const wwwAuth = response.headers.get('WWW-Authenticate');
    expect(wwwAuth).toContain('Bearer');
  });
});

describe('OIDC Features Removed', () => {
  it('does not expose /userinfo endpoint', async () => {
    const request = new IncomingRequest('https://example.com/userinfo');
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    // Should return 404 since OIDC is not supported
    expect(response.status).toBe(404);
  });

  it('does not advertise OIDC scopes in authorization server metadata', async () => {
    const request = new IncomingRequest('https://example.com/.well-known/oauth-authorization-server');
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    const metadata = await asAnyJson(response);

    // Should not contain OIDC-specific scopes
    expect(metadata.scopes_supported).not.toContain('openid');
    expect(metadata.scopes_supported).not.toContain('profile');
    expect(metadata.scopes_supported).not.toContain('email');
    expect(metadata.scopes_supported).not.toContain('offline_access');

    // Should contain MCP scope
    expect(metadata.scopes_supported).toContain('mcp');
  });

  it('advertises refresh_token grant type', async () => {
    const request = new IncomingRequest('https://example.com/.well-known/oauth-authorization-server');
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    const metadata = await asAnyJson(response);

    // Should advertise both authorization_code and refresh_token
    expect(metadata.grant_types_supported).toContain('authorization_code');
    expect(metadata.grant_types_supported).toContain('refresh_token');
  });
});

describe('RFC 6749 Section 6: Refreshing an Access Token', () => {
  const redirectUri = 'https://client.example.com/callback';
  const testResource = 'https://example.com/mcp'; // RFC 8707: resource indicator
  let clientId: string;

  // Helper to store a refresh token directly in KV for testing
  async function storeRefreshToken(
    token: string,
    data: { sub: string; client_id: string; email?: string; scope: string; resource?: string[] },
  ): Promise<void> {
    const key = await sha256Base64Url(token);
    await env.OAUTH_KV.put(
      `refresh_token:${key}`,
      JSON.stringify({
        ...data,
        resource: data.resource ?? [testResource], // RFC 8707: default resource
      }),
      {
        expirationTtl: 3600,
      },
    );
  }

  beforeEach(async () => {
    // Register a client for refresh token tests
    const request = new IncomingRequest('https://example.com/register', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ redirect_uris: [redirectUri] }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);
    const client = await asAnyJson(response);
    clientId = client.client_id;
  });

  it('successfully exchanges refresh token for new access token', async () => {
    const refreshToken = crypto.randomUUID();
    await storeRefreshToken(refreshToken, {
      sub: 'user-123',
      client_id: clientId,
      email: 'test@example.com',
      scope: 'mcp',
    });

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
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(200);
    expect(response.headers.get('Cache-Control')).toBe('no-store');
    expect(response.headers.get('Pragma')).toBe('no-cache');

    const tokenResponse = await asAnyJson(response);
    expect(tokenResponse.access_token).toBeDefined();
    expect(tokenResponse.token_type).toBe('Bearer');
    expect(tokenResponse.expires_in).toBeGreaterThan(0);
    expect(tokenResponse.refresh_token).toBeDefined();
    expect(tokenResponse.scope).toBe('mcp');
    // New refresh token should be different (rotation)
    expect(tokenResponse.refresh_token).not.toBe(refreshToken);
  });

  it('invalidates old refresh token after rotation (single-use)', async () => {
    const refreshToken = crypto.randomUUID();
    await storeRefreshToken(refreshToken, {
      sub: 'user-123',
      client_id: clientId,
      scope: 'mcp',
    });

    // First request - should succeed
    const request1 = new IncomingRequest('https://example.com/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formBody({
        grant_type: 'refresh_token',
        refresh_token: refreshToken,
        client_id: clientId,
      }),
    });
    const ctx1 = createExecutionContext();
    const response1 = await worker.fetch(request1, env, ctx1);
    await waitOnExecutionContext(ctx1);
    expect(response1.status).toBe(200);

    // Second request with same token - should fail (token was rotated/invalidated)
    const request2 = new IncomingRequest('https://example.com/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formBody({
        grant_type: 'refresh_token',
        refresh_token: refreshToken,
        client_id: clientId,
      }),
    });
    const ctx2 = createExecutionContext();
    const response2 = await worker.fetch(request2, env, ctx2);
    await waitOnExecutionContext(ctx2);

    expect(response2.status).toBe(400);
    const error = await asAnyJson(response2);
    expect(error.error).toBe('invalid_grant');
  });

  it('rejects invalid refresh token', async () => {
    const request = new IncomingRequest('https://example.com/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formBody({
        grant_type: 'refresh_token',
        refresh_token: 'invalid-token-that-does-not-exist',
        client_id: clientId,
      }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(400);
    const error = await asAnyJson(response);
    expect(error.error).toBe('invalid_grant');
    expect(error.error_description).toContain('Invalid or expired');
  });

  it('rejects refresh token issued to different client', async () => {
    // Register a second client
    const registerRequest = new IncomingRequest('https://example.com/register', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ redirect_uris: ['https://other.example.com/callback'] }),
    });
    const registerCtx = createExecutionContext();
    const registerResponse = await worker.fetch(registerRequest, env, registerCtx);
    await waitOnExecutionContext(registerCtx);
    const otherClient = await asAnyJson(registerResponse);

    // Store refresh token for the OTHER client
    const refreshToken = crypto.randomUUID();
    await storeRefreshToken(refreshToken, {
      sub: 'user-123',
      client_id: otherClient.client_id,
      scope: 'mcp',
    });

    // Try to use the refresh token with the FIRST client
    const request = new IncomingRequest('https://example.com/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formBody({
        grant_type: 'refresh_token',
        refresh_token: refreshToken,
        client_id: clientId, // Different from token's client_id
      }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(400);
    const error = await asAnyJson(response);
    expect(error.error).toBe('invalid_grant');
    expect(error.error_description).toContain('not issued to this client');
  });

  it('rejects refresh request for unknown client', async () => {
    const refreshToken = crypto.randomUUID();
    await storeRefreshToken(refreshToken, {
      sub: 'user-123',
      client_id: 'non-existent-client',
      scope: 'mcp',
    });

    const request = new IncomingRequest('https://example.com/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formBody({
        grant_type: 'refresh_token',
        refresh_token: refreshToken,
        client_id: 'non-existent-client',
      }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(400);
    const error = await asAnyJson(response);
    expect(error.error).toBe('invalid_client');
    expect(error.error_description).toContain('not found or inactive');
  });

  it('rejects request without refresh_token parameter', async () => {
    const request = new IncomingRequest('https://example.com/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formBody({
        grant_type: 'refresh_token',
        client_id: clientId,
        // Missing refresh_token
      }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(400);
    const error = await asAnyJson(response);
    expect(error.error).toBe('invalid_request');
  });

  it('rejects request without client_id parameter', async () => {
    const refreshToken = crypto.randomUUID();
    await storeRefreshToken(refreshToken, {
      sub: 'user-123',
      client_id: clientId,
      scope: 'mcp',
    });

    const request = new IncomingRequest('https://example.com/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formBody({
        grant_type: 'refresh_token',
        refresh_token: refreshToken,
        // Missing client_id
      }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(400);
    const error = await asAnyJson(response);
    expect(error.error).toBe('invalid_request');
  });

  it('rejects scope that exceeds original grant', async () => {
    const refreshToken = crypto.randomUUID();
    await storeRefreshToken(refreshToken, {
      sub: 'user-123',
      client_id: clientId,
      scope: 'mcp', // Original scope
    });

    const request = new IncomingRequest('https://example.com/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formBody({
        grant_type: 'refresh_token',
        refresh_token: refreshToken,
        client_id: clientId,
        scope: 'mcp admin', // Requesting more than original
      }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(400);
    const error = await asAnyJson(response);
    expect(error.error).toBe('invalid_scope');
    expect(error.error_description).toContain('exceeds original grant');
  });

  it('preserves user identity through refresh flow', async () => {
    const refreshToken = crypto.randomUUID();
    const originalSub = 'user-456';
    const originalEmail = 'preserved@example.com';

    await storeRefreshToken(refreshToken, {
      sub: originalSub,
      client_id: clientId,
      email: originalEmail,
      scope: 'mcp',
    });

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
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(200);
    const tokenResponse = await asAnyJson(response);

    // The new refresh token should preserve the user identity
    // We verify this by checking the new refresh token can be used
    const newRefreshToken = tokenResponse.refresh_token;

    const request2 = new IncomingRequest('https://example.com/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formBody({
        grant_type: 'refresh_token',
        refresh_token: newRefreshToken,
        client_id: clientId,
      }),
    });
    const ctx2 = createExecutionContext();
    const response2 = await worker.fetch(request2, env, ctx2);
    await waitOnExecutionContext(ctx2);

    expect(response2.status).toBe(200);
  });

  it('issues usable refresh token from auth code exchange (end-to-end)', async () => {
    // This test proves the refresh token issued by /token is correctly stored and reusable
    // without requiring IdP mocking - we seed an auth code directly in KV

    const authCode = crypto.randomUUID();
    const codeVerifier = VALID_CODE_VERIFIER;
    const codeChallenge = await sha256Base64Url(codeVerifier);

    // Seed auth code in KV (matching what /callback would store)
    // Note: auth codes use the raw code as key (not hashed)
    await env.OAUTH_KV.put(
      `auth-codes:${authCode}`,
      JSON.stringify({
        sub: 'integration-test-user',
        email: 'integration@example.com',
        code_challenge: codeChallenge,
        client_id: clientId,
        redirect_uri: redirectUri,
        scope: 'mcp',
        resource: [testResource], // RFC 8707: resource indicator
      }),
      { expirationTtl: 300 },
    );

    // Step 1: Exchange auth code for tokens (include resource per RFC 8707)
    const tokenRequest = new IncomingRequest('https://example.com/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formBody({
        grant_type: 'authorization_code',
        code: authCode,
        redirect_uri: redirectUri,
        client_id: clientId,
        code_verifier: codeVerifier,
        resource: testResource, // RFC 8707: must match auth code
      }),
    });
    const tokenCtx = createExecutionContext();
    const tokenResponse = await worker.fetch(tokenRequest, env, tokenCtx);
    await waitOnExecutionContext(tokenCtx);

    expect(tokenResponse.status).toBe(200);
    const tokens = await asAnyJson(tokenResponse);
    expect(tokens.access_token).toBeDefined();
    expect(tokens.refresh_token).toBeDefined();

    // Step 2: Use the refresh token to get new tokens
    const refreshRequest = new IncomingRequest('https://example.com/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formBody({
        grant_type: 'refresh_token',
        refresh_token: tokens.refresh_token,
        client_id: clientId,
      }),
    });
    const refreshCtx = createExecutionContext();
    const refreshResponse = await worker.fetch(refreshRequest, env, refreshCtx);
    await waitOnExecutionContext(refreshCtx);

    expect(refreshResponse.status).toBe(200);
    const refreshedTokens = await asAnyJson(refreshResponse);
    expect(refreshedTokens.access_token).toBeDefined();
    expect(refreshedTokens.refresh_token).toBeDefined();
    // Rotation: new refresh token should be different
    expect(refreshedTokens.refresh_token).not.toBe(tokens.refresh_token);
  });
});

describe('RFC 8707: Resource Indicators for OAuth 2.0', () => {
  const redirectUri = 'https://client.example.com/callback';
  const validResource = 'https://example.com/mcp';
  let clientId: string;

  beforeEach(async () => {
    // Register a client for resource indicator tests
    const request = new IncomingRequest('https://example.com/register', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ redirect_uris: [redirectUri] }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);
    const client = await asAnyJson(response);
    clientId = client.client_id;
  });

  describe('Authorization Endpoint', () => {
    it('rejects request without resource parameter', async () => {
      const codeChallenge = await computeCodeChallenge(VALID_CODE_VERIFIER);
      const url = new URL('https://example.com/authorize');
      url.searchParams.set('response_type', 'code');
      url.searchParams.set('client_id', clientId);
      url.searchParams.set('redirect_uri', redirectUri);
      url.searchParams.set('state', 'test-state');
      url.searchParams.set('code_challenge', codeChallenge);
      url.searchParams.set('code_challenge_method', 'S256');
      // Missing resource parameter

      const request = new IncomingRequest(url.toString());
      const ctx = createExecutionContext();
      const response = await worker.fetch(request, stytchEnv, ctx);
      await waitOnExecutionContext(ctx);

      // RFC 6749 Section 4.1.2.1: After redirect_uri is validated, errors MUST be
      // returned via redirect with error query parameters
      expect(response.status).toBe(302);
      const error = getRedirectError(response);
      expect(error.error).toBe('invalid_target');
      expect(error.error_description).toContain('resource');
      expect(error.state).toBe('test-state');
    });

    it('accepts valid resource parameter', async () => {
      const codeChallenge = await computeCodeChallenge(VALID_CODE_VERIFIER);
      const url = new URL('https://example.com/authorize');
      url.searchParams.set('response_type', 'code');
      url.searchParams.set('client_id', clientId);
      url.searchParams.set('redirect_uri', redirectUri);
      url.searchParams.set('state', 'test-state');
      url.searchParams.set('code_challenge', codeChallenge);
      url.searchParams.set('code_challenge_method', 'S256');
      url.searchParams.set('resource', validResource);

      const request = new IncomingRequest(url.toString());
      const ctx = createExecutionContext();
      const response = await worker.fetch(request, stytchEnv, ctx);
      await waitOnExecutionContext(ctx);

      // Should redirect to IdP, not return error
      expect(response.status).toBe(302);
    });

    it('rejects resource that is not the MCP endpoint', async () => {
      const codeChallenge = await computeCodeChallenge(VALID_CODE_VERIFIER);
      const url = new URL('https://example.com/authorize');
      url.searchParams.set('response_type', 'code');
      url.searchParams.set('client_id', clientId);
      url.searchParams.set('redirect_uri', redirectUri);
      url.searchParams.set('state', 'test-state');
      url.searchParams.set('code_challenge', codeChallenge);
      url.searchParams.set('code_challenge_method', 'S256');
      url.searchParams.set('resource', 'https://example.com/api'); // Not /mcp

      const request = new IncomingRequest(url.toString());
      const ctx = createExecutionContext();
      const response = await worker.fetch(request, stytchEnv, ctx);
      await waitOnExecutionContext(ctx);

      // Resource must be ${origin}/mcp - other resources are rejected
      expect(response.status).toBe(302);
      const error = getRedirectError(response);
      expect(error.error).toBe('invalid_target');
      expect(error.error_description).toContain('MCP endpoint');
      expect(error.state).toBe('test-state');
    });

    it('rejects resource with fragment (RFC 8707 Section 2)', async () => {
      const codeChallenge = await computeCodeChallenge(VALID_CODE_VERIFIER);
      const url = new URL('https://example.com/authorize');
      url.searchParams.set('response_type', 'code');
      url.searchParams.set('client_id', clientId);
      url.searchParams.set('redirect_uri', redirectUri);
      url.searchParams.set('state', 'test-state');
      url.searchParams.set('code_challenge', codeChallenge);
      url.searchParams.set('code_challenge_method', 'S256');
      url.searchParams.set('resource', 'https://example.com/mcp#fragment');

      const request = new IncomingRequest(url.toString());
      const ctx = createExecutionContext();
      const response = await worker.fetch(request, stytchEnv, ctx);
      await waitOnExecutionContext(ctx);

      // RFC 6749 Section 4.1.2.1: After redirect_uri is validated, errors MUST be
      // returned via redirect with error query parameters
      expect(response.status).toBe(302);
      const error = getRedirectError(response);
      expect(error.error).toBe('invalid_target');
      expect(error.state).toBe('test-state');
    });

    it('rejects non-URI resource', async () => {
      const codeChallenge = await computeCodeChallenge(VALID_CODE_VERIFIER);
      const url = new URL('https://example.com/authorize');
      url.searchParams.set('response_type', 'code');
      url.searchParams.set('client_id', clientId);
      url.searchParams.set('redirect_uri', redirectUri);
      url.searchParams.set('state', 'test-state');
      url.searchParams.set('code_challenge', codeChallenge);
      url.searchParams.set('code_challenge_method', 'S256');
      url.searchParams.set('resource', 'not-a-valid-uri');

      const request = new IncomingRequest(url.toString());
      const ctx = createExecutionContext();
      const response = await worker.fetch(request, stytchEnv, ctx);
      await waitOnExecutionContext(ctx);

      // RFC 6749 Section 4.1.2.1: After redirect_uri is validated, errors MUST be
      // returned via redirect with error query parameters
      expect(response.status).toBe(302);
      const error = getRedirectError(response);
      expect(error.error).toBe('invalid_target');
      expect(error.state).toBe('test-state');
    });
  });

  describe('Token Endpoint', () => {
    it('rejects token request without resource parameter', async () => {
      const authCode = crypto.randomUUID();
      const codeVerifier = VALID_CODE_VERIFIER;
      const codeChallenge = await sha256Base64Url(codeVerifier);

      // Seed auth code with resource
      await env.OAUTH_KV.put(
        `auth-codes:${authCode}`,
        JSON.stringify({
          sub: 'test-user',
          code_challenge: codeChallenge,
          client_id: clientId,
          redirect_uri: redirectUri,
          scope: 'mcp',
          resource: [validResource],
        }),
        { expirationTtl: 300 },
      );

      const request = new IncomingRequest('https://example.com/token', {
        method: 'POST',
        headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        body: formBody({
          grant_type: 'authorization_code',
          code: authCode,
          redirect_uri: redirectUri,
          client_id: clientId,
          code_verifier: codeVerifier,
          // Missing resource parameter
        }),
      });
      const ctx = createExecutionContext();
      const response = await worker.fetch(request, stytchEnv, ctx);
      await waitOnExecutionContext(ctx);

      expect(response.status).toBe(400);
      const error = await asAnyJson(response);
      expect(error.error).toBe('invalid_target');
    });

    it('rejects resource from different origin at token endpoint', async () => {
      const authCode = crypto.randomUUID();
      const codeVerifier = VALID_CODE_VERIFIER;
      const codeChallenge = await sha256Base64Url(codeVerifier);

      // Seed auth code with valid resource
      await env.OAUTH_KV.put(
        `auth-codes:${authCode}`,
        JSON.stringify({
          sub: 'test-user',
          code_challenge: codeChallenge,
          client_id: clientId,
          redirect_uri: redirectUri,
          scope: 'mcp',
          resource: [validResource],
        }),
        { expirationTtl: 300 },
      );

      const request = new IncomingRequest('https://example.com/token', {
        method: 'POST',
        headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        body: formBody({
          grant_type: 'authorization_code',
          code: authCode,
          redirect_uri: redirectUri,
          client_id: clientId,
          code_verifier: codeVerifier,
          resource: 'https://different-resource.com/mcp', // Different origin
        }),
      });
      const ctx = createExecutionContext();
      const response = await worker.fetch(request, stytchEnv, ctx);
      await waitOnExecutionContext(ctx);

      // Strict resource validation: only ${origin}/mcp is allowed
      expect(response.status).toBe(400);
      const error = await asAnyJson(response);
      expect(error.error).toBe('invalid_target');
      expect(error.error_description).toContain('MCP endpoint');
    });

    it('accepts subset of authorized resources', async () => {
      const authCode = crypto.randomUUID();
      const codeVerifier = VALID_CODE_VERIFIER;
      const codeChallenge = await sha256Base64Url(codeVerifier);

      // Seed auth code with multiple resources
      await env.OAUTH_KV.put(
        `auth-codes:${authCode}`,
        JSON.stringify({
          sub: 'test-user',
          code_challenge: codeChallenge,
          client_id: clientId,
          redirect_uri: redirectUri,
          scope: 'mcp',
          resource: ['https://example.com/mcp', 'https://example.com/api'],
        }),
        { expirationTtl: 300 },
      );

      // Request token for only one of the authorized resources
      const request = new IncomingRequest('https://example.com/token', {
        method: 'POST',
        headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        body: formBody({
          grant_type: 'authorization_code',
          code: authCode,
          redirect_uri: redirectUri,
          client_id: clientId,
          code_verifier: codeVerifier,
          resource: 'https://example.com/mcp', // Subset of authorized
        }),
      });
      const ctx = createExecutionContext();
      const response = await worker.fetch(request, stytchEnv, ctx);
      await waitOnExecutionContext(ctx);

      expect(response.status).toBe(200);
      const tokens = await asAnyJson(response);
      expect(tokens.access_token).toBeDefined();
    });
  });

  describe('Refresh Token with Resource Narrowing', () => {
    it('allows narrowing resources on refresh', async () => {
      const refreshToken = crypto.randomUUID();
      const refreshTokenKey = await sha256Base64Url(refreshToken);

      // Store refresh token with multiple resources
      await env.OAUTH_KV.put(
        `refresh_token:${refreshTokenKey}`,
        JSON.stringify({
          sub: 'user-123',
          client_id: clientId,
          scope: 'mcp',
          resource: ['https://example.com/mcp', 'https://example.com/api'],
        }),
        { expirationTtl: 3600 },
      );

      // Request new token for only one resource
      const request = new IncomingRequest('https://example.com/token', {
        method: 'POST',
        headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        body: formBody({
          grant_type: 'refresh_token',
          refresh_token: refreshToken,
          client_id: clientId,
          resource: 'https://example.com/mcp', // Narrowed from original
        }),
      });
      const ctx = createExecutionContext();
      const response = await worker.fetch(request, stytchEnv, ctx);
      await waitOnExecutionContext(ctx);

      expect(response.status).toBe(200);
      const tokens = await asAnyJson(response);
      expect(tokens.access_token).toBeDefined();
    });

    it('rejects resource not in original refresh token grant', async () => {
      const refreshToken = crypto.randomUUID();
      const refreshTokenKey = await sha256Base64Url(refreshToken);

      // Store refresh token with specific resource
      await env.OAUTH_KV.put(
        `refresh_token:${refreshTokenKey}`,
        JSON.stringify({
          sub: 'user-123',
          client_id: clientId,
          scope: 'mcp',
          resource: [validResource],
        }),
        { expirationTtl: 3600 },
      );

      // Request new token for different resource
      const request = new IncomingRequest('https://example.com/token', {
        method: 'POST',
        headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        body: formBody({
          grant_type: 'refresh_token',
          refresh_token: refreshToken,
          client_id: clientId,
          resource: 'https://different-resource.com/mcp',
        }),
      });
      const ctx = createExecutionContext();
      const response = await worker.fetch(request, stytchEnv, ctx);
      await waitOnExecutionContext(ctx);

      expect(response.status).toBe(400);
      const error = await asAnyJson(response);
      expect(error.error).toBe('invalid_target');
    });

    it('uses original resources when none specified in refresh', async () => {
      const refreshToken = crypto.randomUUID();
      const refreshTokenKey = await sha256Base64Url(refreshToken);

      // Store refresh token with resource
      await env.OAUTH_KV.put(
        `refresh_token:${refreshTokenKey}`,
        JSON.stringify({
          sub: 'user-123',
          client_id: clientId,
          scope: 'mcp',
          resource: [validResource],
        }),
        { expirationTtl: 3600 },
      );

      // Request without resource parameter
      const request = new IncomingRequest('https://example.com/token', {
        method: 'POST',
        headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        body: formBody({
          grant_type: 'refresh_token',
          refresh_token: refreshToken,
          client_id: clientId,
          // No resource parameter - should use original
        }),
      });
      const ctx = createExecutionContext();
      const response = await worker.fetch(request, stytchEnv, ctx);
      await waitOnExecutionContext(ctx);

      expect(response.status).toBe(200);
      const tokens = await asAnyJson(response);
      expect(tokens.access_token).toBeDefined();
    });
  });
});

describe('JWT Claims: jti (JWT ID)', () => {
  let clientId: string;
  const redirectUri = 'https://client.example.com/callback';
  const validResource = 'https://example.com/mcp';

  beforeEach(async () => {
    // Register a client
    const request = new IncomingRequest('https://example.com/register', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ redirect_uris: [redirectUri] }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);
    const client = await asAnyJson(response);
    clientId = client.client_id;
  });

  it('includes jti claim in access token from authorization code grant', async () => {
    // Store an auth code
    const authCode = crypto.randomUUID();
    const codeChallenge = await sha256Base64Url(VALID_CODE_VERIFIER);
    await env.OAUTH_KV.put(
      `auth-codes:${authCode}`,
      JSON.stringify({
        sub: 'user-123',
        code_challenge: codeChallenge,
        client_id: clientId,
        redirect_uri: redirectUri,
        scope: 'mcp',
        resource: [validResource],
        email: 'test@example.com',
      }),
      { expirationTtl: 300 },
    );

    // Exchange auth code for tokens
    const request = new IncomingRequest('https://example.com/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formBody({
        grant_type: 'authorization_code',
        code: authCode,
        redirect_uri: redirectUri,
        client_id: clientId,
        code_verifier: VALID_CODE_VERIFIER,
        resource: validResource,
      }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(200);
    const tokens = await asAnyJson(response);
    expect(tokens.access_token).toBeDefined();

    // Decode and verify jti claim
    const claims = decodeJwt(tokens.access_token);
    expect(claims.jti).toBeDefined();
    expect(typeof claims.jti).toBe('string');
    // Verify it's a valid UUID format
    expect(claims.jti).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i);
  });

  it('includes jti claim in access token from refresh token grant', async () => {
    // Store a refresh token
    const refreshToken = crypto.randomUUID();
    const refreshTokenKey = await sha256Base64Url(refreshToken);
    await env.OAUTH_KV.put(
      `refresh_token:${refreshTokenKey}`,
      JSON.stringify({
        sub: 'user-123',
        client_id: clientId,
        email: 'test@example.com',
        scope: 'mcp',
        resource: [validResource],
      }),
      { expirationTtl: 3600 },
    );

    // Request token refresh
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
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(200);
    const tokens = await asAnyJson(response);
    expect(tokens.access_token).toBeDefined();

    // Decode and verify jti claim
    const claims = decodeJwt(tokens.access_token);
    expect(claims.jti).toBeDefined();
    expect(typeof claims.jti).toBe('string');
    // Verify it's a valid UUID format
    expect(claims.jti).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i);
  });

  it('generates unique jti for each token', async () => {
    // Store a refresh token
    const refreshToken = crypto.randomUUID();
    const refreshTokenKey = await sha256Base64Url(refreshToken);
    await env.OAUTH_KV.put(
      `refresh_token:${refreshTokenKey}`,
      JSON.stringify({
        sub: 'user-123',
        client_id: clientId,
        email: 'test@example.com',
        scope: 'mcp',
        resource: [validResource],
      }),
      { expirationTtl: 3600 },
    );

    // First refresh
    const request1 = new IncomingRequest('https://example.com/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formBody({
        grant_type: 'refresh_token',
        refresh_token: refreshToken,
        client_id: clientId,
      }),
    });
    const ctx1 = createExecutionContext();
    const response1 = await worker.fetch(request1, env, ctx1);
    await waitOnExecutionContext(ctx1);
    const tokens1 = await asAnyJson(response1);
    const claims1 = decodeJwt(tokens1.access_token);

    // Get the new refresh token for second request
    const newRefreshToken = tokens1.refresh_token;

    // Second refresh with the rotated token
    const request2 = new IncomingRequest('https://example.com/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formBody({
        grant_type: 'refresh_token',
        refresh_token: newRefreshToken,
        client_id: clientId,
      }),
    });
    const ctx2 = createExecutionContext();
    const response2 = await worker.fetch(request2, env, ctx2);
    await waitOnExecutionContext(ctx2);
    const tokens2 = await asAnyJson(response2);
    const claims2 = decodeJwt(tokens2.access_token);

    // Verify jti values are different
    expect(claims1.jti).not.toBe(claims2.jti);
  });
});

describe('RFC 7009: Token Revocation', () => {
  let clientId: string;
  const redirectUri = 'https://client.example.com/callback';
  const validResource = 'https://example.com/mcp';

  beforeEach(async () => {
    // Register a client
    const request = new IncomingRequest('https://example.com/register', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ redirect_uris: [redirectUri] }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);
    const client = await asAnyJson(response);
    clientId = client.client_id;
  });

  it('advertises revocation_endpoint in discovery metadata', async () => {
    const request = new IncomingRequest('https://example.com/.well-known/oauth-authorization-server');
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(200);
    const metadata = await asAnyJson(response);
    expect(metadata.revocation_endpoint).toBe('https://example.com/revoke');
  });

  it('returns 200 when revoking valid access token', async () => {
    // Create an access token in KV
    const accessToken = 'test-access-token-' + crypto.randomUUID();
    const tokenHash = await sha256Base64Url(accessToken);
    await env.OAUTH_KV.put(
      `token:${tokenHash}`,
      JSON.stringify({
        sub: 'user-123',
        client_id: clientId,
        scope: 'mcp',
        resource: [validResource],
      }),
      { expirationTtl: 3600 },
    );

    // Verify token exists
    const beforeRevoke = await env.OAUTH_KV.get(`token:${tokenHash}`);
    expect(beforeRevoke).not.toBeNull();

    // Revoke it
    const request = new IncomingRequest('https://example.com/revoke', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formBody({ token: accessToken }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(200);
    expect(response.headers.get('Cache-Control')).toBe('no-store');
    expect(response.headers.get('Pragma')).toBe('no-cache');

    // Verify token was deleted
    const afterRevoke = await env.OAUTH_KV.get(`token:${tokenHash}`);
    expect(afterRevoke).toBeNull();
  });

  it('returns 200 when revoking valid refresh token', async () => {
    // Create a refresh token in KV
    const refreshToken = 'test-refresh-token-' + crypto.randomUUID();
    const tokenHash = await sha256Base64Url(refreshToken);
    await env.OAUTH_KV.put(
      `refresh_token:${tokenHash}`,
      JSON.stringify({
        sub: 'user-123',
        client_id: clientId,
        scope: 'mcp',
        resource: [validResource],
      }),
      { expirationTtl: 3600 },
    );

    // Verify token exists
    const beforeRevoke = await env.OAUTH_KV.get(`refresh_token:${tokenHash}`);
    expect(beforeRevoke).not.toBeNull();

    // Revoke it
    const request = new IncomingRequest('https://example.com/revoke', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formBody({ token: refreshToken }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(200);

    // Verify token was deleted
    const afterRevoke = await env.OAUTH_KV.get(`refresh_token:${tokenHash}`);
    expect(afterRevoke).toBeNull();
  });

  it('returns 200 for invalid/nonexistent token (RFC 7009 §2.2.1)', async () => {
    // Per RFC 7009 §2.2.1, the server responds 200 even for invalid tokens
    // to prevent attackers from probing for valid tokens
    const request = new IncomingRequest('https://example.com/revoke', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formBody({ token: 'this-token-does-not-exist-anywhere' }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(200);
  });

  it('returns 200 for missing token parameter (RFC 7009 §2.2.1)', async () => {
    // Per RFC 7009 §2.2.1, always return 200 to prevent information leakage
    const request = new IncomingRequest('https://example.com/revoke', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formBody({}),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(200);
  });

  it('only deletes from access token storage when hint is access_token', async () => {
    // Create tokens in both storages
    const token = 'test-token-' + crypto.randomUUID();
    const tokenHash = await sha256Base64Url(token);

    await env.OAUTH_KV.put(
      `token:${tokenHash}`,
      JSON.stringify({ sub: 'user-123', client_id: clientId, scope: 'mcp', resource: [validResource] }),
      { expirationTtl: 3600 },
    );
    await env.OAUTH_KV.put(
      `refresh_token:${tokenHash}`,
      JSON.stringify({ sub: 'user-123', client_id: clientId, scope: 'mcp', resource: [validResource] }),
      { expirationTtl: 3600 },
    );

    // Revoke with access_token hint
    const request = new IncomingRequest('https://example.com/revoke', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formBody({ token, token_type_hint: 'access_token' }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(200);

    // Access token should be deleted, refresh token should remain
    const accessAfter = await env.OAUTH_KV.get(`token:${tokenHash}`);
    const refreshAfter = await env.OAUTH_KV.get(`refresh_token:${tokenHash}`);
    expect(accessAfter).toBeNull();
    expect(refreshAfter).not.toBeNull();
  });

  it('only deletes from refresh token storage when hint is refresh_token', async () => {
    // Create tokens in both storages
    const token = 'test-token-' + crypto.randomUUID();
    const tokenHash = await sha256Base64Url(token);

    await env.OAUTH_KV.put(
      `token:${tokenHash}`,
      JSON.stringify({ sub: 'user-123', client_id: clientId, scope: 'mcp', resource: [validResource] }),
      { expirationTtl: 3600 },
    );
    await env.OAUTH_KV.put(
      `refresh_token:${tokenHash}`,
      JSON.stringify({ sub: 'user-123', client_id: clientId, scope: 'mcp', resource: [validResource] }),
      { expirationTtl: 3600 },
    );

    // Revoke with refresh_token hint
    const request = new IncomingRequest('https://example.com/revoke', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formBody({ token, token_type_hint: 'refresh_token' }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(200);

    // Refresh token should be deleted, access token should remain
    const accessAfter = await env.OAUTH_KV.get(`token:${tokenHash}`);
    const refreshAfter = await env.OAUTH_KV.get(`refresh_token:${tokenHash}`);
    expect(accessAfter).not.toBeNull();
    expect(refreshAfter).toBeNull();
  });

  it('deletes from both storages when no hint provided', async () => {
    // Create tokens in both storages
    const token = 'test-token-' + crypto.randomUUID();
    const tokenHash = await sha256Base64Url(token);

    await env.OAUTH_KV.put(
      `token:${tokenHash}`,
      JSON.stringify({ sub: 'user-123', client_id: clientId, scope: 'mcp', resource: [validResource] }),
      { expirationTtl: 3600 },
    );
    await env.OAUTH_KV.put(
      `refresh_token:${tokenHash}`,
      JSON.stringify({ sub: 'user-123', client_id: clientId, scope: 'mcp', resource: [validResource] }),
      { expirationTtl: 3600 },
    );

    // Revoke without hint
    const request = new IncomingRequest('https://example.com/revoke', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formBody({ token }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(200);

    // Both should be deleted
    const accessAfter = await env.OAUTH_KV.get(`token:${tokenHash}`);
    const refreshAfter = await env.OAUTH_KV.get(`refresh_token:${tokenHash}`);
    expect(accessAfter).toBeNull();
    expect(refreshAfter).toBeNull();
  });

  it('revoked refresh token cannot be used for token refresh', async () => {
    // Create a refresh token
    const refreshToken = crypto.randomUUID();
    const tokenHash = await sha256Base64Url(refreshToken);
    await env.OAUTH_KV.put(
      `refresh_token:${tokenHash}`,
      JSON.stringify({
        sub: 'user-123',
        client_id: clientId,
        scope: 'mcp',
        resource: [validResource],
      }),
      { expirationTtl: 3600 },
    );

    // Revoke it
    const revokeRequest = new IncomingRequest('https://example.com/revoke', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formBody({ token: refreshToken, token_type_hint: 'refresh_token' }),
    });
    const revokeCtx = createExecutionContext();
    await worker.fetch(revokeRequest, env, revokeCtx);
    await waitOnExecutionContext(revokeCtx);

    // Try to use the revoked refresh token
    const refreshRequest = new IncomingRequest('https://example.com/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formBody({
        grant_type: 'refresh_token',
        refresh_token: refreshToken,
        client_id: clientId,
      }),
    });
    const refreshCtx = createExecutionContext();
    const response = await worker.fetch(refreshRequest, env, refreshCtx);
    await waitOnExecutionContext(refreshCtx);

    expect(response.status).toBe(400);
    const error = await asAnyJson(response);
    expect(error.error).toBe('invalid_grant');
  });
});

describe('RFC 6749 §4.1.2.1: Callback Error Redirects', () => {
  let clientId: string;
  const redirectUri = 'https://client.example.com/callback';
  const validResource = 'https://example.com/mcp';

  beforeAll(() => {
    fetchMock.activate();
    // Don't disable net connect - internal routes need real fetch
  });

  beforeEach(async () => {
    // Register a client
    const request = new IncomingRequest('https://example.com/register', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ redirect_uris: [redirectUri] }),
    });
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);
    const client = await asAnyJson(response);
    clientId = client.client_id;
  });

  afterEach(() => {
    fetchMock.assertNoPendingInterceptors();
  });

  it('returns JSON error when tx parameter is invalid format', async () => {
    // Per RFC 6749 §4.1.2.1: errors before redirect_uri is validated
    // should NOT redirect - return error to user directly
    // tx must match [a-zA-Z0-9\-_]+ and be <= 32 chars; use '@' to fail validation
    const request = new IncomingRequest('https://example.com/callback?token=stytch-token&tx=invalid@format');
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    // Should return JSON, not redirect
    expect(response.status).toBe(400);
    const error = await asAnyJson(response);
    expect(error.error).toBe('invalid_request');
    expect(error.error_description).toContain('Invalid tx parameter');
  });

  it('returns JSON error when tx is not found in storage', async () => {
    // Per RFC 6749 §4.1.2.1: errors before redirect_uri is validated
    // should NOT redirect - return error to user directly
    // tx format is UUID without hyphens (32 hex chars)
    const validTx = crypto.randomUUID().replace(/-/g, '');
    const request = new IncomingRequest(`https://example.com/callback?token=stytch-token&tx=${validTx}`);
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    // Should return JSON, not redirect (redirect_uri unknown)
    expect(response.status).toBe(400);
    const error = await asAnyJson(response);
    expect(error.error).toBe('invalid_request');
    expect(error.error_description).toContain('Invalid or expired transaction');
  });

  it('redirects with access_denied on authentication failure', async () => {
    // Seed OAuth tx data in KV (simulating /authorize flow)
    // tx format is UUID without hyphens (32 hex chars)
    const tx = crypto.randomUUID().replace(/-/g, '');
    const originalState = 'client-original-state-abc123';
    const codeChallenge = await sha256Base64Url(VALID_CODE_VERIFIER);

    await env.OAUTH_KV.put(
      `oauth_tx:${tx}`,
      JSON.stringify({
        client_id: clientId,
        redirect_uri: redirectUri,
        code_challenge: codeChallenge,
        code_challenge_method: 'S256',
        original_state: originalState,
        scope: 'mcp',
        resource: [validResource],
      }),
      { expirationTtl: 600 },
    );

    // Mock Stytch to return authentication error
    fetchMock
      .get('https://test.stytch.com')
      .intercept({
        method: 'POST',
        path: '/v1/oauth/authenticate',
      })
      .reply(401, JSON.stringify({ status_code: 401, error_message: 'Invalid token' }));

    // Call callback with the seeded tx
    const request = new IncomingRequest(`https://example.com/callback?token=invalid-stytch-token&tx=${tx}`);
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    // Per RFC 6749 §4.1.2.1: once redirect_uri is known, errors redirect
    expect(response.status).toBe(302);
    const errorParams = getRedirectError(response);
    expect(errorParams.error).toBe('access_denied');
    expect(errorParams.error_description).toContain('Authentication failed');
    expect(errorParams.state).toBe(originalState);
  });

  it('includes state parameter in error redirect when available', async () => {
    // Seed OAuth tx data with a specific state
    // tx format is UUID without hyphens (32 hex chars)
    const tx = crypto.randomUUID().replace(/-/g, '');
    const originalState = 'my-unique-state-xyz789';
    const codeChallenge = await sha256Base64Url(VALID_CODE_VERIFIER);

    await env.OAUTH_KV.put(
      `oauth_tx:${tx}`,
      JSON.stringify({
        client_id: clientId,
        redirect_uri: redirectUri,
        code_challenge: codeChallenge,
        code_challenge_method: 'S256',
        original_state: originalState,
        scope: 'mcp',
        resource: [validResource],
      }),
      { expirationTtl: 600 },
    );

    // Mock Stytch to return error
    fetchMock
      .get('https://test.stytch.com')
      .intercept({
        method: 'POST',
        path: '/v1/oauth/authenticate',
      })
      .reply(500, 'Internal Server Error');

    const request = new IncomingRequest(`https://example.com/callback?token=test-token&tx=${tx}`);
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(302);
    const location = response.headers.get('location');
    expect(location).toBeDefined();

    const redirectUrl = new URL(location!);
    expect(redirectUrl.origin).toBe('https://client.example.com');
    expect(redirectUrl.searchParams.get('state')).toBe(originalState);
  });

  it('propagates IdP error response to client via redirect (RFC 6749 §4.1.2.1)', async () => {
    // This tests the case where the IdP returns an error in the callback URL
    // (e.g., user denies access), not a token. Per RFC 6749 §4.1.2.1, we must
    // propagate this error to the client by redirecting with error params.
    const tx = crypto.randomUUID().replace(/-/g, '');
    const originalState = 'client-state-idp-error';
    const codeChallenge = await sha256Base64Url(VALID_CODE_VERIFIER);

    await env.OAUTH_KV.put(
      `oauth_tx:${tx}`,
      JSON.stringify({
        client_id: clientId,
        redirect_uri: redirectUri,
        code_challenge: codeChallenge,
        code_challenge_method: 'S256',
        original_state: originalState,
        scope: 'mcp',
        resource: [validResource],
      }),
      { expirationTtl: 600 },
    );

    // IdP returns error in callback URL (e.g., user denied access at IdP)
    // Using Stytch's error param format
    const request = new IncomingRequest(
      `https://example.com/callback?stytch_error=access_denied&stytch_error_description=User%20cancelled%20authentication&tx=${tx}`,
    );
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    // RFC 6749 §4.1.2.1: redirect to client with error params
    expect(response.status).toBe(302);
    const errorParams = getRedirectError(response);
    expect(errorParams.error).toBe('access_denied');
    expect(errorParams.error_description).toBe('User cancelled authentication');
    expect(errorParams.state).toBe(originalState);
  });

  it('provides default error_description when IdP omits it', async () => {
    const tx = crypto.randomUUID().replace(/-/g, '');
    const originalState = 'client-state-no-desc';
    const codeChallenge = await sha256Base64Url(VALID_CODE_VERIFIER);

    await env.OAUTH_KV.put(
      `oauth_tx:${tx}`,
      JSON.stringify({
        client_id: clientId,
        redirect_uri: redirectUri,
        code_challenge: codeChallenge,
        code_challenge_method: 'S256',
        original_state: originalState,
        scope: 'mcp',
        resource: [validResource],
      }),
      { expirationTtl: 600 },
    );

    // IdP returns error without description
    const request = new IncomingRequest(`https://example.com/callback?stytch_error=server_error&tx=${tx}`);
    const ctx = createExecutionContext();
    const response = await worker.fetch(request, stytchEnv, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(302);
    const errorParams = getRedirectError(response);
    expect(errorParams.error).toBe('server_error');
    expect(errorParams.error_description).toBe('Authorization failed: server_error'); // Fallback includes error code
    expect(errorParams.state).toBe(originalState);
  });
});

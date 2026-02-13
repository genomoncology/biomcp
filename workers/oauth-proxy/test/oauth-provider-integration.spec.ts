/**
 * OAuth Provider Integration Tests
 *
 * Tests the full callback error flow with the StandardOAuthProvider.
 * This complements the Stytch-based tests in rfc-compliance.spec.ts
 * by verifying the standard OAuth error parameter format works end-to-end.
 */

import { env, createExecutionContext, waitOnExecutionContext } from 'cloudflare:test';
import { describe, it, expect, beforeAll, beforeEach } from 'vitest';
import { Hono } from 'hono';
import type { ParsedEnv, StandardOAuthEnv } from '../src/env';
import { StandardOAuthProvider } from '../src/identity/providers/oauth';
import { createOAuthRoutes } from '../src/routes/oauth';
import { createLogger } from '../src/utils/logger';
import { sha256Base64Url } from '../src/utils/crypto';
import type { CorsVariables } from '../src/middleware/cors';
import { corsMiddleware } from '../src/middleware/cors';

const IncomingRequest = Request<unknown, IncomingRequestCfProperties>;

// Valid code_verifier per RFC 7636 (43-128 chars, unreserved characters)
const VALID_CODE_VERIFIER = 'dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk';

// Test environment for OAuth provider
const testOAuthEnv: StandardOAuthEnv = {
  OAUTH_CLIENT_ID: 'test-oauth-client-id',
  OAUTH_CLIENT_SECRET: 'test-oauth-client-secret',
  OAUTH_AUTHORIZATION_URL: 'https://idp.example.com/authorize',
  OAUTH_TOKEN_URL: 'https://idp.example.com/oauth/token',
  OAUTH_USERINFO_URL: 'https://idp.example.com/userinfo',
  OAUTH_SCOPES: ['openid', 'email', 'profile'],
};

/**
 * Helper to extract OAuth error parameters from a redirect response.
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

describe('OAuth Provider: Callback Error Redirect Integration', () => {
  let app: Hono<{ Bindings: ParsedEnv; Variables: CorsVariables }>;
  let clientId: string;
  const redirectUri = 'https://client.example.com/callback';
  const validResource = 'https://example.com/mcp';

  beforeAll(async () => {
    // Create app with StandardOAuthProvider
    const logger = createLogger(false, 'test');
    const identityProvider = new StandardOAuthProvider(testOAuthEnv, logger.child('oauth'));

    app = new Hono<{ Bindings: ParsedEnv; Variables: CorsVariables }>();
    app.use('*', corsMiddleware());
    app.route('/', createOAuthRoutes({ logger, identityProvider }));
  });

  beforeEach(async () => {
    // Register a client for testing
    const request = new IncomingRequest('https://example.com/register', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ redirect_uris: [redirectUri] }),
    });
    const ctx = createExecutionContext();
    const response = await app.fetch(request, env, ctx);
    await waitOnExecutionContext(ctx);
    const client = (await response.json()) as { client_id: string };
    clientId = client.client_id;
  });

  it('propagates standard OAuth error response to client via redirect (RFC 6749 §4.1.2.1)', async () => {
    // Seed OAuth tx data in KV (simulating /authorize flow)
    // For standard OAuth, tx is passed via the state parameter
    const tx = crypto.randomUUID().replace(/-/g, '');
    const originalState = 'client-state-oauth-error';
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

    // IdP returns error in callback URL using standard OAuth format:
    // - error: OAuth error code
    // - error_description: human-readable description
    // - state: our tx (passed through by IdP)
    const request = new IncomingRequest(
      `https://example.com/callback?error=access_denied&error_description=The%20user%20denied%20the%20request&state=${tx}`,
    );
    const ctx = createExecutionContext();
    const response = await app.fetch(request, env, ctx);
    await waitOnExecutionContext(ctx);

    // RFC 6749 §4.1.2.1: redirect to client with error params
    expect(response.status).toBe(302);
    const errorParams = getRedirectError(response);
    expect(errorParams.error).toBe('access_denied');
    expect(errorParams.error_description).toBe('The user denied the request');
    expect(errorParams.state).toBe(originalState);
  });

  it('handles IdP server_error with fallback description', async () => {
    const tx = crypto.randomUUID().replace(/-/g, '');
    const originalState = 'client-state-server-error';
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
    const request = new IncomingRequest(`https://example.com/callback?error=temporarily_unavailable&state=${tx}`);
    const ctx = createExecutionContext();
    const response = await app.fetch(request, env, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(302);
    const errorParams = getRedirectError(response);
    expect(errorParams.error).toBe('temporarily_unavailable');
    // Fallback description includes error code for observability
    expect(errorParams.error_description).toBe('Authorization failed: temporarily_unavailable');
    expect(errorParams.state).toBe(originalState);
  });

  it('handles invalid_request error from IdP', async () => {
    const tx = crypto.randomUUID().replace(/-/g, '');
    const originalState = 'client-state-invalid-request';
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

    // IdP returns invalid_request error
    const request = new IncomingRequest(
      `https://example.com/callback?error=invalid_request&error_description=Missing%20required%20parameter&state=${tx}`,
    );
    const ctx = createExecutionContext();
    const response = await app.fetch(request, env, ctx);
    await waitOnExecutionContext(ctx);

    expect(response.status).toBe(302);
    const errorParams = getRedirectError(response);
    expect(errorParams.error).toBe('invalid_request');
    expect(errorParams.error_description).toBe('Missing required parameter');
    expect(errorParams.state).toBe(originalState);
  });
});

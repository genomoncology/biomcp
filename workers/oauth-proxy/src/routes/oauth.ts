import type { Context } from 'hono';
import { JWTPayload, SignJWT } from 'jose';
import * as v from 'valibot';
import { TOKEN_EXPIRY_SECONDS } from '../constants';
import type { IdentityProvider } from '../identity';
import { rateLimitMiddleware } from '../middleware/rate-limit';
import {
  createAccessTokenStorage,
  createAuthCodeStorage,
  createClientStorage,
  createOAuthTxStorage,
  createRefreshTokenStorage,
  OAuthTxSchema,
} from '../storage';
import { generateSecureToken, sha256Base64Url, timingSafeEqual } from '../utils/crypto';
import { extractErrorMessage } from '../utils/error';
import { createHono, getOrigin, HonoEnv } from '../utils/hono';
import type { Logger } from '../utils/logger';

// Schema for OAuth state parameter validation
// Allows alphanumeric, URL-safe base64 chars (-_), base64 padding (=), and common safe chars (.~)
const StateSchema = v.pipe(v.string(), v.minLength(1), v.maxLength(128), v.regex(/^[a-zA-Z0-9\-_.=~]+$/));

// RFC 8707 Section 2: resource MUST be an absolute URI without fragment
const ResourceSchema = v.pipe(
  v.string(),
  v.url(),
  v.check((uri) => !uri.includes('#'), 'Resource URI must not contain a fragment'),
);

// Schema for tx parameter validation (server-generated correlation ID)
const TxSchema = v.pipe(v.string(), v.minLength(1), v.maxLength(32), v.regex(/^[a-zA-Z0-9\-_]+$/));

/**
 * Generate a transaction ID for IdP correlation (128 bits entropy).
 * Used internally between Gateway ↔ IdP, separate from client's state.
 */
function generateTx(): string {
  return crypto.randomUUID().replace(/-/g, '');
}

/**
 * Create a redirect response with OAuth error parameters.
 * Per RFC 6749 §4.1.2.1, once redirect_uri and client are validated,
 * authorization errors MUST be returned by redirecting to redirect_uri
 * with error parameters in the query string. Errors before validation
 * (missing/invalid client_id or redirect_uri) return JSON and MUST NOT redirect.
 */
function redirectError(redirectUri: string, state: string | null, error: string, description: string): Response {
  const url = new URL(redirectUri);
  url.searchParams.set('error', error);
  url.searchParams.set('error_description', description);
  if (state) {
    url.searchParams.set('state', state);
  }
  return Response.redirect(url.toString(), 302);
}

/**
 * Create a token endpoint error response with RFC 6749 §5.2 cache headers.
 * Token endpoint errors SHOULD include Cache-Control: no-store and Pragma: no-cache.
 */
function tokenError(c: Context<HonoEnv>, error: string, description: string, status: 400 | 401 | 500 = 400): Response {
  return c.json({ error, error_description: description }, status, { 'Cache-Control': 'no-store', Pragma: 'no-cache' });
}

// Schema for OAuth token request validation
// RFC 7636 Section 4.1: code_verifier uses characters [A-Z] / [a-z] / [0-9] / "-" / "." / "_" / "~"
const CodeVerifierSchema = v.pipe(
  v.string(),
  v.minLength(43),
  v.maxLength(128),
  v.regex(/^[A-Za-z0-9\-._~]+$/, 'code_verifier contains invalid characters'),
);

const TokenRequestSchema = v.object({
  grant_type: v.literal('authorization_code'),
  code: v.pipe(v.string(), v.minLength(1)),
  redirect_uri: v.pipe(v.string(), v.url()),
  client_id: v.pipe(v.string(), v.minLength(1)),
  code_verifier: CodeVerifierSchema,
});

// Schema for refresh token request validation (RFC 6749 Section 6)
// Note: scope uses v.nullish() because formData.get() returns null for missing fields
const RefreshTokenRequestSchema = v.object({
  grant_type: v.literal('refresh_token'),
  refresh_token: v.pipe(v.string(), v.minLength(1)),
  client_id: v.pipe(v.string(), v.minLength(1)),
  scope: v.nullish(v.string()),
});

// Schema for dynamic client registration (RFC 7591)
// Note: scope is restricted to 'mcp' to match scopes_supported in discovery metadata
const ClientRegistrationSchema = v.object({
  redirect_uris: v.pipe(v.array(v.pipe(v.string(), v.url())), v.minLength(1)),
  client_name: v.optional(v.string()),
  client_uri: v.optional(v.pipe(v.string(), v.url())),
  logo_uri: v.optional(v.pipe(v.string(), v.url())),
  scope: v.optional(v.literal('mcp')),
  token_endpoint_auth_method: v.optional(v.literal('none')), // Public clients only
});

export interface OAuthRouteDeps {
  logger: Logger;
  identityProvider: IdentityProvider;
}

/**
 * Handle refresh token grant (RFC 6749 Section 6)
 */
async function handleRefreshTokenGrant(c: Context<HonoEnv>, formData: FormData, logger: Logger): Promise<Response> {
  // 1. Validate request
  const rawData = {
    grant_type: formData.get('grant_type'),
    refresh_token: formData.get('refresh_token'),
    client_id: formData.get('client_id'),
    scope: formData.get('scope'),
  };
  const result = v.safeParse(RefreshTokenRequestSchema, rawData);
  if (!result.success) {
    const issue = result.issues[0];
    logger.debug('Refresh token request validation failed', { error: issue.message });
    return tokenError(c, 'invalid_request', 'Invalid refresh token request');
  }

  const { refresh_token: refreshToken, client_id: clientId, scope: requestedScope } = result.output;

  // 2. Look up refresh token (hash first)
  const refreshTokenKey = await sha256Base64Url(refreshToken);
  const refreshTokenStorage = createRefreshTokenStorage(c.env.OAUTH_KV);
  const refreshTokenData = await refreshTokenStorage.get(refreshTokenKey);

  if (!refreshTokenData) {
    logger.debug('Invalid or expired refresh token', { clientId });
    return tokenError(c, 'invalid_grant', 'Invalid or expired refresh token');
  }

  // 3. Verify client_id matches (RFC 6749 requirement)
  if (refreshTokenData.client_id !== clientId) {
    logger.debug('Refresh token client_id mismatch', { expected: refreshTokenData.client_id, received: clientId });
    return tokenError(c, 'invalid_grant', 'Refresh token was not issued to this client');
  }

  // 4. Verify client still exists and is active
  const clientStorage = createClientStorage(c.env.OAUTH_KV);
  const client = await clientStorage.get(clientId);
  if (!client || client.status !== 'active') {
    logger.debug('Client not found or inactive during refresh', { clientId });
    return tokenError(c, 'invalid_client', 'Client not found or inactive');
  }

  // 5. Validate scope per RFC 6749 Section 6:
  // - Requested scope MUST NOT include any scope not originally granted (lines 2597-2602)
  // - If a new refresh token is issued, its scope MUST be identical to the old one (lines 2660-2662)
  // Current implementation: Only 'mcp' scope is supported, so we require exact match.
  // This satisfies both requirements. To support scope narrowing in the future,
  // we would need to store effectiveScope in access token but originalScope in refresh token.
  const originalScope = refreshTokenData.scope;
  const effectiveScope = requestedScope || originalScope;
  if (requestedScope && requestedScope !== originalScope) {
    logger.debug('Invalid scope in refresh request', { requestedScope, originalScope });
    return tokenError(c, 'invalid_scope', 'Requested scope exceeds original grant');
  }

  // 5b. RFC 8707: Parse and validate optional resource parameter(s)
  // If provided, must be subset of original grant. If not provided, use original resources.
  const requestedResources = formData.getAll('resource').map((r) => r.toString());
  const originalResources = refreshTokenData.resource;
  let effectiveResources: string[];

  if (requestedResources.length > 0) {
    // Validate each resource URI
    for (const resource of requestedResources) {
      const resourceResult = v.safeParse(ResourceSchema, resource);
      if (!resourceResult.success) {
        logger.debug('Invalid resource URI in refresh request', { resource });
        return tokenError(c, 'invalid_target', 'Invalid resource URI format');
      }
    }
    // Verify requested resources are subset of original
    const originalResourceSet = new Set(originalResources);
    for (const resource of requestedResources) {
      if (!originalResourceSet.has(resource)) {
        logger.debug('Resource not in original grant', { resource, originalResources });
        return tokenError(c, 'invalid_target', 'Requested resource was not in original grant');
      }
    }
    effectiveResources = requestedResources;
  } else {
    // No resource specified - use original resources
    effectiveResources = originalResources;
  }

  // 6. Generate new access token JWT
  const encoder = new TextEncoder();
  const secret = encoder.encode(c.env.JWT_SECRET);
  const tokenIssuedAt = Math.floor(Date.now() / 1000);
  const tokenExpiration = tokenIssuedAt + TOKEN_EXPIRY_SECONDS;

  const accessToken = await new SignJWT({
    jti: crypto.randomUUID(),
    email: refreshTokenData.email,
    client_id: clientId,
    scope: effectiveScope,
  })
    .setProtectedHeader({ alg: 'HS256' })
    .setSubject(refreshTokenData.sub)
    .setIssuedAt(tokenIssuedAt)
    .setExpirationTime(tokenExpiration)
    .setIssuer(getOrigin(c))
    .setAudience(effectiveResources.length === 1 ? effectiveResources[0] : effectiveResources)
    .sign(secret);

  // 7. Store new access token
  const accessTokenKey = await sha256Base64Url(accessToken);
  const accessTokenStorage = createAccessTokenStorage(c.env.OAUTH_KV);
  try {
    await accessTokenStorage.put(accessTokenKey, {
      sub: refreshTokenData.sub,
      email: refreshTokenData.email,
      client_id: clientId,
      scope: effectiveScope,
      resource: effectiveResources, // RFC 8707: effective resources for this token
    });
  } catch (storeError) {
    logger.debug('Error storing access token', { error: extractErrorMessage(storeError) });
    return c.json(
      {
        error: 'server_error',
        error_description: 'Failed to store token data',
      },
      500,
    );
  }

  // 8. Generate and store new refresh token (rotation)
  // RFC 8707 Section 2.2: new refresh token keeps original resources (not narrowed)
  const newRefreshToken = generateSecureToken();
  const newRefreshTokenKey = await sha256Base64Url(newRefreshToken);
  try {
    await refreshTokenStorage.put(newRefreshTokenKey, {
      sub: refreshTokenData.sub,
      client_id: clientId,
      email: refreshTokenData.email,
      scope: effectiveScope,
      resource: originalResources, // Keep original grant resources
    });
  } catch (storeError) {
    logger.debug('Error storing refresh token', { error: extractErrorMessage(storeError) });
    return c.json(
      {
        error: 'server_error',
        error_description: 'Failed to store refresh token',
      },
      500,
    );
  }

  // 9. Delete old refresh token AFTER successful storage (prevents stranded clients)
  try {
    await refreshTokenStorage.delete(refreshTokenKey);
    logger.debug('Refresh token rotated', { clientId });
  } catch (deleteError) {
    // Log but don't fail - client has new token, old one will TTL out eventually
    logger.error('Failed to delete old refresh token during rotation', {
      clientId,
      error: extractErrorMessage(deleteError),
    });
  }

  // 10. Return token response
  logger.debug('Returning refreshed token response', { clientId });
  return c.json(
    {
      access_token: accessToken,
      token_type: 'Bearer',
      expires_in: TOKEN_EXPIRY_SECONDS,
      refresh_token: newRefreshToken,
      scope: effectiveScope,
    },
    200,
    {
      'Cache-Control': 'no-store',
      Pragma: 'no-cache',
    },
  );
}

export function createOAuthRoutes(deps: OAuthRouteDeps) {
  const { logger, identityProvider } = deps;
  const app = createHono();

  // Rate limiting for all OAuth endpoints
  // Protects against brute force attacks, client registration spam, and token enumeration
  app.use('*', rateLimitMiddleware());

  // NOTE: /userinfo endpoint is OIDC-specific (OpenID Connect Core Section 5.3)
  // and has been intentionally omitted since this implementation does not support OIDC.

  // Dynamic Client Registration (RFC 7591)
  app.post('/register', async (c) => {
    try {
      const body = await c.req.json();
      logger.debug('Client registration request received', { body });

      // Validate registration request
      const result = v.safeParse(ClientRegistrationSchema, body);
      if (!result.success) {
        const issue = result.issues[0];
        logger.debug('Registration validation failed', { error: issue.message });
        return c.json(
          {
            error: 'invalid_client_metadata',
            error_description: issue.message,
          },
          400,
        );
      }

      const { redirect_uris, client_name, client_uri, logo_uri, scope } = result.output;

      // Generate client_id
      const client_id = crypto.randomUUID();

      // RFC 7591 Section 3.2.1: client_id_issued_at is RECOMMENDED
      const client_id_issued_at = Math.floor(Date.now() / 1000);

      // Build client record
      // NOTE: We use 'mcp' as the default scope since we don't support OIDC scopes (profile, email)
      const clientData = {
        client_id,
        client_name: client_name || `Client ${client_id.substring(0, 8)}`,
        redirect_uris,
        client_uri,
        logo_uri,
        scope: scope || 'mcp',
        status: 'active' as const,
        token_endpoint_auth_method: 'none' as const,
        grant_types: ['authorization_code', 'refresh_token'],
        response_types: ['code'],
        created_at: new Date().toISOString(),
      };

      // Store in KV
      const clientStorage = createClientStorage(c.env.OAUTH_KV);
      await clientStorage.put(client_id, clientData);

      logger.debug('Registered new client', { clientId: client_id });

      // Return client metadata (RFC 7591 Section 3.2.1)
      return c.json(
        {
          client_id,
          client_id_issued_at,
          client_name: clientData.client_name,
          redirect_uris,
          client_uri,
          logo_uri,
          scope: clientData.scope,
          token_endpoint_auth_method: 'none',
          grant_types: clientData.grant_types,
          response_types: clientData.response_types,
        },
        201,
      );
    } catch (error) {
      logger.debug('Registration error', { error: extractErrorMessage(error) });
      return c.json(
        {
          error: 'invalid_client_metadata',
          error_description: 'Invalid client registration request',
        },
        400,
      );
    }
  });

  // OAuth redirect endpoint (redirects to identity provider)
  app.get('/authorize', async (c) => {
    try {
      const url = new URL(c.req.url);
      logger.debug('Authorize endpoint hit', {
        url: url.toString(),
        searchParams: Object.fromEntries(url.searchParams),
      });

      // RFC 6749 Section 4.1.1: response_type is REQUIRED
      const responseType = url.searchParams.get('response_type');
      if (!responseType) {
        return c.json({ error: 'invalid_request', error_description: 'Missing response_type' }, 400);
      }
      if (responseType !== 'code') {
        return c.json({ error: 'unsupported_response_type', error_description: 'Only response_type=code is supported' }, 400);
      }

      // Extract and forward OAuth parameters
      const clientId = url.searchParams.get('client_id');
      if (!clientId) {
        return c.json({ error: 'invalid_request', error_description: 'Missing client_id' }, 400);
      }
      const clientStorage = createClientStorage(c.env.OAUTH_KV);
      const client = await clientStorage.get(clientId);
      if (!client) {
        logger.debug('Unknown client_id', { clientId });
        return c.json({ error: 'invalid_client', error_description: 'Unknown client_id' }, 400);
      }
      if (client.status !== 'active') {
        logger.debug('Inactive client_id', { clientId });
        return c.json({ error: 'invalid_client', error_description: 'Inactive client_id' }, 400);
      }

      const redirectUri = url.searchParams.get('redirect_uri');
      if (!redirectUri) {
        return c.json({ error: 'invalid_request', error_description: 'Missing redirect_uri' }, 400);
      }
      if (!client.redirect_uris?.includes(redirectUri)) {
        logger.debug('Mismatched redirect_uri', { clientId, redirectUri });
        return c.json({ error: 'invalid_request', error_description: 'Mismatched redirect_uri' }, 400);
      }

      // RFC 6749 Section 4.1.2.1: After redirect_uri is validated, errors MUST be
      // returned via redirect with error query parameters, not JSON responses.
      const state = url.searchParams.get('state');
      const codeChallenge = url.searchParams.get('code_challenge');
      // QUESTION: do we want to allow non-PKCE requests?
      if (!codeChallenge) {
        logger.debug('Missing code_challenge for PKCE', { clientId });
        return redirectError(redirectUri, state, 'invalid_request', 'Missing code_challenge for PKCE');
      }
      const codeChallengeMethod = url.searchParams.get('code_challenge_method');
      if (codeChallengeMethod !== 'S256') {
        logger.debug('Unsupported code_challenge_method', { codeChallengeMethod });
        return redirectError(redirectUri, state, 'invalid_request', 'Unsupported code_challenge_method');
      }

      // Extract and validate optional scope parameter
      // Only 'mcp' scope is supported (see discovery.ts scopes_supported)
      const scope = url.searchParams.get('scope');
      if (scope && scope !== 'mcp') {
        logger.debug('Invalid scope requested', { scope });
        return redirectError(redirectUri, state, 'invalid_scope', 'Only mcp scope is supported');
      }

      // Require state from client (RFC 6749 - we must return it unchanged)
      // PKCE is primary protection, but state is required for spec compliance and client correlation
      if (!state) {
        logger.debug('Missing state parameter from client', { clientId });
        return redirectError(redirectUri, null, 'invalid_request', 'Missing required state parameter');
      }

      // Validate client-provided state format
      const stateResult = v.safeParse(StateSchema, state);
      if (!stateResult.success) {
        logger.debug('Invalid state parameter format', { state });
        return redirectError(redirectUri, state, 'invalid_request', 'Invalid state parameter format');
      }

      // RFC 8707: Extract and validate resource parameter(s)
      // Multiple resource parameters are allowed per RFC 8707 Section 2
      const resources = url.searchParams.getAll('resource');
      if (resources.length === 0) {
        logger.debug('Missing required resource parameter', { clientId });
        return redirectError(redirectUri, state, 'invalid_target', 'Missing required resource parameter');
      }
      for (const resource of resources) {
        const resourceResult = v.safeParse(ResourceSchema, resource);
        if (!resourceResult.success) {
          logger.debug('Invalid resource URI', { resource, error: resourceResult.issues[0]?.message });
          return redirectError(redirectUri, state, 'invalid_target', 'Invalid resource URI format');
        }
      }

      // Strict resource validation: only allow ${origin}/mcp
      // This prevents token confusion attacks by ensuring tokens are only issued for this server's MCP endpoint
      const expectedResource = `${url.origin}/mcp`;
      for (const resource of resources) {
        if (resource !== expectedResource) {
          logger.debug('Invalid resource - not allowed', { resource, expectedResource });
          return redirectError(redirectUri, state, 'invalid_target', 'Resource must be the MCP endpoint');
        }
      }

      // Generate server-side tx for IdP correlation (separate from client's state)
      const tx = generateTx();
      logger.debug('OAuth authorization request', {
        clientId,
        redirectUri,
        state,
        tx,
        hasCodeChallenge: !!codeChallenge,
        codeChallengeMethod,
        resources,
      });

      // Store OAuth request parameters in KV, keyed by tx (not client's state)
      // This separates internal correlation (tx) from client CSRF protection (state)
      const oauthTxData = {
        client_id: clientId,
        redirect_uri: redirectUri,
        code_challenge: codeChallenge,
        code_challenge_method: codeChallengeMethod,
        original_state: state, // Preserve client's state for redirect
        scope: scope || undefined,
        resource: resources, // RFC 8707: resource indicator(s)
      };

      try {
        logger.debug('Saving OAuth tx data', { tx, oauthTxData });
        const oauthTxStorage = createOAuthTxStorage(c.env.OAUTH_KV);
        await oauthTxStorage.put(tx, oauthTxData);
        logger.debug('Successfully stored OAuth tx data in KV', { tx });
      } catch (kvError) {
        logger.debug('Error storing OAuth data in KV', { error: extractErrorMessage(kvError) });
        return redirectError(redirectUri, state, 'server_error', 'Error storing OAuth data');
      }

      // Build callback URL (clean, no tx - provider decides where tx goes)
      // Provider embeds tx where appropriate:
      // - Standard OAuth: state param (IdP passes it through)
      // - Stytch: query param in login_redirect_url (Stytch doesn't pass state)
      const callbackUrl = new URL('/callback', url.origin);

      // Get provider-specific authorization URL
      const authUrl = await identityProvider.buildAuthorizationUrl({
        callbackUrl: callbackUrl.toString(),
        tx,
      });

      logger.debug('Redirecting to IdP', {
        provider: identityProvider.name,
        authUrl: authUrl.toString(),
        callbackUrl: callbackUrl.toString(),
      });
      return c.redirect(authUrl.toString(), 302);
    } catch (error) {
      logger.error('Error in authorize endpoint', { error: extractErrorMessage(error) });
      return c.json(
        {
          error: 'server_error',
          error_description: 'Authorization request failed',
        },
        500,
      );
    }
  });

  // OAuth callback endpoint
  app.get('/callback', async (c) => {
    // Declare outside try block so it's accessible in catch for error redirects
    let oauthTxData: OAuthTxSchema | null = null;
    try {
      const url = new URL(c.req.url);
      logger.debug('Callback endpoint hit', {
        url: url.toString(),
        searchParams: Object.fromEntries(url.searchParams),
      });

      // Parse provider-specific callback - provider extracts token and tx from wherever it put them
      const callbackParams = identityProvider.parseCallback(url);

      if (!callbackParams) {
        logger.debug('Invalid callback - provider could not parse', { provider: identityProvider.name });
        return c.json(
          {
            error: 'invalid_request',
            error_description: 'Invalid callback request',
          },
          400,
        );
      }

      logger.debug('Callback parsed', { hasToken: !!callbackParams.token, hasIdpError: !!callbackParams.idpError, tx: callbackParams.tx });

      // Validate tx format before using as KV key
      const txResult = v.safeParse(TxSchema, callbackParams.tx);
      if (!txResult.success) {
        logger.debug('Invalid tx parameter format', { tx: callbackParams.tx });
        return c.json({ error: 'invalid_request', error_description: 'Invalid tx parameter format' }, 400);
      }

      const tx = callbackParams.tx;

      // Look up OAuth request by tx
      const oauthTxStorage = createOAuthTxStorage(c.env.OAUTH_KV);
      try {
        oauthTxData = await oauthTxStorage.get(tx);
        if (oauthTxData) {
          logger.debug('Found OAuth tx data', { tx });

          // Delete tx immediately after lookup (single-use)
          await oauthTxStorage.delete(tx);
          logger.debug('Deleted oauth_tx (single-use)', { tx });
        }
      } catch (error) {
        logger.debug('Error getting OAuth tx data', { error: extractErrorMessage(error) });
      }

      if (!oauthTxData) {
        logger.debug('No OAuth tx data found', { tx });
        return c.json(
          {
            error: 'invalid_request',
            error_description: 'Invalid or expired transaction',
          },
          400,
        );
      }

      // Extract the original client state to return in the final redirect
      const originalState = oauthTxData.original_state;
      if (!originalState) {
        // RFC 6749 §4.1.2.1: redirect with error params once redirect_uri is known
        logger.debug('No original_state found in tx data', { tx });
        return redirectError(oauthTxData.redirect_uri, null, 'server_error', 'Missing original state in transaction data');
      }

      // RFC 6749 §4.1.2.1: If IdP returned an error, propagate it to the client via redirect
      if (callbackParams.idpError) {
        const { error, error_description } = callbackParams.idpError;
        // Include error code in fallback description for observability when IdP omits description
        const description = error_description || `Authorization failed: ${error}`;
        logger.debug('IdP returned error, redirecting to client', {
          error,
          errorDescription: error_description,
          redirectUri: oauthTxData.redirect_uri,
        });
        return redirectError(oauthTxData.redirect_uri, originalState, error, description);
      }

      // Defensive check: callback has neither error nor authorization code
      // This handles malformed IdP responses where parseCallback returns { tx } without token or error
      if (!callbackParams.token) {
        logger.debug('Invalid callback: neither error nor authorization code present');
        return redirectError(oauthTxData.redirect_uri, originalState, 'server_error', 'Invalid callback: missing authorization code');
      }

      // Authenticate with identity provider
      logger.debug('Authenticating with identity provider', { provider: identityProvider.name });

      // Reconstruct gateway callback URL (must match what was sent to IdP per RFC 6749 §4.1.3)
      const gatewayCallbackUrl = new URL('/callback', url.origin).toString();

      let user;
      try {
        user = await identityProvider.authenticate({
          token: callbackParams.token,
          redirectUri: gatewayCallbackUrl,
        });
      } catch (authError) {
        // RFC 6749 §4.1.2.1: redirect with error params once redirect_uri is known
        logger.debug('Authentication error', { error: extractErrorMessage(authError) });
        return redirectError(oauthTxData.redirect_uri, originalState, 'access_denied', 'Authentication failed. Please try again.');
      }

      logger.debug('Auth successful', { sub: user.sub, hasEmail: !!user.email });

      // Generate an authorization code
      const authCode = generateSecureToken();

      // Store the user info with the authorization code
      const authCodeData = {
        sub: user.sub,
        email: user.email,
        code_challenge: oauthTxData.code_challenge,
        client_id: oauthTxData.client_id,
        redirect_uri: oauthTxData.redirect_uri,
        scope: oauthTxData.scope,
        resource: oauthTxData.resource, // RFC 8707: resource indicator(s)
      };

      logger.debug('Storing auth code data', { authCode, clientId: authCodeData.client_id });

      const authCodeStorage = createAuthCodeStorage(c.env.OAUTH_KV);
      await authCodeStorage.put(authCode, authCodeData);

      // Redirect back to client with authorization code
      const redirectURL = new URL(oauthTxData.redirect_uri);
      redirectURL.searchParams.set('code', authCode);
      redirectURL.searchParams.set('state', originalState);

      logger.debug('Redirecting to client with auth code', {
        redirectUri: oauthTxData.redirect_uri,
        state: originalState,
      });
      return c.redirect(redirectURL.toString(), 302);
    } catch (error) {
      logger.error('Callback error', { error: extractErrorMessage(error) });
      // RFC 6749 §4.1.2.1: redirect with error params if redirect_uri is known
      if (oauthTxData) {
        return redirectError(oauthTxData.redirect_uri, oauthTxData.original_state ?? null, 'server_error', 'Callback processing failed');
      }
      // Fallback to JSON if redirect_uri is not yet known
      return c.json(
        {
          error: 'server_error',
          error_description: 'Callback processing failed',
        },
        500,
      );
    }
  });

  // Token exchange endpoint
  app.post('/token', async (c) => {
    try {
      logger.debug('Token endpoint hit');

      const formData = await c.req.formData();

      // Early grant_type branching - parse before schema validation
      // to support multiple grant types with different schemas
      const grantType = formData.get('grant_type');
      if (grantType === 'refresh_token') {
        try {
          return await handleRefreshTokenGrant(c, formData, logger);
        } catch (error) {
          logger.error('Refresh token grant error', { error: extractErrorMessage(error) });
          return tokenError(c, 'server_error', 'Refresh token processing error', 500);
        }
      }

      const rawData = {
        grant_type: grantType,
        code: formData.get('code'),
        redirect_uri: formData.get('redirect_uri'),
        client_id: formData.get('client_id'),
        code_verifier: formData.get('code_verifier'),
      };

      const result = v.safeParse(TokenRequestSchema, rawData);
      if (!result.success) {
        const issues = v.flatten(result.issues);
        const issue = result.issues[0];
        logger.debug('Token request validation failed', { error: issue.message, issues });

        return c.json(
          {
            error: 'invalid_request',
            error_description: `Invalid token request parameter.`,
          },
          400,
        );
      }

      const { code, redirect_uri: redirectUri, client_id: clientId, code_verifier: codeVerifier } = result.output;

      logger.debug('Token request received', {
        grantType,
        hasCode: !!code,
        redirectUri,
        clientId,
        hasCodeVerifier: !!codeVerifier,
      });

      const authCodeStorage = createAuthCodeStorage(c.env.OAUTH_KV);
      const accessTokenStorage = createAccessTokenStorage(c.env.OAUTH_KV);

      const authCodeData = await authCodeStorage.get(code);

      if (!authCodeData) {
        logger.debug('Invalid or expired authorization code', { clientId });
        return tokenError(c, 'invalid_grant', 'Invalid or expired authorization code');
      }

      if (authCodeData.client_id !== clientId) {
        logger.debug('client_id mismatch', { expected: authCodeData.client_id, received: clientId });
        return tokenError(c, 'invalid_grant', 'client_id mismatch');
      }

      // Verify client still exists and is active (may have been deactivated since authorization)
      const clientStorage = createClientStorage(c.env.OAUTH_KV);
      const client = await clientStorage.get(clientId);
      if (!client || client.status !== 'active') {
        logger.debug('Client not found or inactive', { clientId });
        return tokenError(c, 'invalid_client', 'Client not found or inactive');
      }

      // Verify redirect_uri matches
      if (authCodeData.redirect_uri !== redirectUri) {
        logger.debug('redirect_uri mismatch', { expected: authCodeData.redirect_uri, received: redirectUri });
        return tokenError(c, 'invalid_grant', 'redirect_uri mismatch');
      }

      // RFC 8707: Parse and validate resource parameter(s)
      const requestedResources = formData.getAll('resource').map((r) => r.toString());
      if (requestedResources.length === 0) {
        logger.debug('Missing required resource parameter in token request', { clientId });
        return tokenError(c, 'invalid_target', 'Missing required resource parameter');
      }
      for (const resource of requestedResources) {
        const resourceResult = v.safeParse(ResourceSchema, resource);
        if (!resourceResult.success) {
          logger.debug('Invalid resource URI in token request', { resource });
          return tokenError(c, 'invalid_target', 'Invalid resource URI format');
        }
      }

      // Strict resource validation: only allow ${origin}/mcp
      const expectedResource = `${new URL(c.req.url).origin}/mcp`;
      for (const resource of requestedResources) {
        if (resource !== expectedResource) {
          logger.debug('Invalid resource in token request - not allowed', { resource, expectedResource });
          return tokenError(c, 'invalid_target', 'Resource must be the MCP endpoint');
        }
      }

      // Verify requested resources are a subset of auth code's resources
      const authorizedResources = new Set(authCodeData.resource);
      for (const resource of requestedResources) {
        if (!authorizedResources.has(resource)) {
          logger.debug('Resource not in original authorization', { resource, authorizedResources: authCodeData.resource });
          return tokenError(c, 'invalid_target', 'Requested resource was not authorized');
        }
      }

      // PKCE validation - enforce consistency
      const hasCodeChallenge = !!authCodeData.code_challenge;
      const hasCodeVerifier = !!codeVerifier;

      if (hasCodeChallenge && !hasCodeVerifier) {
        // code_challenge was provided in authorize, but code_verifier missing in token request
        logger.debug('PKCE: code_challenge exists but code_verifier missing', { clientId });
        return tokenError(c, 'invalid_grant', 'code_verifier required when code_challenge was provided');
      }

      if (!hasCodeChallenge && hasCodeVerifier) {
        // code_verifier provided but no code_challenge was stored - inconsistent
        logger.debug('PKCE: code_verifier provided but no code_challenge was stored', { clientId });
        return tokenError(c, 'invalid_grant', 'code_verifier provided but no code_challenge was sent in authorization request');
      }

      if (hasCodeChallenge && hasCodeVerifier) {
        const computedChallenge = await sha256Base64Url(codeVerifier);
        const match = timingSafeEqual(computedChallenge, authCodeData.code_challenge);

        logger.debug('PKCE verification', {
          storedChallenge: authCodeData.code_challenge,
          computedChallenge,
          match,
        });

        if (!match) {
          logger.debug('PKCE verification failed', { clientId });
          return tokenError(c, 'invalid_grant', 'PKCE verification failed');
        }
      }

      // Delete the used authorization code
      try {
        await authCodeStorage.delete(code);
        logger.debug('Used authorization code deleted', { clientId });
      } catch (deleteError) {
        logger.debug('Error deleting used auth code', { error: extractErrorMessage(deleteError) });
        // Continue anyway since this isn't critical
      }

      // Generate JWT access token instead of UUID
      const encoder = new TextEncoder();
      const secret = encoder.encode(c.env.JWT_SECRET);

      // TODO: Once Claude Code fixes scopes_supported handling (issue #7744),
      // require offline_access scope: grantedScope.includes('offline_access')
      // For now, we issue refresh tokens unconditionally.
      const grantedScope = authCodeData.scope || 'mcp';

      // Create JWT payload - use origin URL as issuer to match discovery metadata
      const tokenIssuedAt = Math.floor(Date.now() / 1000);
      const tokenExpiration = tokenIssuedAt + TOKEN_EXPIRY_SECONDS;
      const accessTokenPayload: JWTPayload = {
        jti: crypto.randomUUID(),
        email: authCodeData.email,
        client_id: clientId,
        scope: grantedScope,
      };

      // Sign JWT - RFC 8707: audience is the requested resource(s)
      const accessToken = await new SignJWT(accessTokenPayload)
        .setProtectedHeader({ alg: 'HS256' })
        .setSubject(authCodeData.sub)
        .setIssuedAt(tokenIssuedAt)
        .setExpirationTime(tokenExpiration)
        .setIssuer(getOrigin(c))
        .setAudience(requestedResources.length === 1 ? requestedResources[0] : requestedResources)
        .sign(secret);

      logger.debug('Generated JWT access token', { tokenPreview: accessToken.substring(0, 20) });

      // Store token information - use a hash of the token as the key to avoid length limits
      const tokenKey = await sha256Base64Url(accessToken);

      try {
        logger.debug('Storing access token', { tokenKey, clientId });
        await accessTokenStorage.put(tokenKey, {
          sub: authCodeData.sub,
          email: authCodeData.email,
          client_id: clientId,
          scope: grantedScope,
          resource: requestedResources, // RFC 8707: effective resources for this token
        });

        logger.debug('Token data successfully stored', { clientId });
      } catch (storeError) {
        logger.debug('Error storing token data', { error: extractErrorMessage(storeError) });
        return c.json(
          {
            error: 'server_error',
            error_description: 'Failed to store token data',
          },
          500,
        );
      }

      // Generate refresh token (always issued - see TODO above for future offline_access requirement)
      // RFC 8707 Section 2.2: refresh token is bound to full original grant, not narrowed resources
      const refreshToken = generateSecureToken();
      const refreshTokenKey = await sha256Base64Url(refreshToken);
      const refreshTokenStorage = createRefreshTokenStorage(c.env.OAUTH_KV);

      try {
        await refreshTokenStorage.put(refreshTokenKey, {
          sub: authCodeData.sub,
          client_id: clientId,
          email: authCodeData.email,
          scope: grantedScope,
          resource: authCodeData.resource, // RFC 8707: original grant resources (not narrowed)
        });
        logger.debug('Refresh token stored', { clientId });
      } catch (storeError) {
        logger.debug('Error storing refresh token', { error: extractErrorMessage(storeError) });
        return c.json(
          {
            error: 'server_error',
            error_description: 'Failed to store refresh token',
          },
          500,
        );
      }

      // Return the tokens (RFC 6749 Section 5.1)
      const tokenResponse = {
        access_token: accessToken,
        token_type: 'Bearer',
        expires_in: TOKEN_EXPIRY_SECONDS,
        refresh_token: refreshToken,
        scope: grantedScope,
      };

      logger.debug('Returning token response', { clientId, expiresIn: TOKEN_EXPIRY_SECONDS });
      // RFC 6749 Section 5.1: Cache-Control: no-store and Pragma: no-cache MUST be included
      return c.json(tokenResponse, 200, {
        'Cache-Control': 'no-store',
        Pragma: 'no-cache',
      });
    } catch (error) {
      logger.error('Token endpoint error', { error: extractErrorMessage(error) });
      return c.json(
        {
          error: 'server_error',
          error_description: 'Token request failed',
        },
        500,
      );
    }
  });

  // Token Revocation endpoint (RFC 7009)
  // NOTE: No client_id validation - any token holder can revoke. This is acceptable
  // for public clients (PKCE). For confidential clients, consider adding client
  // authentication. See RFC 7009 §2.1 for client authentication requirements.
  app.post('/revoke', async (c) => {
    logger.debug('Revoke endpoint hit');

    const formData = await c.req.formData();
    const token = formData.get('token');

    // RFC 7009 §2.2.1: Always return 200 even for invalid tokens
    // This prevents attackers from probing for valid tokens
    if (!token || typeof token !== 'string') {
      return c.json({}, 200, { 'Cache-Control': 'no-store', Pragma: 'no-cache' });
    }

    const tokenTypeHint = formData.get('token_type_hint');
    const tokenHash = await sha256Base64Url(token);

    const accessTokenStorage = createAccessTokenStorage(c.env.OAUTH_KV);
    const refreshTokenStorage = createRefreshTokenStorage(c.env.OAUTH_KV);

    // Delete from storage based on hint (or try both if no hint)
    if (tokenTypeHint === 'access_token' || !tokenTypeHint) {
      try {
        await accessTokenStorage.delete(tokenHash);
        logger.debug('Deleted access token', { tokenHash: tokenHash.substring(0, 8) });
      } catch (error) {
        logger.debug('Error deleting access token', { error: extractErrorMessage(error) });
      }
    }
    if (tokenTypeHint === 'refresh_token' || !tokenTypeHint) {
      try {
        await refreshTokenStorage.delete(tokenHash);
        logger.debug('Deleted refresh token', { tokenHash: tokenHash.substring(0, 8) });
      } catch (error) {
        logger.debug('Error deleting refresh token', { error: extractErrorMessage(error) });
      }
    }

    // RFC 7009 §2.1: Always return 200, even if token was invalid/already revoked
    return c.json({}, 200, { 'Cache-Control': 'no-store', Pragma: 'no-cache' });
  });

  return app;
}

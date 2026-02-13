/**
 * Standard OAuth 2.0 Identity Provider implementation.
 *
 * Standard OAuth flow differs from Stytch:
 * - Uses standard OAuth parameters (client_id, redirect_uri, state, scope)
 * - TX is placed in state param (IdP passes it through unchanged)
 * - Token exchange via POST /token with standard grant_type=authorization_code
 * - User info via separate /userinfo endpoint
 */

import { createRemoteJWKSet, jwtVerify } from 'jose';
import * as v from 'valibot';
import type { StandardOAuthEnv } from '../../env';
import { createOidcConfigCache } from '../../storage';
import type { Logger } from '../../utils/logger';
import type { AuthenticateContext, AuthenticatedUser, AuthorizationConfig, CallbackParams, IdentityProvider } from '../interface';

/**
 * Schema for OAuth token endpoint response.
 * Some providers include user claims directly in the token response.
 */
const TokenResponseSchema = v.object({
  access_token: v.string(),
  token_type: v.string(),
  expires_in: v.optional(v.number()),
  refresh_token: v.optional(v.string()),
  scope: v.optional(v.string()),
  // Some providers include user info directly in token response
  sub: v.optional(v.string()),
  email: v.optional(v.string()),
  // OIDC providers may return an id_token
  id_token: v.optional(v.string()),
});

/**
 * Schema for OAuth userinfo endpoint response.
 */
const UserInfoResponseSchema = v.object({
  sub: v.string(),
  email: v.optional(v.string()),
  name: v.optional(v.string()),
});

/**
 * Schema for OIDC discovery document.
 * Includes all endpoints needed for auto-discovery.
 */
const OidcDiscoverySchema = v.object({
  issuer: v.string(),
  jwks_uri: v.string(),
  authorization_endpoint: v.optional(v.string()),
  token_endpoint: v.optional(v.string()),
  userinfo_endpoint: v.optional(v.string()),
});

type OidcConfig = v.InferOutput<typeof OidcDiscoverySchema>;

/**
 * Standard OAuth 2.0 Identity Provider.
 *
 * Works with Auth0, Okta, Google, and other standard OAuth 2.0 providers.
 *
 * Endpoint configuration (two modes):
 * 1. Auto-discovery: Set OAUTH_ISSUER - endpoints discovered from /.well-known/openid-configuration
 * 2. Explicit: Set OAUTH_AUTHORIZATION_URL and OAUTH_TOKEN_URL directly
 *
 * Explicit URLs override discovered values (useful for non-standard providers).
 *
 * Key differences from Stytch:
 * 1. Uses state param for TX (standard OAuth passes state through)
 * 2. Standard token exchange with code + redirect_uri
 * 3. Separate userinfo endpoint for user details
 *
 * User identity resolution (in priority order):
 * 1. OAUTH_USERINFO_URL - fetch user info from dedicated endpoint
 * 2. id_token extraction - cryptographically signed, requires OAUTH_ISSUER
 * 3. Token response 'sub' claim - unsigned convenience field, fallback only
 */
export class StandardOAuthProvider implements IdentityProvider {
  readonly name = 'oauth';

  /** KV-backed cache for OIDC config (persistent across requests/isolates) */
  private kvCache: ReturnType<typeof createOidcConfigCache> | null;

  /**
   * Instance-level cache for discovered config.
   * Effective because discoverOidcConfig() is called multiple times per request
   * (getTokenEndpoint, extractClaimsFromIdToken, getUserinfoEndpoint all call it).
   */
  private discoveredConfig: OidcConfig | null = null;

  constructor(
    private env: StandardOAuthEnv,
    private logger: Logger,
    kv?: KVNamespace,
  ) {
    // Only create KV cache if KV is provided
    this.kvCache = kv ? createOidcConfigCache(kv) : null;
  }

  /**
   * Build the authorization URL for standard OAuth.
   *
   * TX is placed in the state parameter - standard OAuth IdPs pass it through unchanged.
   * This keeps redirect_uri clean for exact-match validation.
   *
   * Uses OAUTH_AUTHORIZATION_URL if set, otherwise discovers from OAUTH_ISSUER.
   */
  async buildAuthorizationUrl(config: AuthorizationConfig): Promise<URL> {
    const authEndpoint = await this.getAuthorizationEndpoint();
    const url = new URL(authEndpoint);

    url.searchParams.set('client_id', this.env.OAUTH_CLIENT_ID);
    url.searchParams.set('redirect_uri', config.callbackUrl); // Clean URL, no tx
    url.searchParams.set('response_type', 'code');
    url.searchParams.set('state', config.tx); // TX goes in state param (IdP passes through)

    // Use config scopes if provided, otherwise use env scopes (defaults to 'openid email profile')
    const scopes = config.scopes?.length ? config.scopes : this.env.OAUTH_SCOPES;
    if (scopes.length) {
      url.searchParams.set('scope', scopes.join(' '));
    }

    this.logger.debug('Standard OAuth authorization URL built', { url: url.toString() });

    return url;
  }

  /**
   * Parse the callback from standard OAuth IdP.
   *
   * Standard OAuth returns either:
   * - Success: code + state
   * - Error: error + error_description + state (RFC 6749 §4.1.2.1)
   *
   * We must handle both cases to properly propagate IdP errors to the client.
   */
  parseCallback(url: URL): CallbackParams | null {
    const tx = url.searchParams.get('state'); // Extract TX from state param

    // TX is required for both success and error responses
    if (!tx) {
      this.logger.debug('No state (tx) found in OAuth callback');
      return null;
    }

    // Check for IdP error response (RFC 6749 §4.1.2.1)
    const error = url.searchParams.get('error');
    if (error) {
      const errorDescription = url.searchParams.get('error_description');
      this.logger.debug('IdP returned error in callback', { error, errorDescription, tx });
      return {
        tx,
        idpError: {
          error,
          error_description: errorDescription ?? undefined,
        },
      };
    }

    // Check for success response
    const code = url.searchParams.get('code');
    if (!code) {
      this.logger.debug('No code found in OAuth callback');
      return null;
    }

    this.logger.debug('Parsed OAuth callback', { hasCode: true, tx });

    return {
      token: code,
      tx,
    };
  }

  /**
   * Authenticate with standard OAuth token exchange.
   *
   * 1. Exchange authorization code for access token
   * 2. Get user info (from userinfo endpoint, token response, or id_token)
   *
   * Uses OAUTH_TOKEN_URL/OAUTH_USERINFO_URL if set, otherwise discovers from OAUTH_ISSUER.
   */
  async authenticate(ctx: AuthenticateContext): Promise<AuthenticatedUser> {
    this.logger.debug('Exchanging authorization code for tokens');

    // Step 1: Exchange code for tokens
    const tokenEndpoint = await this.getTokenEndpoint();
    const tokenResponse = await fetch(tokenEndpoint, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: new URLSearchParams({
        grant_type: 'authorization_code',
        code: ctx.token,
        redirect_uri: ctx.redirectUri, // Required by many IdPs - must match original
        client_id: this.env.OAUTH_CLIENT_ID,
        client_secret: this.env.OAUTH_CLIENT_SECRET,
      }),
    });

    if (!tokenResponse.ok) {
      const errorText = await tokenResponse.text();
      this.logger.debug('OAuth token exchange error', { error: errorText, status: tokenResponse.status });
      throw new Error('Token exchange failed');
    }

    const tokensResult = v.safeParse(TokenResponseSchema, await tokenResponse.json());
    if (!tokensResult.success) {
      this.logger.debug('Invalid token response', { issues: tokensResult.issues });
      throw new Error('Invalid token response: missing required fields');
    }

    const tokens = tokensResult.output;
    this.logger.debug('Token exchange successful');

    // Step 2: Get user info (priority order)

    // Option 1: Userinfo endpoint (most reliable)
    const userinfoEndpoint = await this.getUserinfoEndpoint();
    if (userinfoEndpoint) {
      const userInfoResponse = await fetch(userinfoEndpoint, {
        headers: {
          Authorization: `Bearer ${tokens.access_token}`,
        },
      });

      if (!userInfoResponse.ok) {
        const errorText = await userInfoResponse.text();
        this.logger.debug('OAuth userinfo error', { error: errorText, status: userInfoResponse.status });
        throw new Error('Failed to fetch user info');
      }

      const userInfoResult = v.safeParse(UserInfoResponseSchema, await userInfoResponse.json());
      if (!userInfoResult.success) {
        this.logger.debug('Invalid userinfo response', { issues: userInfoResult.issues });
        throw new Error('Invalid userinfo response: missing required fields');
      }

      const userInfo = userInfoResult.output;
      this.logger.debug('OAuth auth successful', { sub: userInfo.sub });

      return {
        sub: userInfo.sub,
        email: userInfo.email,
      };
    }

    // Option 2: Extract from id_token (signed, more trustworthy)
    // Requires OAUTH_ISSUER for secure validation
    if (tokens.id_token) {
      const claims = await this.extractClaimsFromIdToken(tokens.id_token);
      if (claims?.sub) {
        this.logger.debug('OAuth auth successful (from id_token)', { sub: claims.sub });
        return {
          sub: claims.sub,
          email: claims.email ?? tokens.email,
        };
      }
    }

    // Option 3: Fall back to token response (unsigned, convenience field)
    if (tokens.sub) {
      this.logger.debug('OAuth auth successful (from token response)', { sub: tokens.sub });
      return {
        sub: tokens.sub,
        email: tokens.email,
      };
    }

    // No identity source available
    this.logger.debug('No sub in token response, no valid id_token, and no userinfo endpoint available');
    throw new Error(
      'Cannot determine user identity: configure OAUTH_USERINFO_URL or OAUTH_ISSUER, or use a provider that includes sub in token response',
    );
  }

  /**
   * Securely extract claims from an id_token JWT.
   *
   * Requires OAUTH_ISSUER to be configured for OIDC discovery.
   * Validates the JWT signature using the issuer's JWKS.
   *
   * @returns The sub and email claims if validation succeeds, undefined otherwise
   */
  private async extractClaimsFromIdToken(idToken: string): Promise<{ sub: string; email?: string } | undefined> {
    if (!this.env.OAUTH_ISSUER) {
      this.logger.debug('id_token present but OAUTH_ISSUER not configured, skipping extraction');
      return undefined;
    }

    try {
      // Get or discover OIDC config (includes JWKS URI)
      const config = await this.discoverOidcConfig();
      if (!config) {
        this.logger.debug('Failed to discover OIDC config for id_token validation');
        return undefined;
      }

      // Create JWKS fetcher (jose handles key caching internally within the request)
      const jwks = createRemoteJWKSet(new URL(config.jwks_uri));

      // Validate the id_token
      const { payload } = await jwtVerify(idToken, jwks, {
        issuer: this.env.OAUTH_ISSUER,
        audience: this.env.OAUTH_CLIENT_ID,
      });

      if (!payload.sub) {
        this.logger.debug('id_token missing sub claim');
        return undefined;
      }

      return {
        sub: payload.sub,
        email: typeof payload.email === 'string' ? payload.email : undefined,
      };
    } catch (err) {
      this.logger.debug('id_token validation failed', { error: String(err) });
      return undefined;
    }
  }

  /**
   * Get the authorization endpoint URL.
   * Uses explicit OAUTH_AUTHORIZATION_URL if set, otherwise discovers from OAUTH_ISSUER.
   */
  private async getAuthorizationEndpoint(): Promise<string> {
    // Explicit URL takes precedence
    if (this.env.OAUTH_AUTHORIZATION_URL) {
      return this.env.OAUTH_AUTHORIZATION_URL;
    }

    // Try OIDC discovery
    const config = await this.discoverOidcConfig();
    if (config?.authorization_endpoint) {
      return config.authorization_endpoint;
    }

    throw new Error('No authorization endpoint: configure OAUTH_AUTHORIZATION_URL or OAUTH_ISSUER');
  }

  /**
   * Get the token endpoint URL.
   * Uses explicit OAUTH_TOKEN_URL if set, otherwise discovers from OAUTH_ISSUER.
   */
  private async getTokenEndpoint(): Promise<string> {
    // Explicit URL takes precedence
    if (this.env.OAUTH_TOKEN_URL) {
      return this.env.OAUTH_TOKEN_URL;
    }

    // Try OIDC discovery
    const config = await this.discoverOidcConfig();
    if (config?.token_endpoint) {
      return config.token_endpoint;
    }

    throw new Error('No token endpoint: configure OAUTH_TOKEN_URL or OAUTH_ISSUER');
  }

  /**
   * Get the userinfo endpoint URL.
   * Uses explicit OAUTH_USERINFO_URL if set, otherwise discovers from OAUTH_ISSUER.
   * Returns undefined if no userinfo endpoint is available.
   */
  private async getUserinfoEndpoint(): Promise<string | undefined> {
    // Explicit URL takes precedence
    if (this.env.OAUTH_USERINFO_URL) {
      return this.env.OAUTH_USERINFO_URL;
    }

    // Try OIDC discovery (userinfo is optional)
    const config = await this.discoverOidcConfig();
    return config?.userinfo_endpoint;
  }

  /**
   * Discover OIDC configuration from the issuer's well-known endpoint.
   *
   * Uses instance-level cache to avoid duplicate fetches within a single request,
   * and KV cache for persistence across requests and isolates.
   */
  private async discoverOidcConfig(): Promise<OidcConfig | null> {
    const issuer = this.env.OAUTH_ISSUER;

    // No issuer configured, can't discover
    if (!issuer) {
      return null;
    }

    // Return instance cache if already discovered in this request
    if (this.discoveredConfig) {
      return this.discoveredConfig;
    }

    const config = await this.fetchOidcConfig(issuer);
    if (config) {
      this.discoveredConfig = config;
    }
    return config;
  }

  /**
   * Fetch OIDC configuration from the issuer's well-known endpoint.
   * Checks KV cache first, then fetches and caches the result.
   */
  private async fetchOidcConfig(issuer: string): Promise<OidcConfig | null> {
    // Check KV cache first (persistent across isolates)
    if (this.kvCache) {
      const cached = await this.kvCache.get(issuer);
      if (cached) {
        this.logger.debug('OIDC config KV cache hit', { issuer });
        return cached;
      }
    }

    // Fetch OIDC discovery document
    try {
      const discoveryUrl = `${issuer.replace(/\/$/, '')}/.well-known/openid-configuration`;
      this.logger.debug('Fetching OIDC discovery', { discoveryUrl });

      const response = await fetch(discoveryUrl);
      if (!response.ok) {
        this.logger.debug('OIDC discovery fetch failed', { status: response.status });
        return null;
      }

      const discoveryResult = v.safeParse(OidcDiscoverySchema, await response.json());
      if (!discoveryResult.success) {
        this.logger.debug('Invalid OIDC discovery response', { issues: discoveryResult.issues });
        return null;
      }

      const config = discoveryResult.output;

      // RFC 8414 Section 3.2: The issuer value returned MUST be identical to the
      // authorization server's issuer identifier. If not, reject the discovery.
      if (config.issuer !== issuer) {
        this.logger.debug('Issuer mismatch in discovery', { expected: issuer, got: config.issuer });
        return null;
      }

      // Cache the result in KV (persistent across isolates)
      if (this.kvCache) {
        await this.kvCache.put(issuer, config);
        this.logger.debug('OIDC config cached in KV', { issuer });
      }

      return config;
    } catch (err) {
      this.logger.debug('OIDC discovery error', { error: String(err) });
      return null;
    }
  }
}

/**
 * Identity Provider abstraction for OAuth authentication.
 *
 * This interface allows the OAuth gateway to work with multiple identity providers
 * (Stytch, Auth0, Okta, Google, etc.) through a common contract.
 */

/**
 * Authenticated user information returned from identity provider.
 * Providers map their specific response to this common format.
 */
export interface AuthenticatedUser {
  /** Unique user identifier from the IdP */
  sub: string;
  /** User's email address (may be undefined if not provided by IdP) */
  email?: string;
}

/**
 * Configuration for building authorization URLs.
 */
export interface AuthorizationConfig {
  /** Gateway callback URL (clean, no tx embedded) */
  callbackUrl: string;
  /**
   * Correlation ID - provider decides where to put it:
   * - Standard OAuth: state=tx (IdP passes it through)
   * - Stytch: embed in login_redirect_url query param
   */
  tx: string;
  /** Optional scopes to request */
  scopes?: string[];
}

/**
 * IdP error response - RFC 6749 §4.1.2.1 error parameters from the authorization server.
 * When an IdP returns an error instead of an authorization code, we propagate it to the client.
 */
export interface IdpError {
  /** OAuth error code (e.g., 'access_denied', 'server_error') */
  error: string;
  /** Human-readable error description */
  error_description?: string;
}

/**
 * Callback parsing result - extracts tokens/codes and correlation ID from the IdP's callback.
 * May contain either a successful response (token) or an error response from the IdP.
 */
export interface CallbackParams {
  /** The token or authorization code from the callback (present on success) */
  token?: string;
  /** Type of token (provider-specific, e.g., 'oauth' for Stytch) */
  tokenType?: string;
  /**
   * Correlation ID extracted by provider:
   * - Standard OAuth: from state param
   * - Stytch: from tx query param
   */
  tx: string;
  /**
   * IdP error response (RFC 6749 §4.1.2.1).
   * When set, the IdP returned an error instead of an authorization code.
   * This should be propagated to the client via redirect with error params.
   */
  idpError?: IdpError;
}

/**
 * Context for authenticating with the identity provider.
 */
export interface AuthenticateContext {
  /** The token or authorization code from the callback */
  token: string;
  /** Must match original redirect_uri (required by many IdPs for token exchange) */
  redirectUri: string;
}

/**
 * Identity Provider interface - all providers must implement this.
 *
 * The interface is designed so that:
 * 1. Routes generate TX and store PKCE data
 * 2. Provider embeds TX where appropriate for its IdP
 * 3. Provider extracts TX from callback
 * 4. Routes use TX to retrieve stored data
 */
export interface IdentityProvider {
  /** Provider identifier (e.g., 'stytch', 'oauth') */
  readonly name: string;

  /**
   * Build the authorization URL for user redirect.
   *
   * Provider decides where to put the TX correlation ID:
   * - Standard OAuth: use state param (IdP passes it through)
   * - Stytch: embed in login_redirect_url query param
   *
   * Async to support OIDC auto-discovery of authorization endpoint.
   *
   * @param config - Authorization configuration including callback URL and TX
   * @returns URL to redirect the user to
   */
  buildAuthorizationUrl(config: AuthorizationConfig): Promise<URL>;

  /**
   * Parse the callback request to extract tokens/codes and correlation ID.
   *
   * Provider knows where to find the TX:
   * - Standard OAuth: from state param
   * - Stytch: from tx query param
   *
   * @param url - The callback URL with query parameters
   * @returns Parsed callback params, or null if invalid
   */
  parseCallback(url: URL): CallbackParams | null;

  /**
   * Authenticate the token/code with the IdP and get user info.
   *
   * @param ctx - Authentication context including token and redirect URI
   * @returns Authenticated user information
   * @throws Error if authentication fails
   */
  authenticate(ctx: AuthenticateContext): Promise<AuthenticatedUser>;
}

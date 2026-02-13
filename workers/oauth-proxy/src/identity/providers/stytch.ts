/**
 * Stytch Identity Provider implementation.
 *
 * Handles Stytch-specific OAuth behavior:
 * - Uses public_token and login_redirect_url for authorization
 * - Embeds TX in login_redirect_url query param (Stytch doesn't pass state through)
 * - Parses stytch_token_type/stytch_token from callback
 * - Authenticates via POST /oauth/authenticate with Basic auth
 */

import * as v from 'valibot';

import type { StytchEnv } from '../../env';
import type { Logger } from '../../utils/logger';
import { appendPath } from '../../utils/url';
import type { AuthenticateContext, AuthenticatedUser, AuthorizationConfig, CallbackParams, IdentityProvider } from '../interface';

/**
 * Response from Stytch's /oauth/authenticate endpoint.
 */
const StytchAuthResponseSchema = v.object({
  user_id: v.string(),
  user: v.optional(
    v.object({
      emails: v.optional(v.array(v.object({ email: v.string() }))),
    }),
  ),
});

/**
 * Stytch Identity Provider.
 *
 * Stytch has specific behaviors that differ from standard OAuth:
 * 1. It does NOT pass the state parameter through (documented behavior)
 * 2. It uses a hosted login UI that we redirect to with public_token
 * 3. Token authentication uses a different endpoint than standard OAuth token exchange
 */
export class StytchProvider implements IdentityProvider {
  readonly name = 'stytch';

  constructor(
    private env: StytchEnv,
    private logger: Logger,
  ) {}

  /**
   * Get the Stytch API URL for a given path.
   */
  private getApiUrl(path: string): URL {
    return appendPath(this.env.STYTCH_API_URL, path);
  }

  /**
   * Build the authorization URL for Stytch's hosted login UI.
   *
   * Stytch doesn't pass state through, so we embed TX in the callback URL's query params.
   * The login_redirect_url is where Stytch will redirect after authentication.
   */
  async buildAuthorizationUrl(config: AuthorizationConfig): Promise<URL> {
    // Build callback URL with tx embedded in query params
    // (Stytch doesn't pass state, but allows params in redirect URL)
    const callbackUrl = new URL(config.callbackUrl);
    callbackUrl.searchParams.set('tx', config.tx);

    // Build Stytch's hosted login URL
    const stytchUrl = new URL(this.env.STYTCH_OAUTH_URL);
    stytchUrl.searchParams.set('public_token', this.env.STYTCH_PUBLIC_TOKEN);
    stytchUrl.searchParams.set('login_redirect_url', callbackUrl.toString());

    this.logger.debug('Stytch authorization URL built', { url: stytchUrl.toString() });

    return stytchUrl;
  }

  /**
   * Parse the callback from Stytch's hosted login UI.
   *
   * Stytch's callback format:
   * - Success: stytch_token_type + token/stytch_token + tx
   * - Error: stytch_error + stytch_error_description + tx
   *
   * We must handle both cases to properly propagate IdP errors to the client.
   */
  parseCallback(url: URL): CallbackParams | null {
    // Extract TX from query params (we embedded it in login_redirect_url)
    const tx = url.searchParams.get('tx');
    if (!tx) {
      this.logger.debug('No tx found in Stytch callback');
      return null;
    }

    // Check for Stytch error response
    // Stytch documentation doesn't specify callback error format, so we defensively check both:
    // 1. stytch_error/stytch_error_description - hypothetical Stytch-specific format (prioritized)
    // 2. error/error_description - standard OAuth format (fallback)
    // If Stytch uses standard OAuth params, our fallback handles it. If they use custom params,
    // we catch that too. This defensive approach ensures we don't miss errors regardless of format.
    // TODO: see if we can confirm Stytch's actual error callback format.
    const stytchError = url.searchParams.get('stytch_error') || url.searchParams.get('error');
    if (stytchError) {
      const errorDescription = url.searchParams.get('stytch_error_description') || url.searchParams.get('error_description');
      this.logger.debug('Stytch returned error in callback', { error: stytchError, errorDescription, tx });
      return {
        tx,
        idpError: {
          error: stytchError,
          error_description: errorDescription ?? undefined,
        },
      };
    }

    // Extract token from Stytch's callback format
    const tokenType = url.searchParams.get('stytch_token_type');
    const token =
      tokenType === 'oauth' ? url.searchParams.get('token') : url.searchParams.get('token') || url.searchParams.get('stytch_token');

    if (!token) {
      this.logger.debug('No token found in Stytch callback');
      return null;
    }

    this.logger.debug('Parsed Stytch callback', { tokenType, tx });

    return {
      token,
      tokenType: tokenType || undefined,
      tx,
    };
  }

  /**
   * Authenticate with Stytch's API.
   *
   * Stytch uses a different flow than standard OAuth token exchange:
   * - POST to /oauth/authenticate with the token
   * - Use Basic auth with project_id:secret
   * - Returns user info directly (no separate userinfo endpoint)
   *
   * Note: Stytch doesn't require redirect_uri for token authentication,
   * but we accept it in the context for interface consistency.
   */
  async authenticate(ctx: AuthenticateContext): Promise<AuthenticatedUser> {
    this.logger.debug('Authenticating with Stytch API');

    const response = await fetch(this.getApiUrl('oauth/authenticate'), {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Basic ${btoa(`${this.env.STYTCH_PROJECT_ID}:${this.env.STYTCH_SECRET}`)}`,
      },
      body: JSON.stringify({ token: ctx.token }),
    });

    if (!response.ok) {
      const errorText = await response.text();
      this.logger.debug('Stytch authentication error', { error: errorText, status: response.status });
      throw new Error('Authentication failed');
    }

    const authDataResult = v.safeParse(StytchAuthResponseSchema, await response.json());
    if (!authDataResult.success) {
      this.logger.debug('Invalid Stytch response', { issues: authDataResult.issues });
      throw new Error('Invalid Stytch response: missing required fields');
    }
    const authData = authDataResult.output;

    this.logger.debug('Stytch auth successful', { userId: authData.user_id });

    return {
      sub: authData.user_id,
      email: authData.user?.emails?.[0]?.email,
    };
  }
}

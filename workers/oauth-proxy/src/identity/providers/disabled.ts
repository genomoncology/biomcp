/**
 * Disabled Identity Provider.
 *
 * Used when identity features are not needed (e.g., tests that don't exercise auth flows).
 * All methods throw with a clear error message indicating configuration is required.
 */

import type { IdentityProvider, AuthorizationConfig, CallbackParams, AuthenticateContext, AuthenticatedUser } from '../interface';

/**
 * Identity provider that throws on any operation.
 * Use IDENTITY_PROVIDER=disabled when identity features are not needed.
 */
export class DisabledIdentityProvider implements IdentityProvider {
  readonly name = 'disabled';

  private notConfigured(): never {
    throw new Error('Identity provider is disabled. Set IDENTITY_PROVIDER to "stytch" or "oauth" to use identity features.');
  }

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  async buildAuthorizationUrl(_config: AuthorizationConfig): Promise<URL> {
    this.notConfigured();
  }

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  parseCallback(_url: URL): CallbackParams | null {
    this.notConfigured();
  }

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  async authenticate(_ctx: AuthenticateContext): Promise<AuthenticatedUser> {
    this.notConfigured();
  }
}

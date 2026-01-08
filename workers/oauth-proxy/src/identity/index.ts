/**
 * Identity Provider abstraction module.
 *
 * Provides a pluggable identity provider architecture for the OAuth gateway.
 */

// Core interface
export type { IdentityProvider, AuthenticatedUser, AuthorizationConfig, CallbackParams, AuthenticateContext } from './interface';

// Factory
export { createIdentityProvider, type IdentityProviderType, type IdentityProviderFactoryEnv } from './factory';

// Providers
export { StytchProvider } from './providers/stytch';
export { StandardOAuthProvider } from './providers/oauth';
export { DisabledIdentityProvider } from './providers/disabled';

// Re-export types from env.ts for convenience
export type { StytchEnv, StandardOAuthEnv } from '../env';

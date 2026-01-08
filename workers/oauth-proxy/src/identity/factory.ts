/**
 * Identity Provider factory.
 *
 * Creates the appropriate identity provider based on configuration.
 * Validates environment variables using Valibot schemas for the selected provider.
 */

import * as v from 'valibot';
import type { Logger } from '../utils/logger';
import type { IdentityProvider } from './interface';
import { type IdentityProviderType, type StytchEnv, type StandardOAuthEnv, StytchEnvSchema, StandardOAuthEnvSchema } from '../env';
import { AlwaysStrings } from '../utils/types';
import { StytchProvider } from './providers/stytch';
import { StandardOAuthProvider } from './providers/oauth';
import { DisabledIdentityProvider } from './providers/disabled';

/**
 * Extract missing field names from Valibot parse issues.
 */
function getMissingFields(issues: v.BaseIssue<unknown>[]): string {
  return issues
    .map((i) => i.path?.[0]?.key)
    .filter(Boolean)
    .join(', ');
}

// Re-export for external consumers
export type { IdentityProviderType } from '../env';

/**
 * Environment variables for identity provider factory.
 * Contains identity provider type plus raw (unparsed) env vars for each provider.
 * Factory validates and parses the appropriate schema based on selected provider.
 */
export type IdentityProviderFactoryEnv = {
  IDENTITY_PROVIDER: IdentityProviderType;
} & AlwaysStrings<Partial<StytchEnv> & Partial<StandardOAuthEnv>>;

/**
 * Options for creating an identity provider.
 */
export interface IdentityProviderFactoryOptions {
  /** KV namespace for caching (e.g., OIDC discovery). Optional. */
  kv?: KVNamespace;
}

/**
 * Create an identity provider based on configuration.
 *
 * @param env - Environment variables
 * @param logger - Logger instance
 * @param options - Additional options (KV namespace for caching)
 * @returns Configured identity provider
 * @throws Error if required env vars are missing for selected provider
 */
export function createIdentityProvider(
  env: IdentityProviderFactoryEnv,
  logger: Logger,
  options: IdentityProviderFactoryOptions = {},
): IdentityProvider {
  const providerType = env.IDENTITY_PROVIDER;

  switch (providerType) {
    case 'stytch':
      return createStytchProvider(env, logger);

    case 'oauth':
      return createOAuthProvider(env, logger, options);

    case 'disabled':
      return new DisabledIdentityProvider();

    default:
      throw new Error(`Unknown identity provider: ${providerType}`);
  }
}

/**
 * Create a Stytch provider with validated environment.
 */
function createStytchProvider(env: IdentityProviderFactoryEnv, logger: Logger): StytchProvider {
  const parseResult = v.safeParse(StytchEnvSchema, env);
  if (!parseResult.success) {
    throw new Error(`Missing required Stytch env vars: ${getMissingFields(parseResult.issues)}`);
  }
  return new StytchProvider(parseResult.output, logger);
}

/**
 * Create a Standard OAuth provider with validated environment.
 */
function createOAuthProvider(
  env: IdentityProviderFactoryEnv,
  logger: Logger,
  options: IdentityProviderFactoryOptions,
): StandardOAuthProvider {
  const parseResult = v.safeParse(StandardOAuthEnvSchema, env);
  if (!parseResult.success) {
    throw new Error(`Missing required OAuth env vars: ${getMissingFields(parseResult.issues)}`);
  }
  return new StandardOAuthProvider(parseResult.output, logger, options.kv);
}

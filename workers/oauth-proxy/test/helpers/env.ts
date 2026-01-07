/**
 * Type-safe test environment helpers.
 *
 * Uses real bindings from cloudflare:test (miniflare) - no manual stubs needed.
 * The `env` from cloudflare:test provides all required Env fields (KV, Analytics, etc.)
 *
 * See ai-docs/test-environment.md for the full test environment strategy.
 *
 * ## Drift Prevention
 *
 * This helper stays in sync automatically because:
 * 1. `env` from cloudflare:test is typed as `ProvidedEnv extends Env` (test/env.d.ts)
 * 2. `RawEnv = Env & { optional additions }` (src/env.ts)
 * 3. If Env changes (via `pnpm cf-typegen`), both types update automatically
 *
 * If .dev.vars.test adds new bindings, update test/env.d.ts to match.
 * The TypeScript compiler will catch any mismatches.
 */
import { env } from 'cloudflare:test';
import { parseEnv, type RawEnv, type ParsedEnv } from '../../src/env';

/**
 * Create a RawEnv for testing.
 *
 * All test values come from .dev.vars.test via cloudflare:test.
 * Use overrides to customize specific fields for individual tests.
 *
 * @param overrides - Optional partial RawEnv to override specific fields
 * @returns A complete RawEnv ready for parseEnv()
 *
 * @example
 * // Test with custom auth token
 * const env = createTestEnv({ REMOTE_MCP_AUTH_TOKEN: 'valid-32-char-token-for-testing!' });
 *
 * @example
 * // Test with HTTP URL (for HTTPS enforcement tests)
 * const env = createTestEnv({ REMOTE_MCP_SERVER_URL: 'http://insecure.example.com' });
 */
export function createTestEnv(overrides: Partial<RawEnv> = {}): RawEnv {
  return { ...env, ...overrides };
}

/**
 * Create and parse test environment in one step.
 *
 * Convenient for tests that need ParsedEnv directly without
 * needing to call parseEnv separately.
 *
 * @param overrides - Optional partial RawEnv to override specific fields
 * @returns Parsed environment with proper types
 *
 * @example
 * // Get parsed env with custom token
 * const parsed = parseTestEnv({ REMOTE_MCP_AUTH_TOKEN: 'valid-32-char-token-for-testing!' });
 * expect(parsed.REMOTE_MCP_AUTH_TOKEN).toBe('valid-32-char-token-for-testing!');
 */
export function parseTestEnv(overrides: Partial<RawEnv> = {}): ParsedEnv {
  return parseEnv(createTestEnv(overrides));
}

/**
 * Default Stytch configuration for tests that need identity provider.
 * Use with createTestEnv() to get a complete env with Stytch enabled.
 */
export const stytchTestConfig = {
  IDENTITY_PROVIDER: 'stytch' as const,
  STYTCH_PROJECT_ID: 'project-test-00000000-0000-0000-0000-000000000000',
  STYTCH_SECRET: 'secret-test-00000000000000000000000000000000',
  STYTCH_PUBLIC_TOKEN: 'public-token-test-00000000000000000000000000000000',
  STYTCH_API_URL: 'https://test.stytch.com/v1',
  STYTCH_OAUTH_URL: 'https://test.stytch.com/v1/public/oauth/google/start',
};

/**
 * Create a test environment with Stytch identity provider enabled.
 * Use this for tests that exercise OAuth flows (authorize, callback, etc.).
 *
 * @param overrides - Optional additional overrides
 * @returns RawEnv with Stytch config
 */
export function createStytchTestEnv(overrides: Partial<RawEnv> = {}): RawEnv {
  return createTestEnv({ ...stytchTestConfig, ...overrides });
}

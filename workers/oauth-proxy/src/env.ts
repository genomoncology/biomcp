import * as v from 'valibot';
import { AlwaysStrings, Prettify } from './utils/types';

/**
 * Identity provider types. Required - no default.
 * Explicit configuration prevents accidental security misconfigurations.
 */
const IdentityProviderValues = v.union([v.literal('stytch'), v.literal('oauth'), v.literal('disabled')]);

/**
 * Validation schema for identity provider type.
 * Required field - must be explicitly configured.
 */
const IdentityProviderSchema = v.pipe(
  v.optional(v.string()),
  v.check(
    (s): s is string => s !== undefined && s.trim().length > 0,
    'IDENTITY_PROVIDER is required. ' +
      'Set to "disabled" to run without identity features, ' +
      'or configure "stytch" or "oauth" with their required environment variables.',
  ),
  v.transform((s) => v.parse(IdentityProviderValues, s)),
);

/**
 * Supported identity provider types.
 * Inferred from the validation schema to ensure consistency.
 */
export type IdentityProviderType = v.InferOutput<typeof IdentityProviderSchema>;

/**
 * Validation schema for analytics provider type.
 * Defaults to 'cloudflare'.
 */
const AnalyticsProviderSchema = v.optional(v.union([v.literal('cloudflare'), v.literal('bigquery'), v.literal('none')]), 'cloudflare');

/**
 * Supported analytics provider types.
 * Inferred from the validation schema to ensure consistency.
 */
export type AnalyticsProviderType = v.InferOutput<typeof AnalyticsProviderSchema>;

export const BigQueryEnvironmentSchema = v.object({
  BQ_SA_KEY_JSON: v.string(),
  BQ_PROJECT_ID: v.string(),
  BQ_DATASET: v.string(),
  BQ_TABLE: v.string(),
});

export type BigQueryEnvironment = v.InferOutput<typeof BigQueryEnvironmentSchema>;

/**
 * Schema for URL validation that enforces HTTPS.
 * Allows localhost and 127.0.0.1 for local development.
 */
const HttpsUrl = v.pipe(
  v.string(),
  v.url(),
  v.check((url) => {
    const parsed = new URL(url);
    const isHttps = parsed.protocol === 'https:';
    const isLocalhost = parsed.hostname === 'localhost' || parsed.hostname === '127.0.0.1';
    return isHttps || isLocalhost;
  }, 'URL must use HTTPS (except localhost for local development)'),
);

/**
 * Schema for OIDC issuer URL validation.
 * Extends HttpsUrl with additional constraint: no trailing slash.
 *
 * RFC 8414 §3.2 requires strict string equality between configured and
 * discovered issuer values. Most OIDC providers return issuers without
 * trailing slashes. Rejecting trailing slashes at config time prevents
 * mysterious auth failures from issuer mismatch.
 */
const OidcIssuerUrl = v.pipe(
  HttpsUrl,
  v.check((url) => !url.endsWith('/'), 'OAUTH_ISSUER must not end with a trailing slash'),
);

/**
 * Stytch-specific configuration.
 * Required when IDENTITY_PROVIDER=stytch.
 */
export const StytchEnvSchema = v.object({
  STYTCH_PROJECT_ID: v.string(),
  STYTCH_SECRET: v.string(),
  STYTCH_PUBLIC_TOKEN: v.string(),
  STYTCH_OAUTH_URL: HttpsUrl,
  STYTCH_API_URL: HttpsUrl,
});

export type StytchEnv = v.InferOutput<typeof StytchEnvSchema>;

/**
 * Schema for OAuth scopes configuration.
 * Parses a space-separated string into an array of scopes.
 * Defaults to standard OIDC scopes for user identity.
 */
const OAuthScopes = v.pipe(
  v.optional(v.string(), 'openid email profile'),
  v.transform((s) =>
    s
      .split(/\s+/)
      .map((scope) => scope.trim())
      .filter((scope) => scope.length > 0),
  ),
  v.check((arr) => arr.length > 0, 'OAUTH_SCOPES must contain at least one scope'),
);

/**
 * Standard OAuth 2.0 configuration.
 * Required when IDENTITY_PROVIDER=oauth.
 *
 * Endpoint configuration (two modes):
 * 1. Auto-discovery: Set OAUTH_ISSUER - endpoints discovered from /.well-known/openid-configuration
 * 2. Explicit: Set OAUTH_AUTHORIZATION_URL and OAUTH_TOKEN_URL directly
 *
 * Explicit URLs override discovered values (useful for non-standard providers).
 *
 * User identity resolution (in priority order):
 * 1. OAUTH_USERINFO_URL - fetch user info from dedicated endpoint
 * 2. id_token extraction - cryptographically signed, requires OAUTH_ISSUER
 * 3. Token response 'sub' claim - unsigned convenience field, fallback only
 */
export const StandardOAuthEnvSchema = v.object({
  OAUTH_CLIENT_ID: v.string(),
  OAUTH_CLIENT_SECRET: v.string(),
  // Endpoint URLs - optional if OAUTH_ISSUER is set for auto-discovery
  // All URLs must use HTTPS (except localhost for local development)
  OAUTH_AUTHORIZATION_URL: v.optional(HttpsUrl),
  OAUTH_TOKEN_URL: v.optional(HttpsUrl),
  OAUTH_USERINFO_URL: v.optional(HttpsUrl),
  // OIDC issuer for auto-discovery and id_token validation
  OAUTH_ISSUER: v.optional(OidcIssuerUrl),
  // Scopes to request from IdP (space-separated). Defaults to 'openid email profile'.
  OAUTH_SCOPES: OAuthScopes,
});

export type StandardOAuthEnv = v.InferOutput<typeof StandardOAuthEnvSchema>;

/**
 * Raw environment from Cloudflare (defined in worker-configuration.d.ts)
 * extended with optional fields not in wrangler.jsonc.
 *
 * Note: IDENTITY_PROVIDER and ANALYTICS_PROVIDER are `string` in Env
 * (due to --strict-vars false). We validate them in parseEnv() to get
 * proper union types in ParsedEnv.
 */
export type RawEnv = Prettify<
  Env &
    AlwaysStrings<
      {
        // This isn't defined in worker-configuration.d.ts, so we add it here.
        JWT_SECRET?: string;
        // Comma-separated list of allowed origins for CORS.
        ALLOWED_ORIGINS?: string;
        // Bearer token for authenticating to the remote MCP server
        REMOTE_MCP_AUTH_TOKEN?: string;
      } & Partial<BigQueryEnvironment> &
        Partial<StytchEnv> &
        Partial<StandardOAuthEnv>
    >
>;

// TODO: handle ParsedEnv declaration better? It could probably be inferred from parseEnv function directly.

/**
 * Parsed environment with proper types.
 * Consumers should use this instead of raw Env.
 *
 * Note: OAUTH_SCOPES is not parsed here - it's provider-specific and
 * parsed by the identity provider factory when creating the OAuth provider.
 */
export type ParsedEnv = Prettify<
  Omit<RawEnv, 'DEBUG' | 'JWT_SECRET' | 'ALLOWED_ORIGINS' | 'IDENTITY_PROVIDER' | 'ANALYTICS_PROVIDER'> & {
    DEBUG: boolean;
    JWT_SECRET: string;
    ALLOWED_ORIGINS: string[];
    IDENTITY_PROVIDER: IdentityProviderType;
    ANALYTICS_PROVIDER: AnalyticsProviderType;
  }
>;

/**
 * Schema to coerce string or boolean env var to boolean.
 * Recognizes "true" and "1" as truthy for strings, or native boolean values.
 */
const BooleanFromStringOrBool = v.pipe(
  v.optional(v.union([v.string(), v.boolean()])),
  v.transform((s) => {
    if (typeof s === 'boolean') return s;
    return !!s && (s.trim().toLowerCase() === 'true' || s.trim() === '1');
  }),
);

const JWTSecret = v.pipe(v.string('A JWT secret is required'), v.minLength(32, 'JWT secret must be at least 32 characters long'));

const OriginList = v.pipe(
  v.optional(v.string()),
  v.transform((s) =>
    s
      ? s
          .split(',')
          .map((value) => value.trim())
          .filter((value) => value.length > 0)
      : [],
  ),
);

/**
 * Schema for optional auth token with validation.
 * - Empty strings are treated as undefined
 * - Whitespace is rejected (likely misconfiguration)
 * - Tokens must be at least 32 characters for adequate security
 */
const RemoteMcpAuthToken = v.pipe(
  v.optional(v.string()),
  v.transform((s) => {
    if (!s) return undefined;
    const trimmed = s.trim();
    if (trimmed.length === 0) return undefined;
    if (trimmed !== s) {
      throw new Error('REMOTE_MCP_AUTH_TOKEN contains leading/trailing whitespace');
    }
    if (trimmed.length < 32) {
      throw new Error('REMOTE_MCP_AUTH_TOKEN must be at least 32 characters long');
    }
    return trimmed;
  }),
);

/**
 * Parse raw Cloudflare environment to typed environment.
 */
export function parseEnv(raw: RawEnv): ParsedEnv {
  const parsed = {
    ...raw,
    ALLOWED_ORIGINS: v.parse(OriginList, raw.ALLOWED_ORIGINS),
    JWT_SECRET: v.parse(JWTSecret, raw.JWT_SECRET),
    DEBUG: v.parse(BooleanFromStringOrBool, raw.DEBUG),
    REMOTE_MCP_AUTH_TOKEN: v.parse(RemoteMcpAuthToken, raw.REMOTE_MCP_AUTH_TOKEN),
    IDENTITY_PROVIDER: v.parse(IdentityProviderSchema, raw.IDENTITY_PROVIDER),
    ANALYTICS_PROVIDER: v.parse(AnalyticsProviderSchema, raw.ANALYTICS_PROVIDER),
  };

  // Enforce HTTPS when auth token is configured (except localhost for local dev)
  if (parsed.REMOTE_MCP_AUTH_TOKEN && parsed.REMOTE_MCP_SERVER_URL) {
    const url = parsed.REMOTE_MCP_SERVER_URL;
    const isHttp = url.startsWith('http://');
    const isLocalhost = url.startsWith('http://localhost') || url.startsWith('http://127.0.0.1');
    if (isHttp && !isLocalhost) {
      throw new Error('REMOTE_MCP_AUTH_TOKEN requires HTTPS for REMOTE_MCP_SERVER_URL to prevent token leakage');
    }
  }

  // Validate OAUTH_ISSUER if provided (trailing slash causes OIDC discovery mismatch)
  if (raw.OAUTH_ISSUER) {
    parsed.OAUTH_ISSUER = v.parse(OidcIssuerUrl, raw.OAUTH_ISSUER);
  }

  return parsed;
}

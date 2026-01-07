import * as v from 'valibot';
import { TypedKV } from './utils/typed-kv';
import { TOKEN_EXPIRY_SECONDS, REFRESH_TOKEN_EXPIRY_SECONDS } from './constants';

export const createAuthCodeStorage = TypedKV.create(
  'auth-codes',
  v.object({
    sub: v.string(),
    email: v.optional(v.string()),
    code_challenge: v.string(),
    client_id: v.string(),
    redirect_uri: v.string(),
    scope: v.optional(v.string()),
    resource: v.array(v.string()), // RFC 8707: resource indicator(s)
  }),
  { expirationTtl: 300 }, // 5 minutes
);

export const createAccessTokenStorage = TypedKV.create(
  'token',
  v.object({
    sub: v.string(),
    email: v.optional(v.string()),
    client_id: v.string(),
    scope: v.string(),
    resource: v.array(v.string()), // RFC 8707: resource indicator(s)
  }),
  { expirationTtl: TOKEN_EXPIRY_SECONDS },
);

export const createRefreshTokenStorage = TypedKV.create(
  'refresh_token',
  v.object({
    sub: v.string(),
    client_id: v.string(),
    email: v.optional(v.string()),
    scope: v.string(),
    resource: v.array(v.string()), // RFC 8707: resource indicator(s)
  }),
  { expirationTtl: REFRESH_TOKEN_EXPIRY_SECONDS },
);

export const createClientStorage = TypedKV.create(
  'client',
  v.object({
    client_id: v.string(),
    client_name: v.string(),
    redirect_uris: v.array(v.string()),
    client_uri: v.optional(v.string()),
    logo_uri: v.optional(v.string()),
    scope: v.string(),
    // TODO: consider: inactive, suspended, deleted
    status: v.literal('active'),
    token_endpoint_auth_method: v.literal('none'),
    grant_types: v.array(v.string()),
    response_types: v.array(v.string()),
    created_at: v.string(),
  }),
  { expirationTtl: 365 * 24 * 60 * 60 }, // 1 year
);

export const OAuthTxSchema = v.object({
  client_id: v.string(),
  redirect_uri: v.string(),
  code_challenge: v.string(),
  code_challenge_method: v.string(),
  original_state: v.string(),
  scope: v.optional(v.string()),
  resource: v.array(v.string()), // RFC 8707: resource indicator(s)
  nonce: v.optional(v.string()), // For OpenID Connect, not supported yet, but it doesn't hurt to store it
});

export type OAuthTxSchema = v.InferOutput<typeof OAuthTxSchema>;

export const createOAuthTxStorage = TypedKV.create(
  'oauth_tx',
  OAuthTxSchema,
  { expirationTtl: 600 }, // 10 minutes
);

/**
 * Cache for OIDC discovery configuration.
 * Stores endpoints discovered from /.well-known/openid-configuration.
 * Used for auto-discovery of OAuth endpoints and id_token validation.
 */
export const createOidcConfigCache = TypedKV.create(
  'oidc_config',
  v.object({
    issuer: v.string(),
    jwks_uri: v.string(),
    authorization_endpoint: v.optional(v.string()),
    token_endpoint: v.optional(v.string()),
    userinfo_endpoint: v.optional(v.string()),
  }),
  { expirationTtl: 24 * 60 * 60 }, // 24 hours
);

/**
 * MCP session storage for binding sessions to user identity.
 * Prevents session hijacking by ensuring only the session owner can access it.
 */
export const createMcpSessionStorage = TypedKV.create(
  'mcp-session',
  v.object({
    sub: v.string(),
    created_at: v.string(),
  }),
  { expirationTtl: 24 * 60 * 60 }, // 24 hours
);

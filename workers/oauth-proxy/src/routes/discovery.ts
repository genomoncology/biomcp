import { createHono, getOrigin } from '../utils/hono';

// RFC 9728: OAuth 2.0 Protected Resource Metadata
function getProtectedResourceMetadata(origin: string) {
  return {
    resource: `${origin}/mcp`,
    authorization_servers: [origin],
    bearer_methods_supported: ['header'],
    // NOTE: 'mcp' scope for MCP protocol access; OIDC scopes (profile, email) are not supported
    scopes_supported: ['mcp'],
  };
}

// RFC 8414: OAuth 2.0 Authorization Server Metadata
function getAuthServerMetadata(origin: string) {
  return {
    issuer: origin,
    authorization_endpoint: `${origin}/authorize`,
    token_endpoint: `${origin}/token`,
    revocation_endpoint: `${origin}/revoke`,
    registration_endpoint: `${origin}/register`,
    // NOTE: 'mcp' scope for MCP protocol access; OIDC scopes are not supported
    scopes_supported: ['mcp'],
    response_types_supported: ['code'],
    response_modes_supported: ['query'],
    grant_types_supported: ['authorization_code', 'refresh_token'],
    token_endpoint_auth_methods_supported: ['none'],
    code_challenge_methods_supported: ['S256'],
  };
}

// Note: these paths are not currently cached, as they are lightweight and infrequently called.

export function createDiscoveryRoutes() {
  const app = createHono();

  // OAuth protected resource metadata (RFC 9728)
  // Matches base path and path-specific (e.g., /.well-known/oauth-protected-resource/mcp)
  app.get('/.well-known/oauth-protected-resource/*', (c) => {
    return c.json(getProtectedResourceMetadata(getOrigin(c)));
  });

  // OAuth server metadata endpoint
  // Matches base path and path-specific (e.g., /.well-known/oauth-authorization-server/mcp)
  app.get('/.well-known/oauth-authorization-server/*', (c) => {
    return c.json(getAuthServerMetadata(getOrigin(c)));
  });

  return app;
}

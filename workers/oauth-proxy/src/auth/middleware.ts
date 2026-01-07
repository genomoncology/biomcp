import { Context, Next } from 'hono';
import * as v from 'valibot';
import type { JwtValidator } from './jwt';
import type { Logger } from '../utils/logger';
import { getOrigin } from '../utils/hono';

/** Valibot schema for validated JWT claims */
export const AuthClaimsSchema = v.object({
  sub: v.optional(v.string()),
  email: v.optional(v.string()),
  preferred_username: v.optional(v.string()),
});

/** Validated JWT claims type (inferred from schema) */
export type AuthClaims = v.InferOutput<typeof AuthClaimsSchema>;

/** Hono Variables type for routes that use auth middleware */
export type AuthVariables = {
  authClaims?: AuthClaims;
};

/**
 * Normalize a pathname to its base path (first segment only).
 * Examples: /mcp -> /mcp, /mcp/ -> /mcp, /mcp/foo/bar -> /mcp, / -> /
 */
export function normalizeBasePath(pathname: string): string {
  const segments = pathname.split('/').filter(Boolean);
  return segments.length > 0 ? `/${segments[0]}` : '/';
}

/**
 * Compute the RFC 8707 resource URI for the current request.
 * This is the canonical URI that tokens must be authorized for.
 * Format: origin + base path (e.g., "https://gateway.example.com/mcp")
 *
 * Path normalization: /mcp, /mcp/, /mcp/anything -> /mcp
 */
export function getExpectedResource(c: Context): string {
  const url = new URL(c.req.url);
  const basePath = normalizeBasePath(url.pathname);
  return new URL(basePath, url.origin).toString();
}

export function createAuthMiddleware(jwtValidator: JwtValidator, logger: Logger) {
  return async (c: Context, next: Next) => {
    const authHeader = c.req.header('Authorization');
    logger.debug('Auth middleware invoked', { hasAuthHeader: !!authHeader });

    // Get CORS headers from context (set by corsMiddleware)
    const corsHeaders = c.get('corsHeaders') || {};
    const origin = getOrigin(c);

    if (!authHeader || !authHeader.startsWith('Bearer ')) {
      // RFC 6750 Section 3: When no token is provided, only return realm (no error attribute)
      return c.json(
        {
          error: 'invalid_request',
          error_description: 'No access token provided',
        },
        401,
        {
          'WWW-Authenticate': `Bearer realm="${origin}"`,
          ...corsHeaders,
        },
      );
    }

    const accessToken = authHeader.substring(7);

    // RFC 8707: Compute the expected resource URI for audience validation
    const expectedResource = getExpectedResource(c);

    try {
      const result = await jwtValidator.validate(accessToken, { expectedResource });
      logger.debug('Token validation successful', { sub: result.payload.sub });

      // Validate and extract claims using valibot schema
      const claimsResult = v.safeParse(AuthClaimsSchema, result.payload);
      if (!claimsResult.success) {
        logger.debug('JWT claims validation failed', { error: claimsResult.issues[0].message });
        // RFC 6750 Section 3.1: Include error and error_description in WWW-Authenticate
        return c.json(
          {
            error: 'invalid_token',
            error_description: 'Invalid token claims',
          },
          401,
          {
            'WWW-Authenticate': `Bearer realm="${origin}", error="invalid_token", error_description="Invalid token claims"`,
            ...corsHeaders,
          },
        );
      }

      // Store validated claims in context for downstream handlers
      c.set('authClaims', claimsResult.output);
    } catch (error) {
      logger.debug('Token validation failed', { error: String(error) });
      // RFC 6750 Section 3.1: Include error and error_description in WWW-Authenticate
      return c.json(
        {
          error: 'invalid_token',
          error_description: 'Token validation failed',
        },
        401,
        {
          'WWW-Authenticate': `Bearer realm="${origin}", error="invalid_token"`,
          ...corsHeaders,
        },
      );
    }

    return next();
  };
}

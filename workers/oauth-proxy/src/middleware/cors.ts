import type { Context, Next } from 'hono';
import type { ParsedEnv } from '../env';
import { getCorsHeaders } from '../utils/cors';

/** Variables added by CORS middleware */
export type CorsVariables = {
  corsHeaders: Record<string, string>;
};

/**
 * CORS middleware that:
 * 1. Handles OPTIONS preflight requests
 * 2. Stores corsHeaders in context for handlers that return early
 * 3. Adds CORS headers to all responses automatically
 */
export function corsMiddleware() {
  return async (c: Context<{ Bindings: ParsedEnv; Variables: CorsVariables }>, next: Next) => {
    const corsHeaders = getCorsHeaders(c.req.raw, c.env);

    // Handle preflight requests
    if (c.req.method === 'OPTIONS') {
      return c.body(null, 204, corsHeaders);
    }

    // Store in context for handlers that need to return early with errors
    c.set('corsHeaders', corsHeaders);

    await next();

    // Add CORS headers to all responses
    for (const [key, value] of Object.entries(corsHeaders)) {
      c.res.headers.set(key, value);
    }
  };
}

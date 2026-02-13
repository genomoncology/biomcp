import type { Context, MiddlewareHandler } from 'hono';
import { createMiddleware } from 'hono/factory';
import { HTTPException } from 'hono/http-exception';
import type { ParsedEnv } from '../env';

/**
 * Rate limit key function that extracts a unique identifier from the request.
 * Returns the key to rate limit on, or undefined to bypass rate limiting.
 */
export type RateLimitKeyFunc = (c: Context<{ Bindings: ParsedEnv }>) => string | undefined;

/**
 * Default key function using client IP from CF-Connecting-IP header.
 * Falls back to X-Forwarded-For for local development, or undefined if unavailable.
 */
export const ipKey: RateLimitKeyFunc = (c) => {
  return c.req.header('CF-Connecting-IP') || c.req.header('X-Forwarded-For') || undefined;
};

/**
 * Rate limiting middleware for OAuth endpoints using Cloudflare's rate limiting binding.
 *
 * Protects against:
 * - Brute force attacks on authorization codes
 * - Client registration spam
 * - Refresh token enumeration
 *
 * @see https://developers.cloudflare.com/workers/runtime-apis/bindings/rate-limit/
 */
export function rateLimitMiddleware(keyFunc: RateLimitKeyFunc = ipKey): MiddlewareHandler<{ Bindings: ParsedEnv }> {
  return createMiddleware(async (c, next) => {
    const key = keyFunc(c);

    // Fail open if no key available (shouldn't happen in production with CF headers)
    if (!key) {
      await next();
      return;
    }

    const { success } = await c.env.RATE_LIMITER.limit({ key });

    if (!success) {
      throw new HTTPException(429, {
        res: new Response(
          JSON.stringify({
            error: 'rate_limit_exceeded',
            error_description: 'Too many requests. Please try again later.',
          }),
          {
            status: 429,
            headers: {
              'Content-Type': 'application/json',
              'Retry-After': '10',
            },
          },
        ),
      });
    }

    await next();
  });
}

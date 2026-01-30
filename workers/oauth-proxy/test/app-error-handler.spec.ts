/**
 * Tests for the HTTPException response utility and app error handling.
 *
 * Tests that HTTPException responses include CORS headers and preserve
 * their original status codes and custom headers.
 */

import type { Context, Next } from 'hono';
import { Hono } from 'hono';
import { HTTPException } from 'hono/http-exception';
import { describe, it, expect } from 'vitest';
import { createHttpExceptionResponse } from '../src/utils/http-exception';
import { extractErrorMessage } from '../src/utils/error';

type CorsVariables = { corsHeaders: Record<string, string> };

// --- Unit tests for createHttpExceptionResponse ---

describe('createHttpExceptionResponse', () => {
  it('adds CORS headers to HTTPException response', () => {
    const err = new HTTPException(429, {
      res: new Response(JSON.stringify({ error: 'rate_limited' }), {
        status: 429,
        headers: { 'Content-Type': 'application/json', 'Retry-After': '10' },
      }),
    });

    const corsHeaders = {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, POST',
    };

    const response = createHttpExceptionResponse(err, corsHeaders);

    expect(response.status).toBe(429);
    expect(response.headers.get('Access-Control-Allow-Origin')).toBe('*');
    expect(response.headers.get('Access-Control-Allow-Methods')).toBe('GET, POST');
    expect(response.headers.get('Content-Type')).toBe('application/json');
    expect(response.headers.get('Retry-After')).toBe('10');
  });

  it('preserves original status and statusText', async () => {
    const err = new HTTPException(403, {
      res: new Response('Forbidden', {
        status: 403,
        statusText: 'Forbidden',
      }),
    });

    const response = createHttpExceptionResponse(err, {});

    expect(response.status).toBe(403);
    expect(response.statusText).toBe('Forbidden');
    expect(await response.text()).toBe('Forbidden');
  });

  it('handles empty CORS headers', async () => {
    const err = new HTTPException(401, {
      res: new Response(JSON.stringify({ error: 'unauthorized' }), {
        status: 401,
        headers: { 'Content-Type': 'application/json' },
      }),
    });

    const response = createHttpExceptionResponse(err, {});

    expect(response.status).toBe(401);
    expect(response.headers.get('Content-Type')).toBe('application/json');
    const body = await response.json();
    expect(body).toEqual({ error: 'unauthorized' });
  });
});

// --- Integration tests with Hono app ---

/**
 * Simple CORS middleware for testing that doesn't depend on env bindings.
 */
function testCorsMiddleware() {
  return async (c: Context<{ Variables: CorsVariables }>, next: Next) => {
    const corsHeaders = {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, POST, OPTIONS, HEAD',
      'Access-Control-Allow-Headers': 'Authorization, Content-Type',
    };

    if (c.req.method === 'OPTIONS') {
      return c.body(null, 204, corsHeaders);
    }

    c.set('corsHeaders', corsHeaders);
    await next();

    for (const [key, value] of Object.entries(corsHeaders)) {
      c.res.headers.set(key, value);
    }
  };
}

/**
 * Creates a test app using the same error handler pattern as the main app.
 */
function createTestApp() {
  const app = new Hono<{ Variables: CorsVariables }>();

  app.use('*', testCorsMiddleware());

  // Error handler using the extracted utility (same as src/app.ts)
  app.onError((err, c) => {
    if (err instanceof HTTPException) {
      const corsHeaders = c.get('corsHeaders') || {};
      return createHttpExceptionResponse(err, corsHeaders);
    }

    console.error('Unhandled error', extractErrorMessage(err));
    return c.json({ error: 'server_error', error_description: 'An internal error occurred' }, 500);
  });

  // Test routes
  app.get('/rate-limited', () => {
    throw new HTTPException(429, {
      res: new Response(
        JSON.stringify({
          error: 'rate_limit_exceeded',
          error_description: 'Too many requests. Please try again later.',
        }),
        {
          status: 429,
          headers: { 'Content-Type': 'application/json', 'Retry-After': '10' },
        },
      ),
    });
  });

  app.get('/generic-error', () => {
    throw new Error('Something went wrong');
  });

  app.get('/success', (c) => c.json({ message: 'ok' }));

  return app;
}

describe('App Error Handler Integration', () => {
  it('HTTPException responses include CORS headers', async () => {
    const app = createTestApp();

    const response = await app.request('/rate-limited', {
      headers: { Origin: 'https://client.example.com' },
    });

    expect(response.status).toBe(429);
    expect(response.headers.get('Access-Control-Allow-Origin')).toBe('*');
    expect(response.headers.get('Content-Type')).toBe('application/json');
    expect(response.headers.get('Retry-After')).toBe('10');

    const body = await response.json();
    expect(body).toEqual({
      error: 'rate_limit_exceeded',
      error_description: 'Too many requests. Please try again later.',
    });
  });

  it('generic errors return 500 with CORS headers', async () => {
    const app = createTestApp();

    const response = await app.request('/generic-error');

    expect(response.status).toBe(500);
    expect(response.headers.get('Access-Control-Allow-Origin')).toBe('*');

    const body = await response.json();
    expect(body).toEqual({
      error: 'server_error',
      error_description: 'An internal error occurred',
    });
  });

  it('successful responses have CORS headers', async () => {
    const app = createTestApp();

    const response = await app.request('/success');

    expect(response.status).toBe(200);
    expect(response.headers.get('Access-Control-Allow-Origin')).toBe('*');

    const body = await response.json();
    expect(body).toEqual({ message: 'ok' });
  });
});

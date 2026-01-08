import type { Context } from 'hono';
import type { AnalyticsService } from '../analytics/interface';
import type { createAuthMiddleware, AuthVariables } from '../auth/middleware';
import { parseJsonRpcRequest } from '../mcp/schemas';
import { createMcpSessionStorage } from '../storage';
import { extractErrorMessage } from '../utils/error';
import { createHono } from '../utils/hono';
import type { Logger } from '../utils/logger';
import { buildProxyHeaders, proxyPost } from '../utils/proxy';
import { validateSessionId } from '../utils/session';

export interface McpRouteDeps {
  logger: Logger;
  authMiddleware: ReturnType<typeof createAuthMiddleware>;
  analytics: AnalyticsService;
}

/**
 * Extract session ID from request.
 * Per MCP spec, session ID should be in the Mcp-Session-Id header.
 * For backwards compatibility, also check query parameter.
 * Header takes precedence if both are present.
 */
function extractSessionId(c: Context): string | null {
  // Prefer header (per MCP spec)
  const headerSessionId = c.req.header('mcp-session-id');
  if (headerSessionId) {
    return headerSessionId;
  }
  // Fall back to query parameter (backwards compatibility)
  return c.req.query('session_id') ?? null;
}

export function createMcpRoutes(deps: McpRouteDeps) {
  const { logger, authMiddleware, analytics } = deps;
  const app = createHono<AuthVariables>();

  // MCP endpoint (Streamable HTTP transport)
  app.on('HEAD', '/mcp', authMiddleware, (c) => {
    logger.debug('MCP HEAD endpoint hit');
    // For Streamable HTTP, HEAD /mcp should return 204 to indicate the endpoint exists
    return c.body(null, 204);
  });

  app.get('/mcp', authMiddleware, async (c) => {
    logger.debug('MCP GET endpoint hit');
    const REMOTE_MCP_SERVER_URL = c.env.REMOTE_MCP_SERVER_URL;

    // For Streamable HTTP, GET /mcp with session_id initiates event stream
    // Accept session ID from header (per spec) or query param (backwards compat)
    const rawSessionId = extractSessionId(c);
    if (!rawSessionId) {
      return c.body(null, 204);
    }
    const sessionId = validateSessionId(logger, rawSessionId);
    if (!sessionId) {
      return c.json({ error: 'invalid_request', error_description: 'Invalid session_id format' }, 400);
    }

    // Verify session ownership
    const claims = c.get('authClaims');
    const userSub = claims?.sub;
    if (!userSub) {
      logger.debug('Missing sub claim in token', { sessionId });
      return c.json({ error: 'invalid_token', error_description: 'Token missing sub claim' }, 401);
    }

    const sessionStorage = createMcpSessionStorage(c.env.OAUTH_KV);
    const session = await sessionStorage.get(sessionId);
    if (!session) {
      logger.debug('Session not found', { sessionId, userSub });
      return c.json({ error: 'invalid_session', error_description: 'Session not found or expired' }, 403);
    }
    if (session.sub !== userSub) {
      logger.debug('Session ownership mismatch', { sessionId, sessionOwner: session.sub, requestUser: userSub });
      return c.json({ error: 'invalid_session', error_description: 'Session belongs to different user' }, 403);
    }

    // Proxy the GET request to the backend's /mcp endpoint for streaming
    // Session ID is sent via header per MCP spec
    const targetUrl = new URL('/mcp', REMOTE_MCP_SERVER_URL);
    logger.debug('Proxying GET /mcp', { targetUrl, sessionId });

    // Build headers for remote server request
    // Include Last-Event-ID for SSE resumption if provided (HTML Living Standard §9.2)
    const lastEventId = c.req.header('Last-Event-ID');
    const remoteHeaders = buildProxyHeaders({
      accept: 'text/event-stream',
      authToken: c.env.REMOTE_MCP_AUTH_TOKEN,
      sessionId,
      lastEventId,
    });

    try {
      const response = await fetch(targetUrl, {
        method: 'GET',
        headers: remoteHeaders,
      });

      // 204 No Content must not have a body (RFC 7231 §6.3.5)
      if (response.status === 204) {
        return c.body(null, 204);
      }

      // For SSE, we need to stream the response
      if (response.headers.get('content-type')?.includes('text/event-stream')) {
        logger.debug('Streaming SSE response from backend', { sessionId });
        // Return the streamed response directly - CORS middleware will add headers
        // Note: Connection is a hop-by-hop header that proxies must not forward (RFC 2616 §13.5.1).
        // Cloudflare Workers handle connection management automatically.
        return new Response(response.body, {
          status: response.status,
          headers: {
            'Content-Type': 'text/event-stream',
            'Cache-Control': 'no-cache',
          },
        });
      } else {
        // Non-streaming response
        const responseText = await response.text();
        return new Response(responseText, {
          status: response.status,
          headers: {
            'Content-Type': response.headers.get('content-type') || 'text/plain',
          },
        });
      }
    } catch (error) {
      logger.error('Error proxying GET /mcp', { error: extractErrorMessage(error), sessionId });
      return c.json(
        {
          error: 'proxy_error',
          error_description: 'Failed to proxy MCP request',
        },
        502,
      );
    }
  });

  app.post('/mcp', authMiddleware, async (c) => {
    logger.debug('MCP POST endpoint hit');
    const REMOTE_MCP_SERVER_URL = c.env.REMOTE_MCP_SERVER_URL;

    // Get user identity from auth middleware
    const claims = c.get('authClaims');
    const userSub = claims?.sub;
    if (!userSub) {
      logger.debug('Missing sub claim in token');
      return c.json({ error: 'invalid_token', error_description: 'Token missing sub claim' }, 401);
    }

    const sessionStorage = createMcpSessionStorage(c.env.OAUTH_KV);

    // Extract and validate session ID - generate one if not provided (initial connection)
    // Accept session ID from header (per spec) or query param (backwards compat)
    const rawSessionId = extractSessionId(c);
    let sessionId: string;
    if (rawSessionId) {
      const validated = validateSessionId(logger, rawSessionId);
      if (!validated) {
        return c.json({ error: 'invalid_request', error_description: 'Invalid session_id format' }, 400);
      }
      sessionId = validated;

      // Verify session ownership
      const session = await sessionStorage.get(sessionId);
      if (!session) {
        logger.debug('Session not found', { sessionId, userSub });
        return c.json({ error: 'invalid_session', error_description: 'Session not found or expired' }, 403);
      }
      if (session.sub !== userSub) {
        logger.debug('Session ownership mismatch', { sessionId, sessionOwner: session.sub, requestUser: userSub });
        return c.json({ error: 'invalid_session', error_description: 'Session belongs to different user' }, 403);
      }
    } else {
      // New session - generate ID and bind to user
      sessionId = crypto.randomUUID();
      await sessionStorage.put(sessionId, {
        sub: userSub,
        created_at: new Date().toISOString(),
      });
      logger.debug('Created new session', { sessionId, userSub });
    }

    // Get the request body
    const bodyText = await c.req.text();
    logger.debug('MCP POST request', { sessionId, bodyPreview: bodyText.substring(0, 200) });

    // Parse for analytics (failure is non-fatal)
    const parsed = parseJsonRpcRequest(bodyText);
    if (parsed) {
      // Fire-and-forget, don't await
      analytics
        .logEvent({
          method: parsed.method,
          toolName: parsed.toolName,
          requestId: parsed.requestId,
          sessionId,
          userSub,
        })
        .catch((err) => {
          logger.warn('Analytics logging failed', { error: extractErrorMessage(err) });
        });
    }

    // Create new request for proxying
    const newRequest = new Request(c.req.url, {
      method: 'POST',
      headers: c.req.raw.headers,
      body: bodyText,
    });

    // Use the updated proxyPost function that handles SSE properly
    const response = await proxyPost(logger, newRequest, REMOTE_MCP_SERVER_URL, '/mcp', sessionId, {
      authToken: c.env.REMOTE_MCP_AUTH_TOKEN,
    });

    // Add mcp-session-id header so client knows the session for subsequent requests
    // CORS headers will be added by the middleware
    const headers = new Headers(response.headers);
    headers.set('mcp-session-id', sessionId);

    return new Response(response.body, {
      status: response.status,
      headers,
    });
  });

  app.delete('/mcp', authMiddleware, async (c) => {
    logger.debug('MCP DELETE endpoint hit');
    const REMOTE_MCP_SERVER_URL = c.env.REMOTE_MCP_SERVER_URL;

    // Validate session ID from header (per spec) or query param (backwards compat)
    const rawSessionId = extractSessionId(c);
    if (!rawSessionId) {
      return c.json({ error: 'invalid_request', error_description: 'Missing session_id' }, 400);
    }
    const sessionId = validateSessionId(logger, rawSessionId);
    if (!sessionId) {
      return c.json({ error: 'invalid_request', error_description: 'Invalid session_id format' }, 400);
    }

    // Verify session ownership
    const claims = c.get('authClaims');
    const userSub = claims?.sub;
    if (!userSub) {
      logger.debug('Missing sub claim in token', { sessionId });
      return c.json({ error: 'invalid_token', error_description: 'Token missing sub claim' }, 401);
    }

    const sessionStorage = createMcpSessionStorage(c.env.OAUTH_KV);
    const session = await sessionStorage.get(sessionId);
    if (!session) {
      logger.debug('Session not found', { sessionId, userSub });
      return c.json({ error: 'invalid_session', error_description: 'Session not found or expired' }, 403);
    }
    if (session.sub !== userSub) {
      logger.debug('Session ownership mismatch', { sessionId, sessionOwner: session.sub, requestUser: userSub });
      return c.json({ error: 'invalid_session', error_description: 'Session belongs to different user' }, 403);
    }

    // Delete session from KV storage
    await sessionStorage.delete(sessionId);
    logger.debug('Deleted session from KV', { sessionId, userSub });

    // Proxy DELETE to backend
    // Session ID is sent via header per MCP spec
    const targetUrl = new URL('/mcp', REMOTE_MCP_SERVER_URL);
    logger.debug('Proxying DELETE /mcp', { targetUrl, sessionId });

    const remoteHeaders = buildProxyHeaders({
      accept: 'application/json',
      authToken: c.env.REMOTE_MCP_AUTH_TOKEN,
      sessionId,
    });

    try {
      await fetch(targetUrl, {
        method: 'DELETE',
        headers: remoteHeaders,
      });
      // We don't care about the backend response - session is already deleted from our KV
    } catch (error) {
      // Log but don't fail - our session is already cleaned up
      logger.warn('Error proxying DELETE to backend (non-fatal)', { error: extractErrorMessage(error), sessionId });
    }

    // Return 204 No Content on success
    return c.body(null, 204);
  });

  return app;
}

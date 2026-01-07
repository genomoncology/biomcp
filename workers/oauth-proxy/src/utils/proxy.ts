import { extractErrorMessage } from './error';
import type { Logger } from './logger';

export interface ProxyHeaderOptions {
  accept: string;
  authToken?: string;
  contentType?: string;
  /** Session ID to send via Mcp-Session-Id header (per MCP spec) */
  sessionId?: string;
  /** Last-Event-ID for SSE stream resumption (HTML Living Standard §9.2) */
  lastEventId?: string;
}

/**
 * Build headers for proxying requests to remote MCP server.
 * Centralizes User-Agent, Authorization, and session ID header logic.
 */
export function buildProxyHeaders(options: ProxyHeaderOptions): Record<string, string> {
  const headers: Record<string, string> = {
    Accept: options.accept,
    'User-Agent': 'BioMCP-Proxy/1.0',
  };
  if (options.contentType) {
    headers['Content-Type'] = options.contentType;
  }
  if (options.authToken) {
    headers['Authorization'] = `Bearer ${options.authToken}`;
  }
  if (options.sessionId) {
    headers['Mcp-Session-Id'] = options.sessionId;
  }
  if (options.lastEventId) {
    headers['Last-Event-ID'] = options.lastEventId;
  }
  return headers;
}

export interface ProxyOptions {
  /** Bearer token to send to remote server for authentication */
  authToken?: string;
}

/**
 * Proxy POST requests to remote MCP server.
 * Note: CORS headers should be added by the caller or middleware.
 *
 * Security: Request headers (including Authorization) are intentionally NOT logged
 * to prevent token exposure in logs.
 */
export async function proxyPost(logger: Logger, req: Request, remoteServerUrl: string, path: string, sid: string, options?: ProxyOptions) {
  const body = await req.text();
  const targetUrl = new URL(path, remoteServerUrl);

  // Streamable HTTP requires both application/json and text/event-stream
  // The server will decide which format to use based on the response type
  // Session ID is sent via header per MCP spec
  const headers = buildProxyHeaders({
    accept: 'application/json, text/event-stream',
    contentType: 'application/json',
    authToken: options?.authToken,
    sessionId: sid,
  });

  try {
    const response = await fetch(targetUrl, {
      method: 'POST',
      headers,
      body,
    });

    // 204 No Content must not have a body (RFC 7231 §6.3.5)
    if (response.status === 204) {
      return new Response(null, { status: 204 });
    }

    const contentType = response.headers.get('content-type') || '';

    // SSE: stream directly without buffering
    if (contentType.includes('text/event-stream')) {
      logger.debug('Streaming SSE response', { targetUrl, sessionId: sid });
      return new Response(response.body, {
        status: response.status,
        headers: {
          'Content-Type': 'text/event-stream',
          'Cache-Control': 'no-cache',
        },
      });
    }

    // JSON or other: buffer and return
    const responseText = await response.text();
    logger.debug('Proxy response received', { targetUrl, responsePreview: responseText.substring(0, 500) });

    return new Response(responseText, {
      status: response.status,
      headers: { 'Content-Type': contentType || 'application/json' },
    });
  } catch (error) {
    const errorMessage = extractErrorMessage(error);
    logger.error('Proxy fetch error', { error: errorMessage, targetUrl });
    return new Response(JSON.stringify({ error: 'proxy_error', error_description: errorMessage }), {
      status: 502,
      headers: { 'Content-Type': 'application/json' },
    });
  }
}

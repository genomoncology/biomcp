import type { ParsedEnv } from '../env';

const ALLOWED_METHODS = 'GET, POST, OPTIONS, HEAD, DELETE';
const ALLOWED_HEADERS = 'Authorization, Content-Type, Mcp-Session-Id, Last-Event-ID';
const EXPOSE_HEADERS = 'mcp-session-id';

export function getCorsHeaders(req: Request, env: ParsedEnv): Record<string, string> {
  const requestUrl = new URL(req.url);

  // Discovery endpoints (/.well-known/*) are public metadata and should be
  // accessible from any origin. Browser-based OAuth clients need to fetch
  // these before starting the OAuth flow.
  // Include full preflight headers to handle OPTIONS requests properly.
  if (requestUrl.pathname.startsWith('/.well-known/')) {
    return {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, HEAD, OPTIONS',
      'Access-Control-Max-Age': '86400',
    };
  }

  const origin = req.headers.get('Origin');
  if (!origin) return {};

  let originUrl: URL;
  try {
    originUrl = new URL(origin);
  } catch {
    return {};
  }
  const allowedOrigins = new Set<string>([requestUrl.origin]);

  for (const value of env.ALLOWED_ORIGINS) {
    try {
      allowedOrigins.add(new URL(value).origin);
    } catch {
      continue;
    }
  }

  if (!allowedOrigins.has(originUrl.origin)) {
    return {};
  }

  return {
    'Access-Control-Allow-Origin': originUrl.origin,
    'Access-Control-Allow-Methods': ALLOWED_METHODS,
    'Access-Control-Allow-Headers': ALLOWED_HEADERS,
    'Access-Control-Max-Age': '86400',
    'Access-Control-Expose-Headers': EXPOSE_HEADERS,
    Vary: 'Origin',
  };
}

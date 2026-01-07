import { HTTPException } from 'hono/http-exception';

/**
 * Creates a response from an HTTPException, adding CORS headers from context.
 *
 * The CORS middleware stores headers in context but can't add them to HTTPException
 * responses because the error handler returns before CORS middleware's post-processing.
 * This utility preserves the original response while adding CORS headers.
 *
 * @param err - The HTTPException to convert
 * @param corsHeaders - CORS headers to add to the response
 * @returns A new Response with both original and CORS headers
 */
export function createHttpExceptionResponse(err: HTTPException, corsHeaders: Record<string, string>): Response {
  const response = err.getResponse();

  const newHeaders = new Headers(response.headers);
  for (const [key, value] of Object.entries(corsHeaders)) {
    newHeaders.set(key, value);
  }

  return new Response(response.body, {
    status: response.status,
    statusText: response.statusText,
    headers: newHeaders,
  });
}

import type { Context } from 'hono';
import { ContentfulStatusCode } from 'hono/utils/http-status';

export function extractErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  return String(error);
}

export function errorResponse(c: Context, error: string, errorDescription: string, status: ContentfulStatusCode = 400) {
  return c.json({ error, error_description: errorDescription }, status);
}

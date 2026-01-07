import { Hono, type Context } from 'hono';
import type { ParsedEnv } from '../env';
import type { CorsVariables } from '../middleware/cors';

/** Base Hono env type with ParsedEnv bindings and CorsVariables */
export type HonoEnv<V extends Record<string, unknown> = Record<string, never>> = {
  Bindings: ParsedEnv;
  Variables: CorsVariables & V;
};

/** Create a Hono app with ParsedEnv bindings, CorsVariables, and optional additional Variables type */
export function createHono<V extends Record<string, unknown> = Record<string, never>>() {
  return new Hono<HonoEnv<V>>();
}

/** Get the origin from a Hono context */
export function getOrigin(c: Context): string {
  return new URL(c.req.url).origin;
}

import { Hono } from 'hono';
import { HTTPException } from 'hono/http-exception';
import type { AnalyticsService } from './analytics/interface';
import { createAuthMiddleware } from './auth/middleware';
import type { JwtValidator } from './auth/jwt';
import type { ParsedEnv } from './env';
import type { IdentityProvider } from './identity';
import { corsMiddleware, type CorsVariables } from './middleware/cors';
import type { Logger } from './utils/logger';
import { createStatusRoutes, createDiscoveryRoutes, createOAuthRoutes, createMcpRoutes } from './routes';
import { extractErrorMessage } from './utils/error';
import { createHttpExceptionResponse } from './utils/http-exception';

export interface AppDependencies {
  debug: boolean;
  logger: Logger;
  jwtValidator: JwtValidator;
  authMiddleware: ReturnType<typeof createAuthMiddleware>;
  identityProvider: IdentityProvider;
  analytics: AnalyticsService;
}

export function createApp(deps: AppDependencies) {
  const { logger, authMiddleware, identityProvider, analytics } = deps;

  const app = new Hono<{ Bindings: ParsedEnv; Variables: CorsVariables }>();

  // CORS middleware - handles preflight and adds headers to all responses
  app.use('*', corsMiddleware());

  // Error handler
  app.onError((err, c) => {
    // HTTPException has its own response - add CORS headers and return it
    if (err instanceof HTTPException) {
      logger.debug('HTTPException thrown', {
        status: err.status,
        path: c.req.path,
        method: c.req.method,
      });

      const corsHeaders = c.get('corsHeaders') || {};
      return createHttpExceptionResponse(err, corsHeaders);
    }

    logger.error('Unhandled application error', {
      error: extractErrorMessage(err),
      path: c.req.path,
      method: c.req.method,
    });
    return c.json({ error: 'server_error', error_description: 'An internal error occurred' }, 500);
  });

  // Mount sub-apps
  app.route('/', createStatusRoutes({ debug: deps.debug }));
  app.route('/', createDiscoveryRoutes());
  app.route('/', createOAuthRoutes({ logger, identityProvider }));
  app.route('/', createMcpRoutes({ logger, authMiddleware, analytics }));

  // Default 404 response
  app.all('*', (c) => c.json({ error: 'not_found' }, 404));

  return app;
}

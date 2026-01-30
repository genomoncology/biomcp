/**
 * BioMCP Worker – OAuth Gateway with pluggable identity providers
 */

import { createAnalytics } from './analytics/factory';
import { createLogger } from './utils/logger';
import { JwtValidator } from './auth/jwt';
import { createAuthMiddleware } from './auth/middleware';
import { createApp } from './app';
import { parseEnv } from './env';
import { createIdentityProvider } from './identity';

// Export the app as the main worker fetch handler
export default {
  async fetch(request, _env, ctx): Promise<Response> {
    const env = parseEnv(_env);
    const logger = createLogger(env.DEBUG, 'worker');
    const url = new URL(request.url);
    const jwtValidator = new JwtValidator(logger.child('jwt'), {
      jwtSecret: env.JWT_SECRET,
      oauthKv: env.OAUTH_KV,
      validIssuers: [url.origin],
    });
    const authMiddleware = createAuthMiddleware(jwtValidator, logger.child('auth'));
    const identityProvider = createIdentityProvider(env, logger.child('identity'), { kv: env.OAUTH_KV });
    const analytics = createAnalytics(env, logger);

    const app = createApp({
      debug: env.DEBUG,
      logger: logger.child('app'),
      authMiddleware,
      jwtValidator,
      identityProvider,
      analytics,
    });

    return app.fetch(request, env, ctx);
  },
} satisfies ExportedHandler<Env>;

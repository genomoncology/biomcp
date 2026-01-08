import { createHono } from '../utils/hono';

export interface StatusRouteDeps {
  debug: boolean;
}

export function createStatusRoutes({ debug }: StatusRouteDeps) {
  const app = createHono();

  // Only add debug endpoint if debug mode is enabled
  if (debug) {
    app.get('/debug', (c) => {
      const REMOTE_MCP_SERVER_URL = c.env.REMOTE_MCP_SERVER_URL;
      return c.json({
        worker: 'BioMCP-Proxy',
        remote: REMOTE_MCP_SERVER_URL,
        forwardPath: '/mcp',
        resourceEndpoint: null,
        debug: true,
      });
    });
  }

  return app;
}

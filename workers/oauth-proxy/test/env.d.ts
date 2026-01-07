declare module 'cloudflare:test' {
  // Extends base Env with test-specific bindings from .dev.vars.test
  interface ProvidedEnv extends Env {
    JWT_SECRET: string;
    REMOTE_MCP_AUTH_TOKEN?: string;
    REMOTE_MCP_SERVER_URL: string;
    IDENTITY_PROVIDER: string;
  }
}

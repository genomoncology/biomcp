# Deployment Guide

This guide covers deploying the OAuth gateway and MCP proxy to Cloudflare Workers.

## Prerequisites

- Cloudflare account
- Wrangler CLI (`pnpm add -g wrangler`)
- Project configured (see [Setup Guide](setup.md))

## Deployment Steps

### 1. Authenticate with Cloudflare

```bash
wrangler login
```

### 2. Create KV Namespace

The worker requires a KV namespace for storing OAuth data:

```bash
wrangler kv:namespace create OAUTH_KV
```

Note the namespace ID from the output. Update `wrangler.jsonc`:

```jsonc
{
  "kv_namespaces": [
    {
      "binding": "OAUTH_KV",
      "id": "your-production-namespace-id",
    },
  ],
}
```

### 3. Configure Secrets

Secrets should never be in `wrangler.jsonc`. Use the Cloudflare dashboard or CLI:

```bash
# Required
wrangler secret put JWT_SECRET

# Identity provider secrets (choose based on your provider)
wrangler secret put STYTCH_SECRET
# or
wrangler secret put OAUTH_CLIENT_SECRET

# Optional: Backend MCP server auth
wrangler secret put REMOTE_MCP_AUTH_TOKEN
```

### 4. Configure Environment Variables

Update `wrangler.jsonc` with your production values:

```jsonc
{
  "vars": {
    "REMOTE_MCP_SERVER_URL": "https://your-mcp-backend.example.com",
    "IDENTITY_PROVIDER": "stytch",
    "STYTCH_PROJECT_ID": "project-live-xxxxxxxx",
    "STYTCH_PUBLIC_TOKEN": "public-token-live-xxxxxxxx",
    "STYTCH_API_URL": "https://api.stytch.com/v1",
    "STYTCH_OAUTH_URL": "https://api.stytch.com/v1/public/oauth/google/start",
    "ALLOWED_ORIGINS": "https://your-app.example.com",
  },
}
```

### 5. Deploy

```bash
pnpm deploy
```

The worker URL will be displayed (e.g., `https://worker-mcp-proxy.your-account.workers.dev`).

## Custom Domains

To use a custom domain:

1. Go to your worker in the Cloudflare dashboard
2. Navigate to **Triggers** > **Custom Domains**
3. Add your domain (must be proxied through Cloudflare)

Update your identity provider's redirect URI to use the custom domain.

## Production Checklist

### Security

- [ ] `JWT_SECRET` is a strong, unique secret (32+ characters)
- [ ] All secrets are set via `wrangler secret put`, not in config files
- [ ] `REMOTE_MCP_AUTH_TOKEN` is set if backend requires auth
- [ ] `ALLOWED_ORIGINS` is configured for CORS
- [ ] Identity provider redirect URIs point to production URL

### Identity Provider

- [ ] Using production/live credentials (not test/sandbox)
- [ ] Redirect URI registered with `tx={}` parameter: `https://your-domain/callback?tx={}`
- [ ] OAuth scopes configured appropriately
- [ ] (Stytch) Using a B2C Project (Consumer Authentication)

### Monitoring

- [ ] Enable Cloudflare Analytics (default)
- [ ] Consider BigQuery analytics for detailed logging
- [ ] Set up alerting for error rates

## Environment-Specific Deployments

For multiple environments (staging, production), use wrangler environments:

```jsonc
// wrangler.jsonc
{
  "name": "worker-mcp-proxy",
  "env": {
    "staging": {
      "vars": {
        "REMOTE_MCP_SERVER_URL": "https://staging-mcp.example.com",
      },
      "kv_namespaces": [{ "binding": "OAUTH_KV", "id": "staging-namespace-id" }],
    },
    "production": {
      "vars": {
        "REMOTE_MCP_SERVER_URL": "https://mcp.example.com",
      },
      "kv_namespaces": [{ "binding": "OAUTH_KV", "id": "production-namespace-id" }],
    },
  },
}
```

Deploy to specific environment:

```bash
wrangler deploy --env staging
wrangler deploy --env production
```

Set secrets per environment:

```bash
wrangler secret put JWT_SECRET --env production
```

## Rate Limiting

Rate limiting is configured in `wrangler.jsonc`:

```jsonc
{
  "ratelimits": [
    {
      "name": "RATE_LIMITER",
      "namespace_id": "1001",
      "simple": { "limit": 25, "period": 10 },
    },
  ],
}
```

Adjust `limit` (requests) and `period` (seconds, must be 10 or 60) based on your traffic patterns.

## Troubleshooting

### Check Logs

```bash
wrangler tail
```

### Verify Deployment

```bash
# Check discovery endpoint
curl https://your-worker.workers.dev/.well-known/oauth-authorization-server/mcp

# Check debug endpoint (if DEBUG=true)
curl https://your-worker.workers.dev/debug
```

### Common Issues

**"KV namespace not found"**

- Verify `kv_namespaces` in `wrangler.jsonc` has correct ID
- Run `wrangler kv:namespace list` to see available namespaces

**"Invalid JWT secret"**

- Ensure `JWT_SECRET` is at least 32 characters
- Verify secret is set: `wrangler secret list`

**"Identity provider error"**

- Check redirect URI matches exactly (including trailing slash)
- Verify using production credentials, not test/sandbox

**"CORS errors"**

- Add your frontend origin to `ALLOWED_ORIGINS`
- Include protocol: `https://app.example.com`, not just `app.example.com`

## Rollback

To rollback to a previous deployment:

```bash
# List recent deployments
wrangler deployments list

# Rollback to specific deployment
wrangler rollback <deployment-id>
```

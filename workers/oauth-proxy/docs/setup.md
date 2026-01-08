# Setup Guide

This guide covers local development setup, environment configuration, and identity provider integration.

## Prerequisites

- Node.js 18+
- pnpm (`npm install -g pnpm`)
- Cloudflare account (for deployment)
- Identity provider account (Stytch or any OAuth/OIDC provider)

## Local Development

### 1. Install Dependencies

```bash
git clone <repository-url>
cd worker-mcp-proxy
pnpm install
```

### 2. Create Local Configuration

Copy the example configuration and edit with your values:

```bash
cp .dev.vars.example .dev.vars
```

Edit `.dev.vars` with your identity provider credentials. See the example file for all available options.

### 3. Start Development Server

```bash
pnpm dev
```

The server starts at `http://localhost:8787`.

## Environment Variables

### Required Variables

| Variable                | Description                                              |
| ----------------------- | -------------------------------------------------------- |
| `JWT_SECRET`            | Secret for signing JWTs. Must be at least 32 characters. |
| `REMOTE_MCP_SERVER_URL` | Backend MCP server URL (e.g., `http://localhost:8000`)   |
| `IDENTITY_PROVIDER`     | Identity provider to use: `stytch` (default) or `oauth`  |

### Stytch Provider

When `IDENTITY_PROVIDER=stytch`:

| Variable              | Description                                         |
| --------------------- | --------------------------------------------------- |
| `STYTCH_PROJECT_ID`   | Project ID from Stytch dashboard                    |
| `STYTCH_SECRET`       | API secret key                                      |
| `STYTCH_PUBLIC_TOKEN` | Public token for hosted login UI                    |
| `STYTCH_API_URL`      | API base URL (`https://test.stytch.com/v1` for dev) |
| `STYTCH_OAUTH_URL`    | OAuth start URL (e.g., `.../oauth/google/start`)    |

### Generic OAuth Provider

When `IDENTITY_PROVIDER=oauth`:

| Variable                  | Required | Description                                            |
| ------------------------- | -------- | ------------------------------------------------------ |
| `OAUTH_CLIENT_ID`         | Yes      | OAuth client ID                                        |
| `OAUTH_CLIENT_SECRET`     | Yes      | OAuth client secret                                    |
| `OAUTH_ISSUER`            | Yes\*    | OIDC issuer URL for auto-discovery (no trailing slash) |
| `OAUTH_AUTHORIZATION_URL` | No       | Explicit authorization endpoint (overrides discovery)  |
| `OAUTH_TOKEN_URL`         | No       | Explicit token endpoint (overrides discovery)          |
| `OAUTH_USERINFO_URL`      | No       | Explicit userinfo endpoint (overrides discovery)       |

\*If `OAUTH_ISSUER` is provided, endpoints are auto-discovered via `/.well-known/openid-configuration`.

### Optional Variables

| Variable                | Default      | Description                                          |
| ----------------------- | ------------ | ---------------------------------------------------- |
| `DEBUG`                 | `false`      | Enable debug logging and `/debug` endpoint           |
| `ALLOWED_ORIGINS`       | (none)       | Comma-separated CORS origins                         |
| `REMOTE_MCP_AUTH_TOKEN` | (none)       | Bearer token for backend MCP server (HTTPS required) |
| `ANALYTICS_PROVIDER`    | `cloudflare` | Analytics: `cloudflare`, `bigquery`, or `none`       |

### Backend MCP Server Authentication

If your backend MCP server requires authentication, configure `REMOTE_MCP_AUTH_TOKEN`. This token is used by the proxy to authenticate requests to the backend—it is **not** the same as the tokens used by clients to authenticate with the proxy.

**How it works:**

```text
Client                    Proxy                      Backend MCP Server
  │                         │                              │
  │ Authorization: Bearer   │                              │
  │ <client-access-token>   │                              │
  ├────────────────────────>│                              │
  │                         │                              │
  │                         │  Authorization: Bearer       │
  │                         │  <REMOTE_MCP_AUTH_TOKEN>     │
  │                         ├─────────────────────────────>│
  │                         │                              │
```

The proxy validates the client's access token (issued during OAuth flow), then replaces it with `REMOTE_MCP_AUTH_TOKEN` when forwarding requests to the backend. This allows you to:

- Use a single, long-lived token for backend authentication
- Keep backend credentials secret from clients
- Rotate backend tokens independently of client tokens

**Requirements:**

| Requirement    | Details                                                             |
| -------------- | ------------------------------------------------------------------- |
| Minimum length | 32 characters                                                       |
| HTTPS required | When set, `REMOTE_MCP_SERVER_URL` must use HTTPS (except localhost) |
| No whitespace  | Leading/trailing whitespace is rejected                             |

**Example:**

```bash
# .dev.vars (local development - localhost is exempt from HTTPS requirement)
REMOTE_MCP_AUTH_TOKEN=your-secure-backend-token-at-least-32-characters
REMOTE_MCP_SERVER_URL=http://localhost:8000

# Production (must use HTTPS)
REMOTE_MCP_AUTH_TOKEN=your-secure-backend-token-at-least-32-characters
REMOTE_MCP_SERVER_URL=https://mcp-backend.example.com
```

For production, set this as a secret via `wrangler secret put REMOTE_MCP_AUTH_TOKEN`.

> **Important: Backend Must Validate the Token**
>
> The token is only useful if your backend validates it. Without validation, the token provides no security benefit.
>
> **Options:**
>
> 1. **Application-level**: Your MCP server checks `Authorization: Bearer <token>`
> 2. **Reverse proxy**: nginx, Cloudflare Access, or API gateway validates
> 3. **Private network**: Backend only accessible from the proxy (VPC, Cloudflare Tunnel)
>
> See [Backend Authentication Improvements](../future/backend-authentication-improvements.md) for future enhancement options.

### BigQuery Analytics

When `ANALYTICS_PROVIDER=bigquery`:

| Variable         | Description              |
| ---------------- | ------------------------ |
| `BQ_SA_KEY_JSON` | Service account key JSON |
| `BQ_PROJECT_ID`  | BigQuery project ID      |
| `BQ_DATASET`     | Dataset name             |
| `BQ_TABLE`       | Table name               |

## Identity Provider Setup

### Stytch

1. Create a **B2C Project** at [stytch.com](https://stytch.com) (Consumer Authentication, not B2B)
2. Navigate to **API Keys** to get your project ID and secrets
3. Configure OAuth providers (Google, GitHub, etc.) in the dashboard
4. **Important:** Configure redirect URLs in **Redirect URLs** settings. The URL **must** include the `tx={}` query parameter:
   - Development: `http://localhost:8787/callback?tx={}`
   - Production: `https://your-worker.workers.dev/callback?tx={}`

> **Note:** The `tx={}` parameter is required for OAuth state coordination. Without it, the callback will fail.

### Generic OAuth (OIDC)

Works with any OpenID Connect compliant provider:

1. Register an OAuth application with your provider
2. Set the redirect URI to `https://your-worker.workers.dev/callback`
3. Note the client ID and secret
4. Find the issuer URL (usually the base URL of your provider)

**Example providers:**

- Auth0: `https://your-tenant.auth0.com`
- Okta: `https://your-org.okta.com`
- Google: `https://accounts.google.com`
- Azure AD: `https://login.microsoftonline.com/{tenant}/v2.0`

## KV Namespace

The worker uses Cloudflare KV for storage. For local development, wrangler automatically provides a simulated KV namespace.

For production, create and bind a KV namespace:

```bash
# Create namespace
wrangler kv:namespace create OAUTH_KV

# Add the returned ID to wrangler.jsonc
```

Update `wrangler.jsonc`:

```jsonc
{
  "kv_namespaces": [
    {
      "binding": "OAUTH_KV",
      "id": "your-namespace-id",
    },
  ],
}
```

## Verification

After setup, verify everything works:

```bash
# Run tests
pnpm test

# Type check
pnpm check

# Start dev server
pnpm dev

# Test discovery endpoint
curl http://localhost:8787/.well-known/oauth-authorization-server/mcp
```

## Next Steps

- [API Reference](api.md) - Learn about available endpoints
- [Deployment Guide](deployment.md) - Deploy to production

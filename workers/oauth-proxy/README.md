# BioMCP OAuth Proxy

A TypeScript-based OAuth 2.0 gateway and MCP proxy for BioMCP, built on Cloudflare Workers.

> **Note:** This TypeScript implementation replaces `worker_entry_stytch.js`. The old JavaScript worker is deprecated but kept for reference during migration.

## Overview

This worker provides a secure authentication layer for BioMCP's MCP server. It handles OAuth 2.0 authorization flows, issues JWT tokens, and proxies authenticated requests to the backend MCP server.

**Key capabilities:**

- OAuth 2.0 Authorization Code flow with PKCE
- Dynamic client registration (RFC 7591)
- Token refresh with rotation
- Pluggable identity providers (Stytch, generic OAuth/OIDC)
- MCP request proxying with Streamable HTTP transport
- Secure backend authentication
- Rate limiting via Cloudflare bindings
- BigQuery analytics integration

## Breaking Changes from Old Worker

- **Removed `/messages*` endpoint** - Only Streamable HTTP (`/mcp`) is supported. Legacy SSE mode has been dropped.

## Quick Start

```bash
# From this directory (workers/oauth-proxy)
cd workers/oauth-proxy

# Install dependencies
pnpm install

# Configure environment (see docs/setup.md)
cp .dev.vars.example .dev.vars

# Start development server
pnpm dev
```

The server runs at `http://localhost:8787`.

## Documentation

| Document                                           | Description                         |
| -------------------------------------------------- | ----------------------------------- |
| [Setup Guide](docs/setup.md)                       | Local development and configuration |
| [API Reference](docs/api.md)                       | Endpoint documentation              |
| [Deployment Guide](docs/deployment.md)             | Production deployment               |
| [Test Environment](docs/test-environment.md)       | Test configuration patterns         |

## Configuration

### Required Environment Variables

| Variable                | Description                              |
| ----------------------- | ---------------------------------------- |
| `JWT_SECRET`            | Secret for signing JWTs (32+ characters) |
| `REMOTE_MCP_SERVER_URL` | Backend MCP server URL                   |
| `IDENTITY_PROVIDER`     | `stytch`, `oauth`, or `disabled`         |

See [docs/setup.md](docs/setup.md) for complete configuration options.

## API Endpoints

### OAuth

| Endpoint         | Description                 |
| ---------------- | --------------------------- |
| `POST /register` | Dynamic client registration |
| `GET /authorize` | Start authorization flow    |
| `POST /token`    | Exchange code for tokens    |
| `POST /revoke`   | Revoke tokens               |

### MCP Proxy (Streamable HTTP)

| Endpoint      | Description             |
| ------------- | ----------------------- |
| `POST /mcp`   | Send MCP request        |
| `GET /mcp`    | Stream MCP events (SSE) |
| `DELETE /mcp` | Terminate session       |

### Discovery

| Endpoint                                        | Description                 |
| ----------------------------------------------- | --------------------------- |
| `GET /.well-known/oauth-authorization-server/*` | OAuth server metadata       |
| `GET /.well-known/oauth-protected-resource/*`   | Protected resource metadata |

See [docs/api.md](docs/api.md) for complete API documentation.

## Development Commands

```bash
pnpm dev          # Start local server
pnpm test         # Run tests
pnpm check        # TypeScript check
pnpm lint         # ESLint
pnpm format:check # Prettier check
pnpm deploy       # Deploy to Cloudflare
```

## Migration from worker_entry_stytch.js

The environment variables are compatible with the old worker. Key differences:

1. **TypeScript** - Full type safety with Valibot validation
2. **Pluggable IdP** - `IDENTITY_PROVIDER` can be `stytch`, `oauth`, or `disabled`
3. **Streamable HTTP only** - Legacy `/messages*` endpoint removed
4. **Rate limiting** - Built-in via Cloudflare bindings
5. **Test coverage** - Comprehensive test suite

## RFC Compliance

| RFC      | Title                              |
| -------- | ---------------------------------- |
| RFC 6749 | OAuth 2.0 Authorization Framework  |
| RFC 6750 | Bearer Token Usage                 |
| RFC 7009 | Token Revocation                   |
| RFC 7591 | Dynamic Client Registration        |
| RFC 7636 | Proof Key for Code Exchange (PKCE) |
| RFC 8414 | Authorization Server Metadata      |
| RFC 8707 | Resource Indicators                |
| RFC 9728 | Protected Resource Metadata        |

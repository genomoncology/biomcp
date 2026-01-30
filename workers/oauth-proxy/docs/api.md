# API Reference

This document covers all endpoints exposed by the OAuth gateway and MCP proxy.

## Authentication

Protected endpoints require a Bearer token in the `Authorization` header:

```text
Authorization: Bearer <access_token>
```

Tokens are obtained via the OAuth flow (see [OAuth Endpoints](#oauth-endpoints)).

## Discovery Endpoints

### GET /.well-known/oauth-authorization-server/{path}

Returns OAuth 2.0 Authorization Server Metadata (RFC 8414).

**Response:**

```json
{
  "issuer": "https://your-worker.workers.dev",
  "authorization_endpoint": "https://your-worker.workers.dev/authorize",
  "token_endpoint": "https://your-worker.workers.dev/token",
  "registration_endpoint": "https://your-worker.workers.dev/register",
  "revocation_endpoint": "https://your-worker.workers.dev/revoke",
  "response_types_supported": ["code"],
  "grant_types_supported": ["authorization_code", "refresh_token"],
  "code_challenge_methods_supported": ["S256"],
  "token_endpoint_auth_methods_supported": ["none"],
  "scopes_supported": ["mcp"]
}
```

### GET /.well-known/oauth-protected-resource/{path}

Returns Protected Resource Metadata (RFC 9728).

**Response:**

```json
{
  "resource": "https://your-worker.workers.dev/mcp",
  "authorization_servers": ["https://your-worker.workers.dev"],
  "bearer_methods_supported": ["header"],
  "scopes_supported": ["mcp"]
}
```

## OAuth Endpoints

All OAuth endpoints are rate-limited (25 requests per 10 seconds per IP).

### POST /register

Dynamically register a new OAuth client (RFC 7591).

**Request:**

```json
{
  "redirect_uris": ["https://your-app.example.com/callback"],
  "client_name": "My App",
  "client_uri": "https://your-app.example.com"
}
```

**Response (201 Created):**

```json
{
  "client_id": "uuid-client-id",
  "client_id_issued_at": 1704067200,
  "redirect_uris": ["https://your-app.example.com/callback"],
  "client_name": "My App",
  "client_uri": "https://your-app.example.com"
}
```

### GET /authorize

Initiate the authorization flow. Redirects to the identity provider.

**Query Parameters:**

| Parameter               | Required | Description                                               |
| ----------------------- | -------- | --------------------------------------------------------- |
| `response_type`         | Yes      | Must be `code`                                            |
| `client_id`             | Yes      | Client ID from registration                               |
| `redirect_uri`          | Yes      | Must match registered URI                                 |
| `state`                 | Yes      | CSRF protection token (opaque string)                     |
| `code_challenge`        | Yes      | PKCE challenge (base64url-encoded SHA256)                 |
| `code_challenge_method` | Yes      | Must be `S256`                                            |
| `scope`                 | No       | Space-separated scopes (only `mcp` supported)             |
| `resource`              | Yes      | Resource being accessed (must be the `/mcp` endpoint URL) |

**Example:**

```text
GET /authorize?
  response_type=code&
  client_id=abc123&
  redirect_uri=https://app.example.com/callback&
  state=xyz789&
  code_challenge=E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM&
  code_challenge_method=S256&
  resource=https://your-worker.workers.dev/mcp
```

### POST /token

Exchange authorization code for tokens, or refresh an access token.

#### Authorization Code Grant

**Request (application/x-www-form-urlencoded):**

```text
grant_type=authorization_code&
code=AUTH_CODE&
redirect_uri=https://app.example.com/callback&
client_id=abc123&
code_verifier=dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk&
resource=https://your-worker.workers.dev/mcp
```

**Response:**

```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "dGhpcyBpcyBhIHJlZnJlc2ggdG9rZW4...",
  "scope": "mcp"
}
```

#### Refresh Token Grant

**Request:**

```text
grant_type=refresh_token&
refresh_token=REFRESH_TOKEN&
client_id=abc123
```

**Response:** Same format as authorization code grant. A new refresh token is issued (token rotation).

### POST /revoke

Revoke an access or refresh token (RFC 7009).

**Request (application/x-www-form-urlencoded):**

```text
token=TOKEN_TO_REVOKE&
token_type_hint=access_token
```

**Response:** `200 OK` (always, even if token was invalid)

## MCP Endpoints

All MCP endpoints require Bearer token authentication.

### HEAD /mcp

Health check endpoint.

**Response:** `204 No Content`

### POST /mcp

Send an MCP request to the backend server.

**Headers:**

| Header           | Required | Description                        |
| ---------------- | -------- | ---------------------------------- |
| `Authorization`  | Yes      | Bearer token                       |
| `Content-Type`   | Yes      | `application/json`                 |
| `Mcp-Session-Id` | No       | Session ID (for existing sessions) |

**Request Body:** MCP JSON-RPC request

```json
{
  "jsonrpc": "2.0",
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": {}
  },
  "id": 1
}
```

**Response:** MCP JSON-RPC response from backend

**Response Headers:**

| Header           | Description                                    |
| ---------------- | ---------------------------------------------- |
| `mcp-session-id` | Session ID (save this for subsequent requests) |
| `Content-Type`   | `application/json`                             |

### GET /mcp

Stream MCP events via Server-Sent Events (SSE).

**Headers:**

| Header           | Required | Description               |
| ---------------- | -------- | ------------------------- |
| `Authorization`  | Yes      | Bearer token              |
| `Mcp-Session-Id` | Yes      | Session ID from POST /mcp |

**Query Parameters:**

| Parameter    | Required | Description           |
| ------------ | -------- | --------------------- |
| `session_id` | Alt      | Alternative to header |

**Response:** SSE stream from backend

```text
event: message
data: {"jsonrpc":"2.0","method":"notification","params":{}}

event: message
data: {"jsonrpc":"2.0","result":{},"id":2}
```

### DELETE /mcp

Terminate an MCP session.

**Headers:**

| Header           | Required | Description             |
| ---------------- | -------- | ----------------------- |
| `Authorization`  | Yes      | Bearer token            |
| `Mcp-Session-Id` | Yes      | Session ID to terminate |

**Response:** `204 No Content`

## Error Responses

### OAuth Errors

OAuth endpoints return errors per RFC 6749:

```json
{
  "error": "invalid_request",
  "error_description": "Missing required parameter: client_id"
}
```

Common error codes:

| Code                     | Description                             |
| ------------------------ | --------------------------------------- |
| `invalid_request`        | Missing or invalid parameter            |
| `invalid_client`         | Unknown or inactive client              |
| `invalid_grant`          | Invalid, expired, or revoked code/token |
| `unauthorized_client`    | Client not authorized for grant type    |
| `unsupported_grant_type` | Grant type not supported                |
| `invalid_scope`          | Invalid or unsupported scope            |

### Authentication Errors

Protected endpoints return `401 Unauthorized` with `WWW-Authenticate` header:

```text
WWW-Authenticate: Bearer error="invalid_token", error_description="Token has expired"
```

### Rate Limiting

When rate limited, endpoints return `429 Too Many Requests`:

```json
{
  "error": "rate_limit_exceeded",
  "error_description": "Too many requests"
}
```

## Session Management

MCP sessions are bound to the authenticated user:

1. First `POST /mcp` creates a new session
2. Server returns `mcp-session-id` header
3. Include this ID in subsequent requests (header or query param)
4. Sessions are validated for ownership (user can only access their own sessions)
5. `DELETE /mcp` terminates a session

## CORS

Cross-origin requests are supported when `ALLOWED_ORIGINS` is configured:

- Allowed headers: `Authorization`, `Content-Type`, `Mcp-Session-Id`, `Last-Event-ID`
- Exposed headers: `mcp-session-id`
- Credentials: Allowed

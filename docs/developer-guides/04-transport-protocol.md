# Transport Protocol Guide

This guide explains BioMCP's transport protocol options, with a focus on the new Streamable HTTP transport that provides better scalability and reliability for production deployments.

## Overview

BioMCP supports multiple transport protocols to accommodate different deployment scenarios:

| Transport           | Use Case                                     | Endpoint | Protocol Version |
| ------------------- | -------------------------------------------- | -------- | ---------------- |
| **STDIO**           | Local development, direct Claude integration | N/A      | All              |
| **Worker/SSE**      | Legacy cloud deployments                     | `/sse`   | Pre-2025         |
| **Streamable HTTP** | Modern cloud deployments                     | `/mcp`   | 2025-03-26+      |

## Streamable HTTP Transport

### What is Streamable HTTP?

Streamable HTTP is the latest MCP transport protocol (specification version 2025-03-26) that provides:

- **Single endpoint** (`/mcp`) for all operations
- **Dynamic response modes**: JSON for quick operations, SSE for long-running tasks
- **Session management** via `session_id` query parameter
- **Better scalability**: No permanent connections required
- **Automatic reconnection** and session recovery

### Architecture

The Streamable HTTP transport follows this flow:

1. **MCP Client** sends POST request to `/mcp` endpoint
2. **BioMCP Server** processes the request
3. **Response Type** determined by operation:
   - Quick operations return JSON response
   - Long operations return SSE stream
4. **Session Management** maintains state via session_id parameter

### Implementation Details

BioMCP leverages FastMCP's native streamable HTTP support:

```python
# In core.py
mcp_app = FastMCP(
    name="BioMCP",
    stateless_http=True,  # Enables streamable HTTP
)
```

The transport is automatically handled by FastMCP 1.12.3+, providing:

- Request routing
- Session management
- Response type negotiation
- Error handling

## Migration Guide

### From SSE to Streamable HTTP

If you're currently using the legacy SSE transport, migrate to streamable HTTP:

#### 1. Update Server Configuration

**Before (SSE/Worker mode):**

```bash
biomcp run --mode worker
```

**After (Streamable HTTP):**

```bash
biomcp run --mode streamable_http
```

#### 2. Update Client Configuration

**MCP Inspector:**

```bash
npx @modelcontextprotocol/inspector uv run --with . biomcp run --mode streamable_http
```

**Claude Desktop Configuration:**

```json
{
  "mcpServers": {
    "biomcp": {
      "command": "docker",
      "args": [
        "run",
        "-p",
        "8000:8000",
        "biomcp:latest",
        "biomcp",
        "run",
        "--mode",
        "streamable_http"
      ]
    }
  }
}
```

#### 3. Update Cloudflare Worker

The worker now supports both GET (legacy SSE) and POST (streamable HTTP) on the `/mcp` endpoint:

```javascript
// Automatically routes based on method
.get("/mcp", async (c) => {
  // Legacy SSE transport
})
.post("/mcp", async (c) => {
  // Streamable HTTP transport
})
```

### Backward Compatibility

All legacy endpoints remain functional:

- `/sse` - Server-sent events transport
- `/health` - Health check endpoint
- `/events` - Event streaming endpoint

## Configuration Options

### Server Modes

```bash
# Local development (STDIO)
biomcp run

# Legacy SSE transport
biomcp run --mode worker

# Modern streamable HTTP
biomcp run --mode streamable_http --host 0.0.0.0 --port 8000
```

### Environment Variables

| Variable        | Description             | Default |
| --------------- | ----------------------- | ------- |
| `MCP_TRANSPORT` | Override transport mode | None    |
| `MCP_HOST`      | Server bind address     | 0.0.0.0 |
| `MCP_PORT`      | Server port             | 8000    |

## Session Management

Streamable HTTP uses session IDs to maintain state across requests:

```http
POST /mcp?session_id=abc123 HTTP/1.1
Content-Type: application/json

{
  "jsonrpc": "2.0",
  "method": "initialize",
  "params": {...}
}
```

Sessions are:

- Created automatically on first request
- Maintained in server memory
- Cleaned up after inactivity timeout
- Isolated between different clients

## Performance Considerations

### Response Mode Selection

The server automatically selects the optimal response mode:

| Operation Type    | Response Mode | Example                |
| ----------------- | ------------- | ---------------------- |
| Quick queries     | JSON          | `search(limit=10)`     |
| Large results     | SSE           | `search(limit=1000)`   |
| Real-time updates | SSE           | Thinking tool progress |

### Optimization Tips

1. **Use session IDs** for related requests to avoid re-initialization
2. **Batch operations** when possible to reduce round trips
3. **Set appropriate timeouts** for long-running operations
4. **Monitor response times** to identify bottlenecks

## Troubleshooting

### Common Issues

#### 1. Connection Refused

```
Error: connect ECONNREFUSED 127.0.0.1:8000
```

**Solution**: Ensure server is running with `--host 0.0.0.0` for Docker deployments.

#### 2. Session Not Found

```
Error: Session 'xyz' not found
```

**Solution**: Session may have expired. Omit session_id to create new session.

#### 3. Timeout on Large Results

```
Error: Request timeout after 30s
```

**Solution**: Increase client timeout or reduce result size with `limit` parameter.

### Debug Mode

Enable debug logging to troubleshoot transport issues:

```bash
LOG_LEVEL=debug biomcp run --mode streamable_http
```

## Security Considerations

### Authentication

BioMCP does not implement authentication at the transport layer. Secure your deployment using:

- **API Gateway**: AWS API Gateway, Kong, etc.
- **Reverse Proxy**: Nginx with auth modules
- **Cloud IAM**: Platform-specific access controls

### Rate Limiting

Implement rate limiting at the infrastructure layer:

```nginx
# Nginx example
limit_req_zone $binary_remote_addr zone=mcp:10m rate=10r/s;

location /mcp {
    limit_req zone=mcp burst=20;
    proxy_pass http://biomcp:8000;
}
```

### CORS Configuration

For browser-based clients, configure CORS headers:

```python
# Handled automatically by FastMCP when stateless_http=True
```

## Monitoring

### Health Checks

```bash
# Check server health
curl http://localhost:8000/health

# Response
{"status": "ok", "transport": "streamable_http"}
```

### Metrics

Monitor these key metrics:

- Request rate on `/mcp` endpoint
- Response time percentiles (p50, p95, p99)
- Session count and duration
- Error rate by error type

## Next Steps

- Review [MCP Specification](https://spec.modelcontextprotocol.io) for protocol details

For questions or issues, please visit our [GitHub repository](https://github.com/genomoncology/biomcp).

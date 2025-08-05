# Server Deployment Guide

This guide covers various deployment options for BioMCP, from local development to production cloud deployments with authentication.

## Deployment Options Overview

| Mode                  | Use Case      | Transport       | Authentication | Scalability |
| --------------------- | ------------- | --------------- | -------------- | ----------- |
| **Local STDIO**       | Development   | STDIO           | None           | Single user |
| **HTTP Server**       | Small teams   | Streamable HTTP | Optional       | Moderate    |
| **Docker**            | Containerized | Streamable HTTP | Optional       | Moderate    |
| **Cloudflare Worker** | Production    | SSE/HTTP        | OAuth optional | High        |

## Local Development (STDIO)

The simplest deployment for development and testing.

### Setup

```bash
# Install BioMCP
uv tool install biomcp

# Run in STDIO mode (default)
biomcp run
```

### Configuration

For Claude Desktop integration:

```json
{
  "mcpServers": {
    "biomcp": {
      "command": "biomcp",
      "args": ["run"]
    }
  }
}
```

### Use Cases

- Local development
- Single-user research
- Testing new features

## HTTP Server Deployment

Modern deployment using Streamable HTTP transport.

### Basic Setup

```bash
# Run HTTP server
biomcp run --mode http --host 0.0.0.0 --port 8000
```

### With Environment Variables

```bash
# Create .env file
cat > .env << EOF
BIOMCP_HOST=0.0.0.0
BIOMCP_PORT=8000
NCI_API_KEY=your-key
ALPHAGENOME_API_KEY=your-key
EOF

# Run with env file
biomcp run --mode http
```

### Systemd Service (Linux)

Create `/etc/systemd/system/biomcp.service`:

```ini
[Unit]
Description=BioMCP Server
After=network.target

[Service]
Type=simple
User=biomcp
WorkingDirectory=/opt/biomcp
Environment="PATH=/usr/local/bin:/usr/bin"
EnvironmentFile=/opt/biomcp/.env
ExecStart=/usr/local/bin/biomcp run --mode http
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl enable biomcp
sudo systemctl start biomcp
```

### Nginx Reverse Proxy

```nginx
server {
    listen 443 ssl;
    server_name biomcp.example.com;

    ssl_certificate /etc/ssl/certs/biomcp.crt;
    ssl_certificate_key /etc/ssl/private/biomcp.key;

    location /mcp {
        proxy_pass http://localhost:8000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_buffering off;
    }
}
```

## Docker Deployment

Containerized deployment for consistency and portability.

### Basic Dockerfile

```dockerfile
FROM python:3.11-slim

# Install BioMCP
RUN pip install biomcp-python

# Add API keys (use secrets in production!)
ENV NCI_API_KEY=""
ENV ALPHAGENOME_API_KEY=""

# Expose port
EXPOSE 8000

# Run server
CMD ["biomcp", "run", "--mode", "http", "--host", "0.0.0.0"]
```

### With AlphaGenome Support

```dockerfile
FROM python:3.11-slim

# Install system dependencies
RUN apt-get update && apt-get install -y git

# Install BioMCP
RUN pip install biomcp-python

# Install AlphaGenome
RUN git clone https://github.com/google-deepmind/alphagenome.git && \
    cd alphagenome && \
    pip install .

# Configure
ENV MCP_MODE=http
ENV BIOMCP_HOST=0.0.0.0
ENV BIOMCP_PORT=8000

EXPOSE 8000

CMD ["biomcp", "run"]
```

### Docker Compose

```yaml
version: "3.8"

services:
  biomcp:
    build: .
    ports:
      - "8000:8000"
    environment:
      - MCP_MODE=http
      - NCI_API_KEY=${NCI_API_KEY}
      - ALPHAGENOME_API_KEY=${ALPHAGENOME_API_KEY}
    volumes:
      - ./logs:/app/logs
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
```

### Running

```bash
# Build and run
docker-compose up -d

# View logs
docker-compose logs -f

# Scale horizontally
docker-compose up -d --scale biomcp=3
```

## Cloudflare Worker Deployment

Enterprise-grade deployment with global edge distribution.

### Prerequisites

1. Cloudflare account
2. Wrangler CLI installed
3. Remote BioMCP server running

### Architecture

```
Claude Desktop → Cloudflare Worker (Edge) → BioMCP Server (Origin)
```

### Setup Worker

1. **Install dependencies:**

```bash
npm install @modelcontextprotocol/sdk itty-router
```

2. **Create `wrangler.toml`:**

```toml
name = "biomcp-worker"
main = "src/index.js"
compatibility_date = "2024-01-01"

[vars]
REMOTE_MCP_SERVER_URL = "https://your-biomcp-server.com/mcp"
MCP_SERVER_API_KEY = "your-secret-key"

[[kv_namespaces]]
binding = "AUTH_TOKENS"
id = "your-kv-namespace-id"
```

3. **Deploy:**

```bash
wrangler deploy
```

### With OAuth Authentication (Stytch)

1. **Configure Stytch:**

```toml
[vars]
STYTCH_PROJECT_ID = "project-test-..."
STYTCH_SECRET = "secret-test-..."
STYTCH_PUBLIC_TOKEN = "public-token-test-..."
JWT_SECRET = "your-jwt-secret"
```

2. **OAuth Endpoints:**
   The worker automatically provides:

- `/.well-known/oauth-authorization-server`
- `/authorize`
- `/callback`
- `/token`

3. **Client Configuration:**

```json
{
  "mcpServers": {
    "biomcp": {
      "transport": {
        "type": "sse",
        "url": "https://your-worker.workers.dev"
      },
      "auth": {
        "type": "oauth",
        "client_id": "mcp-client",
        "authorization_endpoint": "https://your-worker.workers.dev/authorize",
        "token_endpoint": "https://your-worker.workers.dev/token",
        "scope": "mcp:access"
      }
    }
  }
}
```

## Production Considerations

### Security

1. **API Key Management:**

```bash
# Use environment variables
export NCI_API_KEY="$(vault kv get -field=key secret/biomcp/nci)"

# Or use secrets management
docker run --secret biomcp_keys biomcp:latest
```

2. **Network Security:**

- Use HTTPS everywhere
- Implement rate limiting
- Set up CORS properly
- Use authentication for public endpoints

3. **Access Control:**

```python
# Example middleware
async def auth_middleware(request, call_next):
    token = request.headers.get("Authorization")
    if not validate_token(token):
        return JSONResponse({"error": "Unauthorized"}, status_code=401)
    return await call_next(request)
```

### Monitoring

1. **Health Checks:**

```python
# Built-in health endpoint
GET /health

# Custom health check
@app.get("/health/detailed")
async def health_detailed():
    return {
        "status": "healthy",
        "version": __version__,
        "apis": check_api_status(),
        "timestamp": datetime.utcnow()
    }
```

2. **Metrics:**

```python
# Prometheus metrics
from prometheus_client import Counter, Histogram

request_count = Counter('biomcp_requests_total', 'Total requests')
request_duration = Histogram('biomcp_request_duration_seconds', 'Request duration')
```

3. **Logging:**

```python
# Structured logging
import structlog

logger = structlog.get_logger()
logger.info("request_processed",
    tool="article_searcher",
    duration=0.234,
    user_id="user123"
)
```

### Scaling

1. **Horizontal Scaling:**

```yaml
# Kubernetes deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: biomcp
spec:
  replicas: 3
  selector:
    matchLabels:
      app: biomcp
  template:
    metadata:
      labels:
        app: biomcp
    spec:
      containers:
        - name: biomcp
          image: biomcp:latest
          ports:
            - containerPort: 8000
          resources:
            requests:
              memory: "512Mi"
              cpu: "500m"
            limits:
              memory: "1Gi"
              cpu: "1000m"
```

2. **Caching:**

```python
# Redis caching
import redis
from functools import wraps

redis_client = redis.Redis()

def cache_result(ttl=3600):
    def decorator(func):
        @wraps(func)
        async def wrapper(*args, **kwargs):
            key = f"{func.__name__}:{str(args)}:{str(kwargs)}"
            cached = redis_client.get(key)
            if cached:
                return json.loads(cached)
            result = await func(*args, **kwargs)
            redis_client.setex(key, ttl, json.dumps(result))
            return result
        return wrapper
    return decorator
```

### Performance Optimization

1. **Connection Pooling:**

```python
# Reuse HTTP connections
import httpx

client = httpx.AsyncClient(
    limits=httpx.Limits(max_keepalive_connections=20),
    timeout=httpx.Timeout(30.0)
)
```

2. **Async Processing:**

```python
# Process requests concurrently
async def handle_batch(requests):
    tasks = [process_request(req) for req in requests]
    return await asyncio.gather(*tasks)
```

3. **Response Compression:**

```python
# Enable gzip compression
from fastapi.middleware.gzip import GZipMiddleware

app.add_middleware(GZipMiddleware, minimum_size=1000)
```

## Migration Path

### From STDIO to HTTP

1. Update server startup:

```bash
# Old
biomcp run

# New
biomcp run --mode http
```

2. Update client configuration:

```json
{
  "mcpServers": {
    "biomcp": {
      "url": "http://localhost:8000/mcp"
    }
  }
}
```

### From SSE to Streamable HTTP

1. Update worker code to use `/mcp` endpoint
2. Update client to use new transport:

```json
{
  "transport": {
    "type": "http",
    "url": "https://biomcp.example.com/mcp"
  }
}
```

## Troubleshooting

### Common Issues

1. **Port Already in Use:**

```bash
# Find process using port
lsof -i :8000

# Kill process
kill -9 <PID>
```

2. **API Key Errors:**

```bash
# Verify environment variables
env | grep -E "(NCI|ALPHAGENOME|CBIO)"

# Test API key
curl -H "X-API-KEY: $NCI_API_KEY" https://api.cancer.gov/v2/trials
```

3. **Connection Timeouts:**

- Increase timeout values
- Check firewall rules
- Verify network connectivity

### Debug Mode

```bash
# Enable debug logging
BIOMCP_LOG_LEVEL=DEBUG biomcp run --mode http

# Or in Docker
docker run -e BIOMCP_LOG_LEVEL=DEBUG biomcp:latest
```

## Next Steps

- Set up [monitoring](../how-to-guides/05-logging-and-monitoring-with-bigquery.md)
- Configure [authentication](../getting-started/03-authentication-and-api-keys.md)
- Review [security policies](../policies.md)
- Implement [CI/CD pipeline](02-contributing-and-testing.md)

# Deploying BioMCP as a Cloudflare Worker

This guide explains how to deploy BioMCP as a Cloudflare Worker using Server-Sent Events (SSE) for communication.

## Overview

BioMCP now supports two deployment modes:

1. **Local STDIO Mode**: The traditional mode where the server communicates via standard input/output.
2. **Cloudflare Worker Mode**: Deployment as a Cloudflare Worker using SSE for communication.

## Prerequisites

- A Cloudflare account with Workers enabled
- [Wrangler CLI](https://developers.cloudflare.com/workers/wrangler/install-and-update/) installed
- For local development of the Worker mode: `pip install biomcp-python[worker]`

## Architecture

When deployed as a Cloudflare Worker, BioMCP works as follows:

1. The Cloudflare Worker receives HTTP requests from clients
2. The Worker forwards these requests to your remote MCP server
3. The remote MCP server processes the requests and returns responses
4. The Worker streams these responses back to clients using SSE

### Architecture Diagram

Below is an improved diagram of the setup:

```
+-----------------------+
|  Claude Desktop (or   |
|    other client)      |
+----------+------------+
           |
           v
+----------+------------+
|   Cloudflare Worker   |
+----------+------------+
           |
           v
+-------------------------------+
|   FastMCP Python Service      |
|   (Docker, hosted server)     |
+-------------------------------+
```

## Setup

### 1. Configure Your Remote MCP Server

First, you need to set up a remote MCP server that will handle the actual processing:

#### Using Docker Compose

A Docker Compose file is now provided for building and deploying the remote FastMCP Python service. You must set the `TAG` variable to specify the image version:

```bash
TAG=latest docker compose up -d
```

- The service will be accessible on the configured port (default: 8000).
- Ensure your server is reachable from Cloudflare Workers.

#### Manual Installation

```bash
# Install with worker dependencies
pip install biomcp-python[worker]

# Run the server in worker mode
biomcp run --mode worker --host 0.0.0.0 --port 8000
```

Make sure this server is accessible from the internet, or at least from Cloudflare Workers.

### 2. Configure Cloudflare Worker

Edit the `wrangler.toml` file to point to your remote MCP server:

```toml
[vars]
REMOTE_MCP_SERVER_URL = "https://your-remote-mcp-server.com/mcp"
# Add an API key if your server requires authentication
MCP_SERVER_API_KEY = "your-api-key"
```

### 3. Deploy the Worker

Use Wrangler to deploy your Worker:

```bash
# Login to Cloudflare
npx wrangler@latest login

# Deploy the worker
npx wrangler@latest deploy

# Tail logs for debugging
npx wrangler@latest tail
```

## Benefits of Remote MCP

- **Scalability:** Offloads heavy computation to a dedicated server, reducing load on the Worker and improving performance.
- **Security:** The Worker acts as a secure proxy, hiding your backend and enabling API key protection.
- **Flexibility:** You can update or scale the Python service independently of the Worker.
- **Debugging:** Use `npx wrangler tail` for real-time logs and easier troubleshooting.
- **Modern Deployment:** Docker Compose simplifies environment setup and reproducibility.

## Usage

Once deployed, your Cloudflare Worker will be available at a URL like:
`https://biomcp-worker.<your-worker-subdomain>.workers.dev`

Clients can connect to this endpoint using SSE:

```javascript
// Example client-side JavaScript
const eventSource = new EventSource(
  "https://biomcp-worker.<your-worker-subdomain>.workers.dev",
);

eventSource.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log("Received:", data);

  // Check for the end of the stream
  if (event.data === "[DONE]") {
    eventSource.close();
  }
};

eventSource.onerror = (error) => {
  console.error("EventSource error:", error);
  eventSource.close();
};
```

## Local Development

For local development and testing, you can run the worker mode locally:

```bash
# Run the server in worker mode on localhost
biomcp run --mode worker --host 127.0.0.1 --port 8000
```

Then use Wrangler to develop locally:

```bash
npx wrangler@latest dev
```

## Troubleshooting

### Worker Connection Issues

If the Worker cannot connect to your remote MCP server:

1. Ensure your remote server is publicly accessible
2. Check that the `REMOTE_MCP_SERVER_URL` is correctly set
3. Verify any authentication requirements

### Performance Considerations

- Cloudflare Workers have execution time limits (typically 30 seconds for free accounts)
- Consider implementing timeouts and chunking for large responses
- Monitor your Worker's performance in the Cloudflare dashboard

## Security Considerations

- Always use HTTPS for communication between the Worker and your remote MCP server
- Consider implementing authentication for your remote MCP server
- Do not expose sensitive information in your Worker code

/**
 * BioMCP Worker – Auth‑less version (rev 1.8)
 *
 *  Fix: Added improved error handling and increased timeouts for list requests
 */

// Server URL will be configured from environment variables
let REMOTE_MCP_SERVER_URL = "http://localhost:8000"; // Default fallback
const DEBUG = true;

const log = (m) => DEBUG && console.log("[DEBUG]", m);
const CORS = {
  "Access-Control-Allow-Origin": "*",
  "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
  "Access-Control-Allow-Headers": "*",
  "Access-Control-Max-Age": "86400",
};
const json = (o, s = 200) =>
  new Response(JSON.stringify(o, null, 2), {
    status: s,
    headers: { "Content-Type": "application/json", ...CORS },
  });

let forwardPath = "/messages"; // for proxying JSON‑RPC POSTS (no query)
let resourceEndpoint = null; // full string we echo back (/messages/?sid=…)

// Track active SSE connections to avoid duplicate connections
const activeConnections = new Map();

export default {
  async fetch(req, env, ctx) {
    // Use environment variable if available, otherwise use the default
    REMOTE_MCP_SERVER_URL = env.REMOTE_MCP_SERVER_URL || REMOTE_MCP_SERVER_URL;

    const url = new URL(req.url);
    log(`${req.method} ${url.pathname}${url.search}`);

    if (req.method === "OPTIONS")
      return new Response(null, { status: 204, headers: CORS });
    if (url.pathname === "/status" || url.pathname === "/debug")
      return json({
        worker: "BioMCP-authless",
        remote: REMOTE_MCP_SERVER_URL,
        forwardPath,
        resourceEndpoint,
      });
    if (url.pathname === "/sse" || url.pathname === "/events")
      return serveSSE(req, ctx);

    if (req.method === "POST") {
      const sid = url.searchParams.get("session_id");
      if (!sid) return new Response("Missing session_id", { status: 400 });
      return proxyPost(req, forwardPath, sid);
    }

    return new Response("Not found", { status: 404 });
  },
};

async function proxyPost(req, path, sid) {
  const body = await req.text();
  const target = `${REMOTE_MCP_SERVER_URL}${path}?session_id=${encodeURIComponent(
    sid,
  )}`;

  try {
    // Parse the request to check if it's a list request that might need a longer timeout
    let jsonBody;
    try {
      jsonBody = JSON.parse(body);
    } catch (e) {
      // Not valid JSON, proceed with normal request
      jsonBody = {};
    }

    // Set a longer timeout for list requests that tend to time out
    const timeout =
      jsonBody.method &&
      (jsonBody.method === "tools/list" || jsonBody.method === "resources/list")
        ? 30000
        : 10000;

    // Use AbortController to implement timeout
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), timeout);

    log(`Proxying ${jsonBody.method || "request"} with timeout ${timeout}ms`);

    const resp = await fetch(target, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body,
      signal: controller.signal,
    });

    clearTimeout(timeoutId);

    // If it's a list request, cache the response for future use
    if (
      jsonBody.method &&
      (jsonBody.method === "tools/list" || jsonBody.method === "resources/list")
    ) {
      log(`Received response for ${jsonBody.method}`);
    }

    return new Response(await resp.text(), {
      status: resp.status,
      headers: { "Content-Type": "application/json", ...CORS },
    });
  } catch (error) {
    log(`POST error: ${error.message}`);

    // For timeout errors, provide a default empty response for list requests
    if (error.name === "AbortError") {
      try {
        const jsonBody = JSON.parse(body);
        if (jsonBody.method === "tools/list") {
          log("Returning empty tools list due to timeout");
          return new Response(
            JSON.stringify({
              jsonrpc: "2.0",
              id: jsonBody.id,
              result: { tools: [] },
            }),
            {
              status: 200,
              headers: { "Content-Type": "application/json", ...CORS },
            },
          );
        } else if (jsonBody.method === "resources/list") {
          log("Returning empty resources list due to timeout");
          return new Response(
            JSON.stringify({
              jsonrpc: "2.0",
              id: jsonBody.id,
              result: { resources: [] },
            }),
            {
              status: 200,
              headers: { "Content-Type": "application/json", ...CORS },
            },
          );
        }
      } catch (e) {
        // If parsing fails, fall through to default error response
      }
    }

    return new Response(JSON.stringify({ error: error.message }), {
      status: 502,
      headers: { "Content-Type": "application/json", ...CORS },
    });
  }
}

function serveSSE(clientReq, ctx) {
  const enc = new TextEncoder();
  let keepalive;
  const upstreamCtl = new AbortController();

  const stream = new ReadableStream({
    async start(ctrl) {
      ctrl.enqueue(enc.encode("event: ready\ndata: {}\n\n"));

      clientReq.signal.addEventListener("abort", () => {
        clearInterval(keepalive);
        upstreamCtl.abort();
        ctrl.close();
      });

      try {
        const u = await fetch(`${REMOTE_MCP_SERVER_URL}/sse`, {
          headers: { Accept: "text/event-stream" },
          signal: upstreamCtl.signal,
        });

        if (!u.ok || !u.body) throw new Error(`Upstream SSE ${u.status}`);
        const r = u.body.getReader();

        while (true) {
          const { value, done } = await r.read();
          if (done) break;
          if (value) {
            const text = new TextDecoder().decode(value);
            // capture first endpoint once
            if (!resourceEndpoint) {
              const m = text.match(
                /data:\s*(\/messages\/\?session_id=[A-Za-z0-9_-]+)/,
              );
              if (m) {
                resourceEndpoint = m[1];
                forwardPath = resourceEndpoint.split("?")[0];
                log(`Captured endpoint ${resourceEndpoint}`);
                ctrl.enqueue(
                  enc.encode(`event: resource\ndata: ${resourceEndpoint}\n\n`),
                );
              }
            }
            ctrl.enqueue(value);
          }
        }
      } catch (e) {
        if (e.name !== "AbortError") {
          log(`SSE error: ${e.message}`);
          ctrl.enqueue(enc.encode(`event: error\ndata: ${e.message}\n\n`));
        }
      }

      // Reduce keepalive interval to 5 seconds to prevent timeouts
      keepalive = setInterval(() => {
        try {
          ctrl.enqueue(enc.encode(":keepalive\n\n"));
        } catch (_) {
          clearInterval(keepalive);
        }
      }, 5000);
    },
  });

  return new Response(stream, {
    headers: {
      "Content-Type": "text/event-stream",
      "Cache-Control": "no-cache",
      Connection: "keep-alive",
      ...CORS,
    },
  });
}

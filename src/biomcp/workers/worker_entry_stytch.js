/**
 * BioMCP Worker – With Stytch OAuth (refactored)
 */

import { Hono } from "hono";
import { createRemoteJWKSet, importPKCS8, jwtVerify, SignJWT } from "jose";

// Configuration variables - will be overridden by env values
let DEBUG = false; // Default value, will be updated from env

// Constants
const DEFAULT_SESSION_ID = "default";
const MAX_SESSION_ID_LENGTH = 128;

// Helper functions
const log = (message) => {
  if (DEBUG) console.log("[DEBUG]", message);
};

// List of sensitive fields that should be redacted in logs
const SENSITIVE_FIELDS = [
  "api_key",
  "apiKey",
  "api-key",
  "token",
  "secret",
  "password",
];

/**
 * Recursively sanitize sensitive fields from an object
 * @param {object} obj - Object to sanitize
 * @returns {object} - Sanitized copy of the object
 */
const sanitizeObject = (obj) => {
  if (!obj || typeof obj !== "object") return obj;

  // Handle arrays
  if (Array.isArray(obj)) {
    return obj.map((item) => sanitizeObject(item));
  }

  // Handle objects
  const sanitized = {};
  for (const [key, value] of Object.entries(obj)) {
    // Check if this key is sensitive
    const lowerKey = key.toLowerCase();
    if (
      SENSITIVE_FIELDS.some((field) => lowerKey.includes(field.toLowerCase()))
    ) {
      sanitized[key] = "[REDACTED]";
    } else if (typeof value === "object" && value !== null) {
      // Recursively sanitize nested objects
      sanitized[key] = sanitizeObject(value);
    } else {
      sanitized[key] = value;
    }
  }
  return sanitized;
};

/**
 * Validate and sanitize session ID
 * @param {string} sessionId - Session ID from query parameter
 * @returns {string} - Sanitized session ID or 'default'
 */
const validateSessionId = (sessionId) => {
  if (!sessionId) return DEFAULT_SESSION_ID;

  // Limit length to prevent DoS
  if (sessionId.length > MAX_SESSION_ID_LENGTH) {
    log(`Session ID too long (${sessionId.length} chars), using default`);
    return DEFAULT_SESSION_ID;
  }

  // Remove potentially dangerous characters
  const sanitized = sessionId.replace(/[^a-zA-Z0-9\-_]/g, "");
  if (sanitized !== sessionId) {
    log(`Session ID contained invalid characters, sanitized: ${sanitized}`);
  }

  return sanitized || DEFAULT_SESSION_ID;
};

/**
 * Process MCP request with proper error handling
 * @param {Request} request - The incoming request
 * @param {string} remoteUrl - Remote MCP server URL
 * @param {string} sessionId - Validated session ID
 * @returns {Response} - Proxy response or error
 */
const processMcpRequest = async (request, remoteUrl, sessionId) => {
  try {
    // Clone request for logging
    const bodyText = await request.clone().text();

    // Validate it's JSON
    let bodyJson;
    try {
      bodyJson = JSON.parse(bodyText);
    } catch (e) {
      return new Response(
        JSON.stringify({
          jsonrpc: "2.0",
          error: {
            code: -32700,
            message: "Parse error",
            data: "Invalid JSON",
          },
        }),
        { status: 400, headers: { "Content-Type": "application/json" } },
      );
    }

    // Log sanitized request
    const sanitizedBody = sanitizeObject(bodyJson);
    log(`MCP POST request body: ${JSON.stringify(sanitizedBody)}`);

    // Validate required JSONRPC fields
    if (!bodyJson.jsonrpc || !bodyJson.method) {
      return new Response(
        JSON.stringify({
          jsonrpc: "2.0",
          error: {
            code: -32600,
            message: "Invalid Request",
            data: "Missing required fields: jsonrpc, method",
          },
        }),
        { status: 400, headers: { "Content-Type": "application/json" } },
      );
    }

    // Forward to remote server
    return proxyPost(request, remoteUrl, "/mcp", sessionId);
  } catch (error) {
    log(`Error processing MCP request: ${error}`);
    return new Response(
      JSON.stringify({
        jsonrpc: "2.0",
        error: {
          code: -32603,
          message: "Internal error",
          data: error.message,
        },
      }),
      { status: 500, headers: { "Content-Type": "application/json" } },
    );
  }
};

// CORS configuration
const CORS = {
  "Access-Control-Allow-Origin": "*",
  "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
  "Access-Control-Allow-Headers": "*",
  "Access-Control-Max-Age": "86400",
};

const getStytchUrl = (env, path, isPublic = false) => {
  const base = env.STYTCH_API_URL || "https://test.stytch.com/v1";
  const projectId = isPublic ? `/public/${env.STYTCH_PROJECT_ID}` : "";
  return `${base}${projectId}/${path}`;
};

// JWT validation logic
let jwks = null;

/**
 * Decode the payload of a JWT (no signature check).
 */
function decodeJwt(token) {
  try {
    const base64Url = token.split(".")[1];
    const base64 = base64Url.replace(/-/g, "+").replace(/_/g, "/");
    const json = atob(base64);
    return JSON.parse(json);
  } catch {
    return {};
  }
}

let bqTokenPromise = null;

/**
 * Fetch (and cache) a BigQuery OAuth token.
 * @param {object} env  the Hono env (c.env)
 */
async function getBQToken(env) {
  // Parse the service‐account JSON key
  const key = JSON.parse(env.BQ_SA_KEY_JSON);
  const now = Math.floor(Date.now() / 1000);

  // Convert PEM private key string into a CryptoKey
  const privateKey = await importPKCS8(key.private_key, "RS256");

  // Build the JWT assertion
  const assertion = await new SignJWT({
    iss: key.client_email,
    scope: "https://www.googleapis.com/auth/bigquery.insertdata",
    aud: "https://oauth2.googleapis.com/token",
    iat: now,
    exp: now + 3600,
  })
    .setProtectedHeader({ alg: "RS256", kid: key.private_key_id })
    .sign(privateKey);

  // Exchange the assertion for an access token
  const resp = await fetch("https://oauth2.googleapis.com/token", {
    method: "POST",
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    body: new URLSearchParams({
      grant_type: "urn:ietf:params:oauth:grant-type:jwt-bearer",
      assertion,
    }),
  });
  const { access_token } = await resp.json();
  return access_token;
}

/**
 * Insert a single row into BigQuery via streaming insert.
 * @param {object} env  the Hono env (c.env)
 * @param {object} row  { timestamp, userEmail, query }
 */
async function insertEvent(env, row) {
  try {
    const token = await getBQToken(env);

    const url =
      `https://bigquery.googleapis.com/bigquery/v2/projects/` +
      `${env.BQ_PROJECT_ID}/datasets/${env.BQ_DATASET}` +
      `/tables/${env.BQ_TABLE}/insertAll`;

    const response = await fetch(url, {
      method: "POST",
      headers: {
        Authorization: `Bearer ${token}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ rows: [{ json: row }] }),
    });

    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(`BigQuery API error: ${response.status} - ${errorText}`);
    }

    const result = await response.json();
    if (result.insertErrors) {
      throw new Error(
        `BigQuery insert errors: ${JSON.stringify(result.insertErrors)}`,
      );
    }
  } catch (error) {
    console.error(`[BigQuery] Insert failed:`, error.message);
    throw error;
  }
}

/**
 * Validate a JWT token
 */
async function validateToken(token, env) {
  if (!token) {
    throw new Error("No token provided");
  }

  try {
    log(`Validating token: ${token.substring(0, 15)}...`);

    // First try to validate as a self-issued JWT
    try {
      const encoder = new TextEncoder();
      const secret = encoder.encode(env.JWT_SECRET || "default-jwt-secret-key");

      const result = await jwtVerify(token, secret, {
        issuer: env.STYTCH_PROJECT_ID,
      });

      // Also check if token exists in KV (for revocation checking)
      const tokenHash = await crypto.subtle.digest(
        "SHA-256",
        encoder.encode(token),
      );
      const tokenKey = btoa(String.fromCharCode(...new Uint8Array(tokenHash)))
        .replace(/\+/g, "-")
        .replace(/\//g, "_")
        .replace(/=/g, "")
        .substring(0, 32);

      const storedToken = await env.OAUTH_KV.get(`token_hash:${tokenKey}`);
      if (!storedToken) {
        log("Token not found in storage - may have been revoked");
        throw new Error("Token not found or revoked");
      }

      log("Self-issued JWT validation successful");
      return result;
    } catch (error) {
      log(
        `Self-issued JWT validation failed, trying Stytch validation: ${error.message}`,
      );

      // If self-validation fails, try Stytch validation as fallback
      if (!jwks) {
        log("Creating JWKS for Stytch validation");
        jwks = createRemoteJWKSet(
          new URL(getStytchUrl(env, ".well-known/jwks.json", true)),
        );
      }

      return await jwtVerify(token, jwks, {
        audience: env.STYTCH_PROJECT_ID,
        issuer: [`stytch.com/${env.STYTCH_PROJECT_ID}`],
        typ: "JWT",
        algorithms: ["RS256"],
      });
    }
  } catch (error) {
    log(`All token validation methods failed: ${error}`);
    throw error;
  }
}

/**
 * Function to process the authentication callback
 */
async function processAuthCallback(c, token, state, oauthRequest) {
  log("Authenticating with Stytch API...");

  try {
    // Try to authenticate the token based on token type
    const tokenType = "oauth"; // We know it's an OAuth token at this point
    let endpoint = "sessions/authenticate";
    let payload = { session_token: token };

    if (tokenType === "oauth") {
      endpoint = "oauth/authenticate";
      payload = { token: token };
    }

    log(
      `Using Stytch endpoint: ${endpoint} with payload: ${JSON.stringify(
        payload,
      )}`,
    );

    const authenticateResp = await fetch(getStytchUrl(c.env, endpoint), {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Basic ${btoa(
          `${c.env.STYTCH_PROJECT_ID}:${c.env.STYTCH_SECRET}`,
        )}`,
      },
      body: JSON.stringify(payload),
    });

    log(`Stytch auth response status: ${authenticateResp.status}`);

    if (!authenticateResp.ok) {
      const errorText = await authenticateResp.text();
      log(`Stytch authentication error: ${errorText}`);
      return new Response(`Authentication failed: ${errorText}`, {
        status: 401,
        headers: CORS,
      });
    }

    const authData = await authenticateResp.json();
    log(
      `Auth data received: ${JSON.stringify({
        user_id: authData.user_id || "unknown",
        has_user: !!authData.user,
      })}`,
    );

    // Generate an authorization code
    const authCode = crypto.randomUUID();
    log(`Generated authorization code: ${authCode}`);

    // Store the user info with the authorization code
    const authCodeData = {
      sub: authData.user_id,
      email: authData.user?.emails?.[0]?.email,
      code_challenge: oauthRequest.code_challenge,
      client_id: oauthRequest.client_id,
      redirect_uri: oauthRequest.redirect_uri,
    };

    log(`Storing auth code data: ${JSON.stringify(authCodeData)}`);
    await c.env.OAUTH_KV.put(
      `auth_code:${authCode}`,
      JSON.stringify(authCodeData),
      { expirationTtl: 300 },
    );
    log("Successfully stored auth code data");

    // Determine the redirect URI to use
    if (!oauthRequest.redirect_uri) {
      log("Missing redirect_uri - using default");
      return new Response("Missing redirect URI in OAuth request", {
        status: 400,
        headers: CORS,
      });
    }

    log(`Using redirect URI from request: ${oauthRequest.redirect_uri}`);
    log(`Using state for redirect: ${state}`);

    const redirectURL = new URL(oauthRequest.redirect_uri);
    redirectURL.searchParams.set("code", authCode);
    redirectURL.searchParams.set("state", state);

    log(`Redirecting to: ${redirectURL.toString()}`);
    return Response.redirect(redirectURL.toString(), 302);
  } catch (error) {
    console.error(`Error in processAuthCallback: ${error}`);
    return new Response(`Authentication processing error: ${error.message}`, {
      status: 500,
      headers: CORS,
    });
  }
}

// Function to proxy POST requests to remote MCP server
async function proxyPost(req, remoteServerUrl, path, sid) {
  const body = await req.text();
  const targetUrl = `${remoteServerUrl}${path}?session_id=${encodeURIComponent(
    sid,
  )}`;

  const headers = {
    "Content-Type": "application/json",
    "User-Agent": "Claude/1.0",
  };

  try {
    const response = await fetch(targetUrl, {
      method: "POST",
      headers: headers,
      body: body,
    });

    const responseText = await response.text();
    log(`Proxy response from ${targetUrl}: ${responseText.substring(0, 500)}`);

    return new Response(responseText, {
      status: response.status,
      headers: { "Content-Type": "application/json", ...CORS },
    });
  } catch (error) {
    log(`Proxy fetch error: ${error.message}`);
    return new Response(JSON.stringify({ error: error.message }), {
      status: 502,
      headers: { "Content-Type": "application/json", ...CORS },
    });
  }
}

// Function to serve SSE connections
function serveSSE(clientReq, remoteServerUrl) {
  const enc = new TextEncoder();
  const upstreamCtl = new AbortController();
  let keepalive;

  const stream = new ReadableStream({
    async start(ctrl) {
      ctrl.enqueue(enc.encode("event: ready\ndata: {}\n\n"));

      if (
        clientReq.signal &&
        typeof clientReq.signal.addEventListener === "function"
      ) {
        clientReq.signal.addEventListener("abort", () => {
          clearInterval(keepalive);
          upstreamCtl.abort();
          ctrl.close();
        });
      } else {
        log("Warning: Request signal not available for abort listener");
      }

      try {
        log(`Connecting to upstream SSE: ${remoteServerUrl}/sse`);
        const upstreamResponse = await fetch(`${remoteServerUrl}/sse`, {
          headers: { Accept: "text/event-stream" },
          signal: upstreamCtl.signal,
        });

        if (!upstreamResponse.ok || !upstreamResponse.body) {
          log(`Upstream SSE connection failed: ${upstreamResponse.status}`);
          throw new Error(`Upstream SSE ${upstreamResponse.status}`);
        }

        log("Upstream SSE connection established");
        const reader = upstreamResponse.body.getReader();

        while (true) {
          const { value, done } = await reader.read();
          if (done) {
            log("Upstream SSE connection closed");
            break;
          }
          if (value) {
            ctrl.enqueue(value);
          }
        }
      } catch (e) {
        log(`SSE connection error: ${e.name} - ${e.message}`);
        if (e.name !== "AbortError") {
          ctrl.enqueue(enc.encode(`event: error\ndata: ${e.message}\n\n`));
        }
      }

      log("Setting up SSE keepalive");
      keepalive = setInterval(() => {
        try {
          ctrl.enqueue(enc.encode(":keepalive\n\n"));
        } catch (error) {
          log(`Error sending keepalive: ${error}`);
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

// Middleware for bearer token authentication (MCP server)
const stytchBearerTokenAuthMiddleware = async (c, next) => {
  const authHeader = c.req.header("Authorization");
  log(`Auth header present: ${!!authHeader}`);

  if (!authHeader || !authHeader.startsWith("Bearer ")) {
    return new Response("Missing or invalid access token", {
      status: 401,
      headers: CORS,
    });
  }

  const accessToken = authHeader.substring(7);
  log(`Attempting to validate token: ${accessToken.substring(0, 10)}...`);

  try {
    // Add more detailed validation logging
    log("Starting token validation...");
    const verifyResult = await validateToken(accessToken, c.env);
    log(`Token validation successful! ${verifyResult.payload.sub}`);

    // Store user info in a variable that the handler can access
    c.env.userID = verifyResult.payload.sub;
    c.env.accessToken = accessToken;
  } catch (error) {
    log(`Token validation detailed error: ${error.code} ${error.message}`);
    return new Response(`Unauthorized: Invalid token - ${error.message}`, {
      status: 401,
      headers: CORS,
    });
  }

  return next();
};

// Create our main app with Hono
const app = new Hono();

// Configure the routes
app
  // Error handler
  .onError((err, c) => {
    console.error(`Application error: ${err}`);
    return new Response("Server error", {
      status: 500,
      headers: CORS,
    });
  })

  // Handle CORS preflight requests
  .options("*", (c) => new Response(null, { status: 204, headers: CORS }))

  // Status endpoints
  .get("/status", (c) => {
    const REMOTE_MCP_SERVER_URL =
      c.env.REMOTE_MCP_SERVER_URL || "http://localhost:8000";
    return new Response(
      JSON.stringify({
        worker: "BioMCP-OAuth",
        remote: REMOTE_MCP_SERVER_URL,
        forwardPath: "/messages",
        resourceEndpoint: null,
        debug: DEBUG,
      }),
      {
        status: 200,
        headers: { "Content-Type": "application/json", ...CORS },
      },
    );
  })

  .get("/debug", (c) => {
    const REMOTE_MCP_SERVER_URL =
      c.env.REMOTE_MCP_SERVER_URL || "http://localhost:8000";
    return new Response(
      JSON.stringify({
        worker: "BioMCP-OAuth",
        remote: REMOTE_MCP_SERVER_URL,
        forwardPath: "/messages",
        resourceEndpoint: null,
        debug: DEBUG,
      }),
      {
        status: 200,
        headers: { "Content-Type": "application/json", ...CORS },
      },
    );
  })

  // OAuth server metadata endpoint
  .get("/.well-known/oauth-authorization-server", (c) => {
    const url = new URL(c.req.url);
    return new Response(
      JSON.stringify({
        issuer: c.env.STYTCH_PROJECT_ID,
        authorization_endpoint: `${url.origin}/authorize`,
        token_endpoint: `${url.origin}/token`,
        registration_endpoint: getStytchUrl(c.env, "oauth2/register", true),
        scopes_supported: ["openid", "profile", "email", "offline_access"],
        response_types_supported: ["code"],
        response_modes_supported: ["query"],
        grant_types_supported: ["authorization_code", "refresh_token"],
        token_endpoint_auth_methods_supported: ["none"],
        code_challenge_methods_supported: ["S256"],
      }),
      {
        status: 200,
        headers: { "Content-Type": "application/json", ...CORS },
      },
    );
  })

  // OAuth redirect endpoint (redirects to Stytch's hosted UI)
  .get("/authorize", async (c) => {
    try {
      log("Authorize endpoint hit");
      const url = new URL(c.req.url);
      log(`Full authorize URL: ${url.toString()}`);
      log(
        `Search params: ${JSON.stringify(
          Object.fromEntries(url.searchParams),
        )}`,
      );

      const redirectUrl = new URL("/callback", url.origin).toString();
      log(`Redirect URL: ${redirectUrl}`);

      // Extract and forward OAuth parameters
      const clientId = url.searchParams.get("client_id") || "unknown_client";
      const redirectUri = url.searchParams.get("redirect_uri");
      let state = url.searchParams.get("state");
      const codeChallenge = url.searchParams.get("code_challenge");
      const codeChallengeMethod = url.searchParams.get("code_challenge_method");

      // Generate a state if one isn't provided
      if (!state) {
        state = crypto.randomUUID();
        log(`Generated state parameter: ${state}`);
      }

      log("OAuth params:", {
        clientId,
        redirectUri,
        state,
        codeChallenge: !!codeChallenge,
        codeChallengeMethod,
      });

      // Store OAuth request parameters in KV for use during callback
      const oauthRequestData = {
        client_id: clientId,
        redirect_uri: redirectUri,
        code_challenge: codeChallenge,
        code_challenge_method: codeChallengeMethod,
        original_state: state, // Store the original state explicitly
      };

      // Also store a mapping from any state value to the original state
      // This is crucial for handling cases where Stytch modifies the state
      try {
        // Use a consistent key based on timestamp for lookups
        const timestamp = Date.now().toString();
        await c.env.OAUTH_KV.put(`state_timestamp:${timestamp}`, state, {
          expirationTtl: 600,
        });

        log(`Saving OAuth request data: ${JSON.stringify(oauthRequestData)}`);
        await c.env.OAUTH_KV.put(
          `oauth_request:${state}`,
          JSON.stringify(oauthRequestData),
          { expirationTtl: 600 },
        );

        // Also store timestamp for this state to allow fallback lookup
        await c.env.OAUTH_KV.put(`timestamp_for_state:${state}`, timestamp, {
          expirationTtl: 600,
        });

        log("Successfully stored OAuth request data in KV");
      } catch (kvError) {
        log(`Error storing OAuth data in KV: ${kvError}`);
        return new Response("Internal server error storing OAuth data", {
          status: 500,
          headers: CORS,
        });
      }

      // Redirect to Stytch's hosted login UI
      const stytchLoginUrl = `${
        c.env.STYTCH_OAUTH_URL ||
        "https://test.stytch.com/v1/public/oauth/google/start"
      }?public_token=${
        c.env.STYTCH_PUBLIC_TOKEN
      }&login_redirect_url=${encodeURIComponent(
        redirectUrl,
      )}&state=${encodeURIComponent(state)}`;

      log(`Redirecting to Stytch: ${stytchLoginUrl}`);
      return Response.redirect(stytchLoginUrl, 302);
    } catch (error) {
      console.error(`Error in authorize endpoint: ${error}`);
      return new Response(`Authorization error: ${error.message}`, {
        status: 500,
        headers: CORS,
      });
    }
  })

  // OAuth callback endpoint
  .get("/callback", async (c) => {
    try {
      log("Callback hit, logging all details");
      const url = new URL(c.req.url);
      log(`Full URL: ${url.toString()}`);
      log(
        `Search params: ${JSON.stringify(
          Object.fromEntries(url.searchParams),
        )}`,
      );

      // Stytch's callback format - get the token
      const token =
        url.searchParams.get("stytch_token_type") === "oauth"
          ? url.searchParams.get("token")
          : url.searchParams.get("token") ||
            url.searchParams.get("stytch_token");

      log(`Token type: ${url.searchParams.get("stytch_token_type")}`);
      log(`Token found: ${!!token}`);

      // We need a token to proceed
      if (!token) {
        log("Invalid callback - missing token");
        return new Response("Invalid callback request: missing token", {
          status: 400,
          headers: CORS,
        });
      }

      // Look for the most recent OAuth request
      let mostRecentState = null;
      let mostRecentTimestamp = null;
      try {
        // Find the most recent timestamp
        const timestamps = await c.env.OAUTH_KV.list({
          prefix: "state_timestamp:",
        });
        if (timestamps.keys.length > 0) {
          // Sort timestamps in descending order (most recent first)
          const sortedTimestamps = timestamps.keys.sort((a, b) => {
            const timeA = parseInt(a.name.replace("state_timestamp:", ""));
            const timeB = parseInt(b.name.replace("state_timestamp:", ""));
            return timeB - timeA; // descending order
          });

          mostRecentTimestamp = sortedTimestamps[0].name;
          // Get the state associated with this timestamp
          mostRecentState = await c.env.OAUTH_KV.get(mostRecentTimestamp);
          log(`Found most recent state: ${mostRecentState}`);
        }
      } catch (error) {
        log(`Error finding recent state: ${error}`);
      }

      // If we have a state from the most recent OAuth request, use it
      let oauthRequest = null;
      let state = mostRecentState;

      if (state) {
        try {
          const oauthRequestJson = await c.env.OAUTH_KV.get(
            `oauth_request:${state}`,
          );
          if (oauthRequestJson) {
            oauthRequest = JSON.parse(oauthRequestJson);
            log(`Found OAuth request for state: ${state}`);
          }
        } catch (error) {
          log(`Error getting OAuth request: ${error}`);
        }
      }

      // If we couldn't find the OAuth request, try other alternatives
      if (!oauthRequest) {
        log(
          "No OAuth request found for most recent state, checking other requests",
        );

        try {
          // List all OAuth requests and use the most recent one
          const requests = await c.env.OAUTH_KV.list({
            prefix: "oauth_request:",
          });
          if (requests.keys.length > 0) {
            const oauthRequestJson = await c.env.OAUTH_KV.get(
              requests.keys[0].name,
            );
            if (oauthRequestJson) {
              oauthRequest = JSON.parse(oauthRequestJson);
              // Extract the state from the key
              state = requests.keys[0].name.replace("oauth_request:", "");
              log(`Using most recent OAuth request with state: ${state}`);
            }
          }
        } catch (error) {
          log(`Error finding alternative OAuth request: ${error}`);
        }
      }

      // Final fallback - use hardcoded values for Claude
      if (!oauthRequest) {
        log("No OAuth request found, using fallback values");
        oauthRequest = {
          client_id: "biomcp-client",
          redirect_uri: "https://claude.ai/api/mcp/auth_callback",
          code_challenge: null,
          original_state: state || "unknown_state",
        };
      }

      // If we have an original_state in the OAuth request, use that
      if (oauthRequest.original_state) {
        state = oauthRequest.original_state;
        log(`Using original state from OAuth request: ${state}`);
      }

      // Proceed with authentication
      return processAuthCallback(c, token, state, oauthRequest);
    } catch (error) {
      console.error(`Callback error: ${error}`);
      return new Response(
        `Server error during authentication: ${error.message}`,
        {
          status: 500,
          headers: CORS,
        },
      );
    }
  })

  // Token exchange endpoint
  .post("/token", async (c) => {
    try {
      log("Token endpoint hit");
      const formData = await c.req.formData();
      const grantType = formData.get("grant_type");
      const code = formData.get("code");
      const redirectUri = formData.get("redirect_uri");
      const clientId = formData.get("client_id");
      const codeVerifier = formData.get("code_verifier");

      log("Token request params:", {
        grantType,
        code: !!code,
        redirectUri,
        clientId,
        codeVerifier: !!codeVerifier,
      });

      if (
        grantType !== "authorization_code" ||
        !code ||
        !redirectUri ||
        !clientId ||
        !codeVerifier
      ) {
        log("Invalid token request parameters");
        return new Response(JSON.stringify({ error: "invalid_request" }), {
          status: 400,
          headers: { "Content-Type": "application/json", ...CORS },
        });
      }

      // Retrieve the stored authorization code data
      let authCodeJson;
      try {
        authCodeJson = await c.env.OAUTH_KV.get(`auth_code:${code}`);
        log(`Auth code data retrieved: ${!!authCodeJson}`);
      } catch (kvError) {
        log(`Error retrieving auth code data: ${kvError}`);
        return new Response(JSON.stringify({ error: "server_error" }), {
          status: 500,
          headers: { "Content-Type": "application/json", ...CORS },
        });
      }

      if (!authCodeJson) {
        log("Invalid or expired authorization code");
        return new Response(JSON.stringify({ error: "invalid_grant" }), {
          status: 400,
          headers: { "Content-Type": "application/json", ...CORS },
        });
      }

      let authCodeData;
      try {
        authCodeData = JSON.parse(authCodeJson);
        log(`Auth code data parsed: ${JSON.stringify(authCodeData)}`);
      } catch (parseError) {
        log(`Error parsing auth code data: ${parseError}`);
        return new Response(JSON.stringify({ error: "server_error" }), {
          status: 500,
          headers: { "Content-Type": "application/json", ...CORS },
        });
      }

      // Verify the code_verifier against the stored code_challenge
      if (authCodeData.code_challenge) {
        log("Verifying PKCE code challenge");
        const encoder = new TextEncoder();
        const data = encoder.encode(codeVerifier);
        const digest = await crypto.subtle.digest("SHA-256", data);

        // Convert to base64url encoding
        const base64Digest = btoa(
          String.fromCharCode(...new Uint8Array(digest)),
        )
          .replace(/\+/g, "-")
          .replace(/\//g, "_")
          .replace(/=/g, "");

        log("Code challenge comparison:", {
          stored: authCodeData.code_challenge,
          computed: base64Digest,
          match: base64Digest === authCodeData.code_challenge,
        });

        if (base64Digest !== authCodeData.code_challenge) {
          log("PKCE verification failed");
          return new Response(JSON.stringify({ error: "invalid_grant" }), {
            status: 400,
            headers: { "Content-Type": "application/json", ...CORS },
          });
        }
      }

      // Delete the used authorization code
      try {
        await c.env.OAUTH_KV.delete(`auth_code:${code}`);
        log("Used authorization code deleted");
      } catch (deleteError) {
        log(`Error deleting used auth code: ${deleteError}`);
        // Continue anyway since this isn't critical
      }

      // Generate JWT access token instead of UUID
      const encoder = new TextEncoder();
      const secret = encoder.encode(
        c.env.JWT_SECRET || "default-jwt-secret-key",
      );

      // Create JWT payload
      const accessTokenPayload = {
        sub: authCodeData.sub,
        email: authCodeData.email,
        client_id: clientId,
        scope: "openid profile email",
        iss: c.env.STYTCH_PROJECT_ID,
        aud: clientId,
        exp: Math.floor(Date.now() / 1000) + 3600, // 1 hour expiry
        iat: Math.floor(Date.now() / 1000),
      };

      // Sign JWT
      const accessToken = await new SignJWT(accessTokenPayload)
        .setProtectedHeader({ alg: "HS256" })
        .setIssuedAt()
        .setExpirationTime("1h")
        .sign(secret);

      log(`Generated JWT access token: ${accessToken.substring(0, 20)}...`);

      // Generate refresh token (still using UUID for simplicity)
      const refreshToken = crypto.randomUUID();

      // Store token information - use a hash of the token as the key to avoid length limits
      const tokenHash = await crypto.subtle.digest(
        "SHA-256",
        encoder.encode(accessToken),
      );
      const tokenKey = btoa(String.fromCharCode(...new Uint8Array(tokenHash)))
        .replace(/\+/g, "-")
        .replace(/\//g, "_")
        .replace(/=/g, "")
        .substring(0, 32); // Use first 32 chars of hash

      try {
        log(`Storing access token with key: access_token:${tokenKey}`);
        await c.env.OAUTH_KV.put(
          `access_token:${tokenKey}`,
          JSON.stringify({
            token: accessToken,
            hash: tokenKey,
            ...accessTokenPayload,
          }),
          { expirationTtl: 3600 },
        );

        // Also store a mapping from the full token to the hash for validation
        await c.env.OAUTH_KV.put(`token_hash:${tokenKey}`, accessToken, {
          expirationTtl: 3600,
        });

        log("Storing refresh token");
        await c.env.OAUTH_KV.put(
          `refresh_token:${refreshToken}`,
          JSON.stringify({
            sub: authCodeData.sub,
            client_id: clientId,
          }),
          { expirationTtl: 30 * 24 * 60 * 60 },
        );

        log("Token data successfully stored");
      } catch (storeError) {
        log(`Error storing token data: ${storeError}`);
        return new Response(JSON.stringify({ error: "server_error" }), {
          status: 500,
          headers: { "Content-Type": "application/json", ...CORS },
        });
      }

      // Return the tokens
      const tokenResponse = {
        access_token: accessToken,
        token_type: "Bearer",
        expires_in: 3600,
        refresh_token: refreshToken,
        scope: "openid profile email",
      };

      log("Returning token response");
      return new Response(JSON.stringify(tokenResponse), {
        status: 200,
        headers: { "Content-Type": "application/json", ...CORS },
      });
    } catch (error) {
      console.error(`Token endpoint error: ${error}`);
      return new Response(JSON.stringify({ error: "server_error" }), {
        status: 500,
        headers: { "Content-Type": "application/json", ...CORS },
      });
    }
  })

  // Messages endpoint for all paths that start with /messages
  .post("/messages*", async (c) => {
    log("All messages endpoints hit");
    const REMOTE_MCP_SERVER_URL =
      c.env.REMOTE_MCP_SERVER_URL || "http://localhost:8000";
    const sid = new URL(c.req.url).searchParams.get("session_id");

    if (!sid) {
      return new Response("Missing session_id", {
        status: 400,
        headers: CORS,
      });
    }

    // Read the body
    const body = await c.req.text();
    const authHeader = c.req.header("Authorization") || "";
    let userEmail = "unknown";

    if (authHeader.startsWith("Bearer ")) {
      const token = authHeader.slice(7);
      const claims = decodeJwt(token);
      userEmail =
        claims.email || claims.preferred_username || claims.sub || "unknown";
    }

    log(`[Proxy] user=${userEmail}  query=${body}`);

    let sendToBQ = false;
    let parsed;
    let domain = null;
    let toolName = null;
    let sanitizedBody = body; // Default to original body

    try {
      parsed = JSON.parse(body);
      const args = parsed.params?.arguments;

      // Check if this is a think tool call
      toolName = parsed.params?.name;
      if (toolName === "think") {
        sendToBQ = false;
        log("[BigQuery] Skipping think tool call");
      } else if (args && Object.keys(args).length > 0) {
        // Extract domain from the arguments (for search/fetch tools)
        domain = args.domain || null;

        // Skip logging if domain is "thinking" or "think"
        if (domain === "thinking" || domain === "think") {
          sendToBQ = false;
        } else {
          sendToBQ = true;
        }

        // Sanitize sensitive data before logging to BigQuery
        if (sendToBQ) {
          // Use the comprehensive sanitization function
          const sanitized = sanitizeObject(parsed);
          sanitizedBody = JSON.stringify(sanitized);

          // Log if we actually sanitized something
          if (JSON.stringify(parsed) !== sanitizedBody) {
            log(
              "[BigQuery] Sanitized sensitive fields from query before logging",
            );
          }
        }
      }
    } catch (e) {
      console.log("[BigQuery] skipping insert—cannot parse JSON body", e);
    }

    const { BQ_SA_KEY_JSON, BQ_PROJECT_ID, BQ_DATASET, BQ_TABLE } = c.env;

    if (sendToBQ && BQ_SA_KEY_JSON && BQ_PROJECT_ID && BQ_DATASET && BQ_TABLE) {
      const eventRow = {
        timestamp: new Date().toISOString(),
        userEmail,
        query: sanitizedBody, // Use sanitized body instead of original
      };
      // fire & forget
      c.executionCtx.waitUntil(
        insertEvent(c.env, eventRow).catch((error) => {
          console.error("[BigQuery] Insert failed:", error);
        }),
      );
    } else {
      const missing = [
        !sendToBQ
          ? toolName === "think"
            ? "think tool"
            : domain === "thinking" || domain === "think"
            ? `domain is ${domain}`
            : "no query args"
          : null,
        !BQ_SA_KEY_JSON && "BQ_SA_KEY_JSON",
        !BQ_PROJECT_ID && "BQ_PROJECT_ID",
        !BQ_DATASET && "BQ_DATASET",
        !BQ_TABLE && "BQ_TABLE",
      ].filter(Boolean);
      console.log("[BigQuery] skipping insert—", missing.join(", "));
    }

    // Make a new Request object with the body we've already read
    const newRequest = new Request(c.req.url, {
      method: c.req.method,
      headers: c.req.headers,
      body: body,
    });

    // Forward everything to proxyPost like the auth-less version does
    return proxyPost(newRequest, REMOTE_MCP_SERVER_URL, "/messages", sid);
  })

  // SSE endpoint (protected with bearer token authentication)
  .use("/sse", stytchBearerTokenAuthMiddleware)
  .get("/sse", (c) => {
    log("SSE endpoint hit");
    const REMOTE_MCP_SERVER_URL =
      c.env.REMOTE_MCP_SERVER_URL || "http://localhost:8000";
    return serveSSE(c.req, REMOTE_MCP_SERVER_URL);
  })
  // MCP endpoint (alias for SSE, protected with bearer token authentication)
  .use("/mcp", stytchBearerTokenAuthMiddleware)
  .get("/mcp", (c) => {
    log("MCP GET endpoint hit");
    const REMOTE_MCP_SERVER_URL =
      c.env.REMOTE_MCP_SERVER_URL || "http://localhost:8000";
    return serveSSE(c.req, REMOTE_MCP_SERVER_URL);
  })
  .post("/mcp", async (c) => {
    log("MCP POST endpoint hit - Streamable HTTP transport");
    const REMOTE_MCP_SERVER_URL =
      c.env.REMOTE_MCP_SERVER_URL || "http://localhost:8000";

    // Extract and validate session ID
    const rawSessionId = new URL(c.req.url).searchParams.get("session_id");
    const sessionId = validateSessionId(rawSessionId);

    // Process the request with proper validation and error handling
    return processMcpRequest(c.req, REMOTE_MCP_SERVER_URL, sessionId);
  })
  .get("/events", (c) => {
    log("Events endpoint hit");
    const REMOTE_MCP_SERVER_URL =
      c.env.REMOTE_MCP_SERVER_URL || "http://localhost:8000";
    return serveSSE(c.req, REMOTE_MCP_SERVER_URL);
  })

  // Default 404 response
  .all(
    "*",
    () =>
      new Response("Not Found", {
        status: 404,
        headers: CORS,
      }),
  );

// Export the app as the main worker fetch handler
export default {
  fetch: (request, env, ctx) => {
    // Initialize DEBUG from environment variables
    DEBUG = env.DEBUG === "true" || env.DEBUG === true;

    return app.fetch(request, env, ctx);
  },
};

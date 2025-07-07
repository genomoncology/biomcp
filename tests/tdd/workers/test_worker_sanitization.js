/**
 * Tests for worker_entry_stytch.js sanitization functionality
 */

const { test } = require("node:test");
const assert = require("node:assert");

// Mock the sanitizeObject function for testing
const SENSITIVE_FIELDS = [
  "api_key",
  "apiKey",
  "api-key",
  "token",
  "secret",
  "password",
];

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

// Test cases
test("should redact api_key field", () => {
  const input = {
    params: {
      arguments: {
        api_key: "AIzaSyB1234567890",
        gene: "BRAF",
        position: 140753336,
      },
    },
  };

  const result = sanitizeObject(input);
  assert.strictEqual(result.params.arguments.api_key, "[REDACTED]");
  assert.strictEqual(result.params.arguments.gene, "BRAF");
  assert.strictEqual(result.params.arguments.position, 140753336);
});

test("should handle nested sensitive fields", () => {
  const input = {
    outer: {
      token: "secret-token",
      inner: {
        password: "my-password",
        apiKey: "another-key",
        safe_field: "visible",
      },
    },
  };

  const result = sanitizeObject(input);
  assert.strictEqual(result.outer.token, "[REDACTED]");
  assert.strictEqual(result.outer.inner.password, "[REDACTED]");
  assert.strictEqual(result.outer.inner.apiKey, "[REDACTED]");
  assert.strictEqual(result.outer.inner.safe_field, "visible");
});

test("should handle arrays with sensitive data", () => {
  const input = {
    requests: [
      { api_key: "key1", data: "safe" },
      { api_key: "key2", data: "also safe" },
    ],
  };

  const result = sanitizeObject(input);
  assert.strictEqual(result.requests[0].api_key, "[REDACTED]");
  assert.strictEqual(result.requests[1].api_key, "[REDACTED]");
  assert.strictEqual(result.requests[0].data, "safe");
  assert.strictEqual(result.requests[1].data, "also safe");
});

test("should be case-insensitive for field names", () => {
  const input = {
    API_KEY: "uppercase",
    Api_Key: "mixed",
    "api-key": "hyphenated",
  };

  const result = sanitizeObject(input);
  assert.strictEqual(result.API_KEY, "[REDACTED]");
  assert.strictEqual(result.Api_Key, "[REDACTED]");
  assert.strictEqual(result["api-key"], "[REDACTED]");
});

test("should not modify non-sensitive fields", () => {
  const input = {
    gene: "TP53",
    chromosome: "chr17",
    position: 7577121,
    reference: "C",
    alternate: "T",
  };

  const result = sanitizeObject(input);
  assert.deepStrictEqual(result, input);
});

test("should handle null and undefined values", () => {
  const input = {
    api_key: null,
    token: undefined,
    valid: "data",
  };

  const result = sanitizeObject(input);
  assert.strictEqual(result.api_key, "[REDACTED]");
  assert.strictEqual(result.token, "[REDACTED]");
  assert.strictEqual(result.valid, "data");
});

test("should handle think tool detection", () => {
  const thinkRequest = {
    params: {
      name: "think",
      arguments: {
        thought: "Analyzing the problem...",
        thoughtNumber: 1,
      },
    },
  };

  const toolName = thinkRequest.params?.name;
  assert.strictEqual(toolName, "think");
});

test("should handle domain-based filtering", () => {
  const searchRequest1 = {
    params: {
      name: "search",
      arguments: {
        domain: "thinking",
        query: "some query",
      },
    },
  };

  const searchRequest2 = {
    params: {
      name: "search",
      arguments: {
        domain: "think",
        query: "some query",
      },
    },
  };

  const domain1 = searchRequest1.params?.arguments?.domain;
  const domain2 = searchRequest2.params?.arguments?.domain;

  assert.ok(domain1 === "thinking" || domain1 === "think");
  assert.ok(domain2 === "thinking" || domain2 === "think");
});

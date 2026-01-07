/**
 * CORS Utility Tests
 *
 * Tests for getCorsHeaders function which builds CORS response headers.
 */

import { describe, it, expect } from 'vitest';
import { getCorsHeaders } from '../../src/utils/cors';
import { parseTestEnv } from '../helpers/env';
import type { ParsedEnv } from '../../src/env';

// Helper to create a ParsedEnv for testing with specific allowed origins
const createMockEnv = (allowedOrigins: string[] = []): ParsedEnv => parseTestEnv({ ALLOWED_ORIGINS: allowedOrigins.join(',') });

// Helper to create a Request with Origin header
const createRequest = (url: string, origin?: string): Request => {
  const headers = new Headers();
  if (origin) {
    headers.set('Origin', origin);
  }
  return new Request(url, { headers });
};

describe('getCorsHeaders', () => {
  describe('discovery endpoints (/.well-known/*)', () => {
    const expectedDiscoveryHeaders = {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, HEAD, OPTIONS',
      'Access-Control-Max-Age': '86400',
    };

    it('should return full CORS headers for oauth-authorization-server', () => {
      const request = createRequest('https://example.com/.well-known/oauth-authorization-server');
      const env = createMockEnv();
      const headers = getCorsHeaders(request, env);
      expect(headers).toEqual(expectedDiscoveryHeaders);
    });

    it('should return full CORS headers for oauth-protected-resource', () => {
      const request = createRequest('https://example.com/.well-known/oauth-protected-resource');
      const env = createMockEnv();
      const headers = getCorsHeaders(request, env);
      expect(headers).toEqual(expectedDiscoveryHeaders);
    });

    it('should return full CORS headers for path-specific discovery endpoints', () => {
      const request = createRequest('https://example.com/.well-known/oauth-authorization-server/mcp');
      const env = createMockEnv();
      const headers = getCorsHeaders(request, env);
      expect(headers).toEqual(expectedDiscoveryHeaders);
    });

    it('should return wildcard CORS regardless of Origin header', () => {
      const request = createRequest('https://example.com/.well-known/oauth-authorization-server', 'https://attacker.com');
      const env = createMockEnv();
      const headers = getCorsHeaders(request, env);
      expect(headers).toEqual(expectedDiscoveryHeaders);
    });

    it('should return full CORS headers even without Origin header', () => {
      const request = createRequest('https://example.com/.well-known/oauth-protected-resource');
      const env = createMockEnv();
      const headers = getCorsHeaders(request, env);
      expect(headers).toEqual(expectedDiscoveryHeaders);
    });
  });

  describe('missing or invalid origin', () => {
    it('should return empty object when Origin header is missing', () => {
      const request = createRequest('https://example.com/api');
      const env = createMockEnv();
      expect(getCorsHeaders(request, env)).toEqual({});
    });

    it('should return empty object when Origin is invalid URL', () => {
      const request = createRequest('https://example.com/api', 'not-a-valid-url');
      const env = createMockEnv();
      expect(getCorsHeaders(request, env)).toEqual({});
    });

    it('should return empty object when Origin is empty string', () => {
      const request = createRequest('https://example.com/api', '');
      const env = createMockEnv();
      expect(getCorsHeaders(request, env)).toEqual({});
    });
  });

  describe('same-origin requests', () => {
    it('should allow requests from same origin as request URL', () => {
      const request = createRequest('https://example.com/api', 'https://example.com');
      const env = createMockEnv();
      const headers = getCorsHeaders(request, env);
      expect(headers['Access-Control-Allow-Origin']).toBe('https://example.com');
    });

    it('should match origin exactly (port must match)', () => {
      const request = createRequest('https://example.com:8080/api', 'https://example.com:8080');
      const env = createMockEnv();
      const headers = getCorsHeaders(request, env);
      expect(headers['Access-Control-Allow-Origin']).toBe('https://example.com:8080');
    });
  });

  describe('allowed origins from environment', () => {
    it('should allow origin from ALLOWED_ORIGINS list', () => {
      const request = createRequest('https://api.example.com/endpoint', 'https://app.example.com');
      const env = createMockEnv(['https://app.example.com']);
      const headers = getCorsHeaders(request, env);
      expect(headers['Access-Control-Allow-Origin']).toBe('https://app.example.com');
    });

    it('should allow origin when multiple ALLOWED_ORIGINS are configured', () => {
      const request = createRequest('https://api.example.com/endpoint', 'https://app2.example.com');
      const env = createMockEnv(['https://app1.example.com', 'https://app2.example.com', 'https://app3.example.com']);
      const headers = getCorsHeaders(request, env);
      expect(headers['Access-Control-Allow-Origin']).toBe('https://app2.example.com');
    });

    it('should reject origin not in ALLOWED_ORIGINS list', () => {
      const request = createRequest('https://api.example.com/endpoint', 'https://attacker.com');
      const env = createMockEnv(['https://app.example.com']);
      expect(getCorsHeaders(request, env)).toEqual({});
    });

    it('should skip invalid URLs in ALLOWED_ORIGINS gracefully', () => {
      const request = createRequest('https://api.example.com/endpoint', 'https://app.example.com');
      const env = createMockEnv(['not-a-url', 'https://app.example.com', 'also-invalid']);
      const headers = getCorsHeaders(request, env);
      expect(headers['Access-Control-Allow-Origin']).toBe('https://app.example.com');
    });
  });

  describe('response header values', () => {
    it('should include all required CORS headers', () => {
      const request = createRequest('https://example.com/api', 'https://example.com');
      const env = createMockEnv();
      const headers = getCorsHeaders(request, env);

      expect(headers['Access-Control-Allow-Origin']).toBe('https://example.com');
      expect(headers['Access-Control-Allow-Methods']).toBe('GET, POST, OPTIONS, HEAD, DELETE');
      expect(headers['Access-Control-Allow-Headers']).toBe('Authorization, Content-Type, Mcp-Session-Id, Last-Event-ID');
      expect(headers['Access-Control-Max-Age']).toBe('86400');
      expect(headers['Access-Control-Expose-Headers']).toBe('mcp-session-id');
      expect(headers['Vary']).toBe('Origin');
    });
  });

  describe('edge cases', () => {
    it('should handle HTTP origins (not just HTTPS)', () => {
      const request = createRequest('http://localhost:3000/api', 'http://localhost:3000');
      const env = createMockEnv();
      const headers = getCorsHeaders(request, env);
      expect(headers['Access-Control-Allow-Origin']).toBe('http://localhost:3000');
    });

    it('should reject origin with different protocol', () => {
      const request = createRequest('https://example.com/api', 'http://example.com');
      const env = createMockEnv();
      expect(getCorsHeaders(request, env)).toEqual({});
    });

    it('should reject origin with different port', () => {
      const request = createRequest('https://example.com/api', 'https://example.com:8080');
      const env = createMockEnv();
      expect(getCorsHeaders(request, env)).toEqual({});
    });

    it('should handle origin with path (extracts origin only)', () => {
      const request = createRequest('https://example.com/api', 'https://example.com/other/path');
      const env = createMockEnv();
      const headers = getCorsHeaders(request, env);
      // Origin is extracted from URL, path is ignored
      expect(headers['Access-Control-Allow-Origin']).toBe('https://example.com');
    });
  });
});

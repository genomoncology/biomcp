/**
 * Auth Middleware Unit Tests
 *
 * Tests for path normalization and resource URI computation.
 */

import { describe, it, expect } from 'vitest';
import { Hono } from 'hono';
import { normalizeBasePath, getExpectedResource } from '../../src/auth/middleware';

describe('normalizeBasePath', () => {
  describe('single segment paths', () => {
    it('should return /mcp for /mcp', () => {
      expect(normalizeBasePath('/mcp')).toBe('/mcp');
    });

    it('should return /mcp for /mcp/', () => {
      expect(normalizeBasePath('/mcp/')).toBe('/mcp');
    });

    it('should return /api for /api', () => {
      expect(normalizeBasePath('/api')).toBe('/api');
    });
  });

  describe('multi-segment paths', () => {
    it('should return /mcp for /mcp/foo', () => {
      expect(normalizeBasePath('/mcp/foo')).toBe('/mcp');
    });

    it('should return /mcp for /mcp/foo/bar', () => {
      expect(normalizeBasePath('/mcp/foo/bar')).toBe('/mcp');
    });

    it('should return /mcp for /mcp/foo/bar/', () => {
      expect(normalizeBasePath('/mcp/foo/bar/')).toBe('/mcp');
    });

    it('should return /api for /api/v1/users', () => {
      expect(normalizeBasePath('/api/v1/users')).toBe('/api');
    });
  });

  describe('root path', () => {
    it('should return / for /', () => {
      expect(normalizeBasePath('/')).toBe('/');
    });

    it('should return / for empty string', () => {
      expect(normalizeBasePath('')).toBe('/');
    });
  });

  describe('edge cases', () => {
    it('should handle paths with multiple consecutive slashes', () => {
      // filter(Boolean) removes empty strings from split
      expect(normalizeBasePath('//mcp')).toBe('/mcp');
      expect(normalizeBasePath('/mcp//')).toBe('/mcp');
      expect(normalizeBasePath('//mcp//foo')).toBe('/mcp');
    });

    it('should handle paths with special characters in segment', () => {
      expect(normalizeBasePath('/mcp-server')).toBe('/mcp-server');
      expect(normalizeBasePath('/mcp_server')).toBe('/mcp_server');
      expect(normalizeBasePath('/mcp.v1')).toBe('/mcp.v1');
    });

    it('should handle paths with encoded characters', () => {
      expect(normalizeBasePath('/mcp%20server')).toBe('/mcp%20server');
      expect(normalizeBasePath('/mcp%2Fencoded')).toBe('/mcp%2Fencoded');
    });

    it('should handle very long paths', () => {
      expect(normalizeBasePath('/mcp/a/b/c/d/e/f/g/h/i/j')).toBe('/mcp');
    });
  });
});

describe('getExpectedResource', () => {
  /**
   * Helper to get the expected resource for a given URL.
   * Creates a minimal Hono app to capture the context.
   */
  async function getResourceForUrl(url: string): Promise<string> {
    let capturedResource: string | null = null;

    const app = new Hono();
    app.all('*', (c) => {
      capturedResource = getExpectedResource(c);
      return c.text('ok');
    });

    await app.request(url);

    if (capturedResource === null) {
      throw new Error('Resource was not captured');
    }

    return capturedResource;
  }

  describe('standard MCP paths', () => {
    it('should return origin + /mcp for /mcp', async () => {
      const resource = await getResourceForUrl('https://gateway.example.com/mcp');
      expect(resource).toBe('https://gateway.example.com/mcp');
    });

    it('should return origin + /mcp for /mcp/', async () => {
      const resource = await getResourceForUrl('https://gateway.example.com/mcp/');
      expect(resource).toBe('https://gateway.example.com/mcp');
    });

    it('should return origin + /mcp for /mcp/messages', async () => {
      const resource = await getResourceForUrl('https://gateway.example.com/mcp/messages');
      expect(resource).toBe('https://gateway.example.com/mcp');
    });

    it('should return origin + /mcp for /mcp/sse/events', async () => {
      const resource = await getResourceForUrl('https://gateway.example.com/mcp/sse/events');
      expect(resource).toBe('https://gateway.example.com/mcp');
    });
  });

  describe('different origins', () => {
    it('should handle http origin', async () => {
      const resource = await getResourceForUrl('http://localhost:8787/mcp');
      expect(resource).toBe('http://localhost:8787/mcp');
    });

    it('should handle origin with port', async () => {
      const resource = await getResourceForUrl('https://gateway.example.com:8443/mcp');
      expect(resource).toBe('https://gateway.example.com:8443/mcp');
    });

    it('should handle subdomain', async () => {
      const resource = await getResourceForUrl('https://api.gateway.example.com/mcp');
      expect(resource).toBe('https://api.gateway.example.com/mcp');
    });
  });

  describe('root path', () => {
    it('should return origin + / for /', async () => {
      const resource = await getResourceForUrl('https://gateway.example.com/');
      expect(resource).toBe('https://gateway.example.com/');
    });

    it('should return origin + / for root without trailing slash', async () => {
      // URL constructor normalizes this to include trailing slash for root
      const resource = await getResourceForUrl('https://gateway.example.com');
      expect(resource).toBe('https://gateway.example.com/');
    });
  });

  describe('query strings and fragments are stripped', () => {
    it('should ignore query string', async () => {
      const resource = await getResourceForUrl('https://gateway.example.com/mcp?foo=bar');
      expect(resource).toBe('https://gateway.example.com/mcp');
    });

    it('should ignore fragment', async () => {
      // Note: Fragments are not sent to server, but URL constructor handles them
      const resource = await getResourceForUrl('https://gateway.example.com/mcp#section');
      expect(resource).toBe('https://gateway.example.com/mcp');
    });

    it('should ignore both query string and fragment', async () => {
      const resource = await getResourceForUrl('https://gateway.example.com/mcp?foo=bar#section');
      expect(resource).toBe('https://gateway.example.com/mcp');
    });
  });

  describe('different base paths', () => {
    it('should work with /api base path', async () => {
      const resource = await getResourceForUrl('https://gateway.example.com/api/v1/users');
      expect(resource).toBe('https://gateway.example.com/api');
    });

    it('should work with /oauth base path', async () => {
      const resource = await getResourceForUrl('https://gateway.example.com/oauth/token');
      expect(resource).toBe('https://gateway.example.com/oauth');
    });

    it('should handle hyphenated paths', async () => {
      const resource = await getResourceForUrl('https://gateway.example.com/mcp-server/tools');
      expect(resource).toBe('https://gateway.example.com/mcp-server');
    });
  });
});

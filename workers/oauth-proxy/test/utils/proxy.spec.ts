/**
 * Proxy Utility Tests
 *
 * Tests for the proxyPost function, specifically auth token handling.
 * Uses fetchMock from cloudflare:test for mocking outbound fetch.
 */

import { fetchMock } from 'cloudflare:test';
import { describe, it, expect, vi, beforeAll, afterEach } from 'vitest';
import { proxyPost, buildProxyHeaders } from '../../src/utils/proxy';
import type { Logger } from '../../src/utils/logger';
import { toHeaders } from '../helpers/fetch-mock';

// Mock logger
const createMockLogger = (): Logger => ({
  debug: vi.fn(),
  info: vi.fn(),
  warn: vi.fn(),
  error: vi.fn(),
  event: vi.fn(),
  child: vi.fn().mockReturnThis(),
});

describe('buildProxyHeaders', () => {
  it('should build headers with required fields only', () => {
    const headers = buildProxyHeaders({
      accept: 'application/json',
    });

    expect(headers['Accept']).toBe('application/json');
    expect(headers['User-Agent']).toBe('BioMCP-Proxy/1.0');
    expect(headers['Content-Type']).toBeUndefined();
    expect(headers['Authorization']).toBeUndefined();
  });

  it('should include Content-Type when provided', () => {
    const headers = buildProxyHeaders({
      accept: 'application/json',
      contentType: 'text/plain',
    });

    expect(headers['Accept']).toBe('application/json');
    expect(headers['Content-Type']).toBe('text/plain');
    expect(headers['User-Agent']).toBe('BioMCP-Proxy/1.0');
  });

  it('should include Authorization header when authToken is provided', () => {
    const headers = buildProxyHeaders({
      accept: 'application/json',
      authToken: 'test-token-12345',
    });

    expect(headers['Authorization']).toBe('Bearer test-token-12345');
    expect(headers['Accept']).toBe('application/json');
    expect(headers['User-Agent']).toBe('BioMCP-Proxy/1.0');
  });

  it('should build headers with all options', () => {
    const headers = buildProxyHeaders({
      accept: 'text/event-stream',
      contentType: 'application/json',
      authToken: 'my-secret-token',
    });

    expect(headers['Accept']).toBe('text/event-stream');
    expect(headers['Content-Type']).toBe('application/json');
    expect(headers['Authorization']).toBe('Bearer my-secret-token');
    expect(headers['User-Agent']).toBe('BioMCP-Proxy/1.0');
  });

  it('should handle SSE accept header', () => {
    const headers = buildProxyHeaders({
      accept: 'application/json, text/event-stream',
    });

    expect(headers['Accept']).toBe('application/json, text/event-stream');
    expect(headers['User-Agent']).toBe('BioMCP-Proxy/1.0');
  });

  it('should not include Authorization when authToken is undefined', () => {
    const headers = buildProxyHeaders({
      accept: 'application/json',
      authToken: undefined,
    });

    expect(headers['Authorization']).toBeUndefined();
  });

  it('should not include Content-Type when contentType is undefined', () => {
    const headers = buildProxyHeaders({
      accept: 'application/json',
      contentType: undefined,
    });

    expect(headers['Content-Type']).toBeUndefined();
  });
});

describe('proxyPost', () => {
  let mockLogger: Logger;
  let capturedHeaders: Headers | null = null;

  beforeAll(() => {
    fetchMock.activate();
    // Safe to disableNetConnect() here - these unit tests only call proxyPost() directly,
    // not internal worker routes that would need real fetch
    fetchMock.disableNetConnect();
  });

  afterEach(() => {
    fetchMock.assertNoPendingInterceptors();
    capturedHeaders = null;
  });

  // Helper to set up the fetch mock for each test
  const setupFetchMock = () => {
    fetchMock
      .get('https://remote.com')
      .intercept({
        method: 'POST',
        path: (path) => path.startsWith('/mcp'),
      })
      .reply((opts) => {
        capturedHeaders = toHeaders(opts.headers);
        return {
          statusCode: 200,
          data: JSON.stringify({ result: 'ok' }),
          responseOptions: { headers: { 'content-type': 'application/json' } },
        };
      });
  };

  describe('Authorization header', () => {
    const validToken = 'secret-token-that-is-at-least-32-chars';

    it('should include Authorization header when authToken is provided', async () => {
      mockLogger = createMockLogger();
      setupFetchMock();

      const mockRequest = new Request('http://example.com', {
        method: 'POST',
        body: JSON.stringify({ test: 'data' }),
      });

      await proxyPost(mockLogger, mockRequest, 'https://remote.com', '/mcp', 'session-123', {
        authToken: validToken,
      });

      expect(capturedHeaders?.get('Authorization')).toBe(`Bearer ${validToken}`);
    });

    it('should omit Authorization header when authToken is undefined', async () => {
      mockLogger = createMockLogger();
      setupFetchMock();

      const mockRequest = new Request('http://example.com', {
        method: 'POST',
        body: JSON.stringify({ test: 'data' }),
      });

      await proxyPost(mockLogger, mockRequest, 'https://remote.com', '/mcp', 'session-123');

      expect(capturedHeaders?.get('Authorization')).toBeNull();
    });

    it('should omit Authorization header when options is undefined', async () => {
      mockLogger = createMockLogger();
      setupFetchMock();

      const mockRequest = new Request('http://example.com', {
        method: 'POST',
        body: JSON.stringify({ test: 'data' }),
      });

      await proxyPost(mockLogger, mockRequest, 'https://remote.com', '/mcp', 'session-123', undefined);

      expect(capturedHeaders?.get('Authorization')).toBeNull();
    });

    it('should format token correctly as Bearer scheme', async () => {
      mockLogger = createMockLogger();
      setupFetchMock();

      const mockRequest = new Request('http://example.com', {
        method: 'POST',
        body: JSON.stringify({ test: 'data' }),
      });

      await proxyPost(mockLogger, mockRequest, 'https://remote.com', '/mcp', 'session-123', {
        authToken: validToken,
      });

      const authHeader = capturedHeaders?.get('Authorization');
      expect(authHeader).toMatch(/^Bearer /);
      expect(authHeader).toBe(`Bearer ${validToken}`);
    });
  });

  describe('standard headers', () => {
    it('should include Content-Type, Accept, and User-Agent headers', async () => {
      mockLogger = createMockLogger();
      setupFetchMock();

      const mockRequest = new Request('http://example.com', {
        method: 'POST',
        body: JSON.stringify({ test: 'data' }),
      });

      await proxyPost(mockLogger, mockRequest, 'https://remote.com', '/mcp', 'session-123');

      expect(capturedHeaders?.get('Content-Type')).toBe('application/json');
      expect(capturedHeaders?.get('Accept')).toBe('application/json, text/event-stream');
      expect(capturedHeaders?.get('User-Agent')).toBe('BioMCP-Proxy/1.0');
    });
  });
});

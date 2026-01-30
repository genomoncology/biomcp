/**
 * JSON-RPC Schema Tests
 *
 * Tests for parsing MCP JSON-RPC requests for analytics.
 */

import { describe, it, expect } from 'vitest';
import { parseJsonRpcRequest } from '../../src/mcp/schemas';

describe('parseJsonRpcRequest', () => {
  describe('valid JSON-RPC requests', () => {
    it('parses basic request with method', () => {
      const body = JSON.stringify({
        jsonrpc: '2.0',
        method: 'initialize',
        id: 1,
      });

      const result = parseJsonRpcRequest(body);

      expect(result).toEqual({
        method: 'initialize',
        toolName: undefined,
        requestId: 1,
      });
    });

    it('parses request with string id', () => {
      const body = JSON.stringify({
        jsonrpc: '2.0',
        method: 'tools/list',
        id: 'request-123',
      });

      const result = parseJsonRpcRequest(body);

      expect(result).toEqual({
        method: 'tools/list',
        toolName: undefined,
        requestId: 'request-123',
      });
    });

    it('parses request with null id (notification)', () => {
      const body = JSON.stringify({
        jsonrpc: '2.0',
        method: 'notifications/cancelled',
        id: null,
      });

      const result = parseJsonRpcRequest(body);

      expect(result).toEqual({
        method: 'notifications/cancelled',
        toolName: undefined,
        requestId: null,
      });
    });

    it('parses request without id', () => {
      const body = JSON.stringify({
        jsonrpc: '2.0',
        method: 'ping',
      });

      const result = parseJsonRpcRequest(body);

      expect(result).toEqual({
        method: 'ping',
        toolName: undefined,
        requestId: undefined,
      });
    });
  });

  describe('tools/call requests', () => {
    it('extracts tool name from tools/call', () => {
      const body = JSON.stringify({
        jsonrpc: '2.0',
        method: 'tools/call',
        id: 1,
        params: {
          name: 'search_database',
          arguments: { query: 'test' },
        },
      });

      const result = parseJsonRpcRequest(body);

      expect(result).toEqual({
        method: 'tools/call',
        toolName: 'search_database',
        requestId: 1,
      });
    });

    it('handles tools/call without arguments', () => {
      const body = JSON.stringify({
        jsonrpc: '2.0',
        method: 'tools/call',
        id: 2,
        params: {
          name: 'get_current_time',
        },
      });

      const result = parseJsonRpcRequest(body);

      expect(result).toEqual({
        method: 'tools/call',
        toolName: 'get_current_time',
        requestId: 2,
      });
    });

    it('handles tools/call with malformed params', () => {
      const body = JSON.stringify({
        jsonrpc: '2.0',
        method: 'tools/call',
        id: 3,
        params: { invalid: 'structure' }, // missing 'name'
      });

      const result = parseJsonRpcRequest(body);

      expect(result).toEqual({
        method: 'tools/call',
        toolName: undefined, // Failed to extract, but request still parsed
        requestId: 3,
      });
    });

    it('handles tools/call without params', () => {
      const body = JSON.stringify({
        jsonrpc: '2.0',
        method: 'tools/call',
        id: 4,
      });

      const result = parseJsonRpcRequest(body);

      expect(result).toEqual({
        method: 'tools/call',
        toolName: undefined,
        requestId: 4,
      });
    });
  });

  describe('other MCP methods', () => {
    it('parses resources/read', () => {
      const body = JSON.stringify({
        jsonrpc: '2.0',
        method: 'resources/read',
        id: 1,
        params: { uri: 'file:///path/to/file' },
      });

      const result = parseJsonRpcRequest(body);

      expect(result).toEqual({
        method: 'resources/read',
        toolName: undefined,
        requestId: 1,
      });
    });

    it('parses prompts/get', () => {
      const body = JSON.stringify({
        jsonrpc: '2.0',
        method: 'prompts/get',
        id: 1,
        params: { name: 'greeting' },
      });

      const result = parseJsonRpcRequest(body);

      expect(result).toEqual({
        method: 'prompts/get',
        toolName: undefined,
        requestId: 1,
      });
    });
  });

  describe('invalid inputs', () => {
    it('returns null for invalid JSON', () => {
      const result = parseJsonRpcRequest('not json');
      expect(result).toBeNull();
    });

    it('returns null for empty string', () => {
      const result = parseJsonRpcRequest('');
      expect(result).toBeNull();
    });

    it('returns null for wrong jsonrpc version', () => {
      const body = JSON.stringify({
        jsonrpc: '1.0',
        method: 'test',
        id: 1,
      });

      const result = parseJsonRpcRequest(body);
      expect(result).toBeNull();
    });

    it('returns null for missing jsonrpc field', () => {
      const body = JSON.stringify({
        method: 'test',
        id: 1,
      });

      const result = parseJsonRpcRequest(body);
      expect(result).toBeNull();
    });

    it('returns null for missing method field', () => {
      const body = JSON.stringify({
        jsonrpc: '2.0',
        id: 1,
      });

      const result = parseJsonRpcRequest(body);
      expect(result).toBeNull();
    });

    it('returns null for non-string method', () => {
      const body = JSON.stringify({
        jsonrpc: '2.0',
        method: 123,
        id: 1,
      });

      const result = parseJsonRpcRequest(body);
      expect(result).toBeNull();
    });

    it('returns null for array body', () => {
      const body = JSON.stringify([{ jsonrpc: '2.0', method: 'test', id: 1 }]);

      const result = parseJsonRpcRequest(body);
      expect(result).toBeNull();
    });

    it('returns null for null body', () => {
      const body = JSON.stringify(null);

      const result = parseJsonRpcRequest(body);
      expect(result).toBeNull();
    });
  });
});

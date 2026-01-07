/**
 * Sanitize Utility Tests
 *
 * Tests for sanitizeObject function which recursively redacts sensitive fields.
 */

import { describe, it, expect } from 'vitest';
import { sanitizeObject } from '../../src/utils/sanitize';

describe('sanitizeObject', () => {
  describe('primitives', () => {
    it('should return null unchanged', () => {
      expect(sanitizeObject(null)).toBe(null);
    });

    it('should return undefined unchanged', () => {
      expect(sanitizeObject(undefined)).toBe(undefined);
    });

    it('should return strings unchanged', () => {
      expect(sanitizeObject('hello')).toBe('hello');
      expect(sanitizeObject('')).toBe('');
    });

    it('should return numbers unchanged', () => {
      expect(sanitizeObject(123)).toBe(123);
      expect(sanitizeObject(0)).toBe(0);
      expect(sanitizeObject(-1)).toBe(-1);
    });

    it('should return booleans unchanged', () => {
      expect(sanitizeObject(true)).toBe(true);
      expect(sanitizeObject(false)).toBe(false);
    });
  });

  describe('arrays', () => {
    it('should return empty array as empty array', () => {
      expect(sanitizeObject([])).toEqual([]);
    });

    it('should preserve array of primitives', () => {
      expect(sanitizeObject([1, 'two', true, null])).toEqual([1, 'two', true, null]);
    });

    it('should recursively sanitize objects in arrays', () => {
      const input = [{ password: 'secret123' }, { name: 'test' }];
      const expected = [{ password: '[REDACTED]' }, { name: 'test' }];
      expect(sanitizeObject(input)).toEqual(expected);
    });

    it('should handle nested arrays', () => {
      const input = [[{ api_key: 'key123' }]];
      const expected = [[{ api_key: '[REDACTED]' }]];
      expect(sanitizeObject(input)).toEqual(expected);
    });
  });

  describe('objects', () => {
    it('should return empty object as empty object', () => {
      expect(sanitizeObject({})).toEqual({});
    });

    it('should preserve non-sensitive fields', () => {
      const input = { name: 'test', value: 123, active: true };
      expect(sanitizeObject(input)).toEqual(input);
    });

    it('should recursively sanitize nested objects', () => {
      const input = { user: { profile: { password: 'secret' } } };
      const expected = { user: { profile: { password: '[REDACTED]' } } };
      expect(sanitizeObject(input)).toEqual(expected);
    });
  });

  describe('sensitive field detection', () => {
    it('should redact "api_key" field', () => {
      expect(sanitizeObject({ api_key: 'abc123' })).toEqual({ api_key: '[REDACTED]' });
    });

    it('should redact "apiKey" field (camelCase)', () => {
      expect(sanitizeObject({ apiKey: 'abc123' })).toEqual({ apiKey: '[REDACTED]' });
    });

    it('should redact "api-key" field (kebab-case)', () => {
      expect(sanitizeObject({ 'api-key': 'abc123' })).toEqual({ 'api-key': '[REDACTED]' });
    });

    it('should redact "token" field', () => {
      expect(sanitizeObject({ token: 'abc123' })).toEqual({ token: '[REDACTED]' });
    });

    it('should redact "secret" field', () => {
      expect(sanitizeObject({ secret: 'abc123' })).toEqual({ secret: '[REDACTED]' });
    });

    it('should redact "password" field', () => {
      expect(sanitizeObject({ password: 'abc123' })).toEqual({ password: '[REDACTED]' });
    });

    it('should redact fields containing sensitive words (case-insensitive)', () => {
      expect(sanitizeObject({ API_KEY: 'abc123' })).toEqual({ API_KEY: '[REDACTED]' });
      expect(sanitizeObject({ PASSWORD: 'abc123' })).toEqual({ PASSWORD: '[REDACTED]' });
      expect(sanitizeObject({ Token: 'abc123' })).toEqual({ Token: '[REDACTED]' });
    });

    it('should redact fields with sensitive words as substrings', () => {
      expect(sanitizeObject({ access_token: 'abc123' })).toEqual({ access_token: '[REDACTED]' });
      expect(sanitizeObject({ auth_secret: 'abc123' })).toEqual({ auth_secret: '[REDACTED]' });
      expect(sanitizeObject({ my_api_key_value: 'abc123' })).toEqual({ my_api_key_value: '[REDACTED]' });
    });

    it('should preserve fields that do not match sensitive patterns', () => {
      const input = {
        username: 'john',
        email: 'john@example.com',
        id: 123,
      };
      expect(sanitizeObject(input)).toEqual(input);
    });
  });

  describe('mixed scenarios', () => {
    it('should handle complex nested structure', () => {
      const input = {
        user: {
          name: 'John',
          credentials: {
            password: 'secret123',
            api_key: 'key123',
          },
          preferences: {
            theme: 'dark',
          },
        },
        items: [{ access_token: 'tok1' }, { refresh_token: 'tok2' }],
        metadata: {
          created: '2024-01-01',
        },
      };

      const expected = {
        user: {
          name: 'John',
          credentials: {
            password: '[REDACTED]',
            api_key: '[REDACTED]',
          },
          preferences: {
            theme: 'dark',
          },
        },
        items: [{ access_token: '[REDACTED]' }, { refresh_token: '[REDACTED]' }],
        metadata: {
          created: '2024-01-01',
        },
      };

      expect(sanitizeObject(input)).toEqual(expected);
    });

    it('should redact keys containing sensitive words even as parent keys', () => {
      // "tokens" contains "token", so the entire key is redacted
      const input = { tokens: [{ id: 1 }] };
      expect(sanitizeObject(input)).toEqual({ tokens: '[REDACTED]' });
    });

    it('should handle null values in objects', () => {
      const input = { password: null, name: 'test' };
      // password key is sensitive, but value is null - still redact
      expect(sanitizeObject(input)).toEqual({ password: '[REDACTED]', name: 'test' });
    });

    it('should handle undefined values in objects', () => {
      const input = { api_key: undefined, name: 'test' };
      expect(sanitizeObject(input)).toEqual({ api_key: '[REDACTED]', name: 'test' });
    });
  });
});

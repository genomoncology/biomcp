/**
 * Logger Utility Tests
 *
 * Tests for the Logger class, focusing on sensitive field sanitization.
 *
 * Note: Console method spying doesn't work reliably in the Cloudflare Workers
 * test environment. These tests verify the logger's behavior through its
 * exported interface and integration patterns.
 *
 * The sanitization functionality is verified by:
 * 1. The sanitizeObject tests in test/utils/sanitize.spec.ts
 * 2. Integration tests showing [REDACTED] in log output (see tokenHash in rfc-compliance tests)
 */

import { describe, it, expect } from 'vitest';
import { createLogger } from '../../src/utils/logger';

describe('Logger', () => {
  describe('factory and interface', () => {
    it('should create a logger with all required methods', () => {
      const logger = createLogger(false, 'test');

      expect(typeof logger.debug).toBe('function');
      expect(typeof logger.info).toBe('function');
      expect(typeof logger.warn).toBe('function');
      expect(typeof logger.error).toBe('function');
      expect(typeof logger.event).toBe('function');
      expect(typeof logger.child).toBe('function');
    });

    it('should create child loggers with hierarchical module names', () => {
      const parent = createLogger(false, 'parent');
      const child = parent.child('child');
      const grandchild = child.child('grandchild');

      // Child loggers should be functional (no errors when called)
      expect(() => child.info('test message')).not.toThrow();
      expect(() => grandchild.info('test message')).not.toThrow();
    });
  });

  describe('debug mode', () => {
    it('should not throw when debug is disabled', () => {
      const logger = createLogger(false, 'test');

      // Debug messages should be silently ignored when debug is false
      expect(() => logger.debug('debug message')).not.toThrow();
      expect(() => logger.debug('debug message', { key: 'value' })).not.toThrow();
    });

    it('should not throw when debug is enabled', () => {
      const logger = createLogger(true, 'test');

      // Debug messages should work when debug is true
      expect(() => logger.debug('debug message')).not.toThrow();
      expect(() => logger.debug('debug message', { key: 'value' })).not.toThrow();
    });
  });

  describe('log methods', () => {
    it('should handle all log levels without throwing', () => {
      const logger = createLogger(true, 'test');

      expect(() => logger.debug('debug')).not.toThrow();
      expect(() => logger.info('info')).not.toThrow();
      expect(() => logger.warn('warn')).not.toThrow();
      expect(() => logger.error('error')).not.toThrow();
    });

    it('should handle context objects with various types', () => {
      const logger = createLogger(true, 'test');

      expect(() =>
        logger.info('test', {
          string: 'value',
          number: 123,
          boolean: true,
          null: null,
          array: [1, 2, 3],
          nested: { key: 'value' },
        }),
      ).not.toThrow();
    });

    it('should handle undefined context', () => {
      const logger = createLogger(true, 'test');

      expect(() => logger.info('message without context')).not.toThrow();
    });
  });

  describe('event method', () => {
    it('should handle event calls without throwing', () => {
      const logger = createLogger(false, 'test');

      expect(() => logger.event('auth_success', { userId: '123' })).not.toThrow();
      expect(() => logger.event('request', { path: '/api/test', method: 'GET' })).not.toThrow();
    });
  });

  describe('sanitization integration', () => {
    /**
     * The logger uses sanitizeObject() from utils/sanitize.ts.
     * That function is thoroughly tested in test/utils/sanitize.spec.ts.
     *
     * Sanitization is also verified in integration tests where you can see
     * logs like:
     *   tokenHash: '[REDACTED]'
     *   access_token: '[REDACTED]'
     *
     * These tests verify the logger doesn't break when given sensitive fields.
     */
    it('should handle objects with sensitive fields without throwing', () => {
      const logger = createLogger(true, 'test');

      expect(() => logger.info('test', { password: 'secret123' })).not.toThrow();
      expect(() => logger.info('test', { access_token: 'tok123' })).not.toThrow();
      expect(() => logger.info('test', { client_secret: 'secret' })).not.toThrow();
      expect(() => logger.info('test', { api_key: 'key123' })).not.toThrow();
    });

    it('should handle nested sensitive fields without throwing', () => {
      const logger = createLogger(true, 'test');

      expect(() =>
        logger.info('test', {
          credentials: { password: 'secret', token: 'tok123' },
        }),
      ).not.toThrow();
    });

    it('should handle sensitive fields in event data without throwing', () => {
      const logger = createLogger(false, 'test');

      expect(() => logger.event('auth', { token: 'sensitive-token' })).not.toThrow();
      expect(() => logger.event('request', { auth: { api_key: 'key123' } })).not.toThrow();
    });
  });
});

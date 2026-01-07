/**
 * Session Utility Tests
 *
 * Tests for validateSessionId function which validates session IDs.
 *
 * We validate UUID format only because:
 * - We generate all session IDs ourselves using crypto.randomUUID()
 * - Clients only send back the session ID we gave them
 * - UUID validation is stricter and avoids security concerns
 *
 * MCP spec allows visible ASCII (0x21-0x7E), max 128 chars, but we don't need
 * that flexibility since we control session ID generation.
 */

import { describe, it, expect, vi } from 'vitest';
import { validateSessionId } from '../../src/utils/session';
import type { Logger } from '../../src/utils/logger';

// Mock logger
const createMockLogger = (): Logger => ({
  debug: vi.fn(),
  info: vi.fn(),
  warn: vi.fn(),
  error: vi.fn(),
  event: vi.fn(),
  child: vi.fn().mockReturnThis(),
});

describe('validateSessionId', () => {
  describe('valid session IDs (UUID format only)', () => {
    it('should accept lowercase UUID format', () => {
      const logger = createMockLogger();
      const uuid = '550e8400-e29b-41d4-a716-446655440000';
      expect(validateSessionId(logger, uuid)).toBe(uuid);
    });

    it('should accept uppercase UUID format', () => {
      const logger = createMockLogger();
      const uuid = '550E8400-E29B-41D4-A716-446655440000';
      expect(validateSessionId(logger, uuid)).toBe(uuid);
    });

    it('should accept mixed case UUID format', () => {
      const logger = createMockLogger();
      const uuid = '550e8400-E29B-41d4-A716-446655440000';
      expect(validateSessionId(logger, uuid)).toBe(uuid);
    });

    it('should accept UUID v4 format (random)', () => {
      const logger = createMockLogger();
      // crypto.randomUUID() produces v4 UUIDs
      const uuid = crypto.randomUUID();
      expect(validateSessionId(logger, uuid)).toBe(uuid);
    });

    it('should accept various valid UUID versions', () => {
      const logger = createMockLogger();
      // UUID v1 (time-based)
      expect(validateSessionId(logger, '6ba7b810-9dad-11d1-80b4-00c04fd430c8')).toBeTruthy();
      // UUID v4 (random)
      expect(validateSessionId(logger, 'f47ac10b-58cc-4372-a567-0e02b2c3d479')).toBeTruthy();
    });
  });

  describe('invalid session IDs (non-UUID formats)', () => {
    it('should return null for undefined', () => {
      const logger = createMockLogger();
      expect(validateSessionId(logger, undefined)).toBe(null);
    });

    it('should return null for empty string', () => {
      const logger = createMockLogger();
      expect(validateSessionId(logger, '')).toBe(null);
    });

    it('should reject alphanumeric non-UUID strings', () => {
      const logger = createMockLogger();
      expect(validateSessionId(logger, 'abc123')).toBe(null);
      expect(validateSessionId(logger, 'session-id-123')).toBe(null);
      expect(validateSessionId(logger, 'session_id_123')).toBe(null);
    });

    it('should reject UUID without dashes', () => {
      const logger = createMockLogger();
      expect(validateSessionId(logger, '550e8400e29b41d4a716446655440000')).toBe(null);
    });

    it('should reject UUID with wrong segment lengths', () => {
      const logger = createMockLogger();
      // Wrong format
      expect(validateSessionId(logger, '550e8400-e29b-41d4-a716-4466554400')).toBe(null); // too short
      expect(validateSessionId(logger, '550e8400-e29b-41d4-a716-44665544000000')).toBe(null); // too long
    });

    it('should reject session ID with spaces', () => {
      const logger = createMockLogger();
      expect(validateSessionId(logger, 'session id')).toBe(null);
      expect(logger.debug).toHaveBeenCalled();
    });

    it('should reject session ID with control characters', () => {
      const logger = createMockLogger();
      expect(validateSessionId(logger, 'session\x00id')).toBe(null); // null char
      expect(validateSessionId(logger, 'session\tid')).toBe(null); // tab
      expect(validateSessionId(logger, 'session\nid')).toBe(null); // newline
    });

    it('should reject session ID with unicode', () => {
      const logger = createMockLogger();
      expect(validateSessionId(logger, 'séssion')).toBe(null);
      expect(validateSessionId(logger, '会话')).toBe(null);
    });

    it('should reject JWT-style session IDs', () => {
      const logger = createMockLogger();
      // We only accept UUIDs, not JWTs
      const jwt = 'eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.abc123';
      expect(validateSessionId(logger, jwt)).toBe(null);
    });

    it('should reject special characters even if MCP spec allows them', () => {
      const logger = createMockLogger();
      // MCP spec allows these, but we only accept UUIDs for security
      expect(validateSessionId(logger, 'session.id')).toBe(null);
      expect(validateSessionId(logger, 'session/id')).toBe(null);
      expect(validateSessionId(logger, 'session:id')).toBe(null);
      expect(validateSessionId(logger, 'session@id')).toBe(null);
    });
  });

  describe('logging', () => {
    it('should log debug message for invalid session ID', () => {
      const logger = createMockLogger();
      validateSessionId(logger, 'invalid-not-a-uuid');
      expect(logger.debug).toHaveBeenCalledWith('Invalid session ID', expect.any(Object));
    });

    it('should not log for valid UUID session ID', () => {
      const logger = createMockLogger();
      validateSessionId(logger, '550e8400-e29b-41d4-a716-446655440000');
      expect(logger.debug).not.toHaveBeenCalled();
    });

    it('should not log for undefined session ID', () => {
      const logger = createMockLogger();
      validateSessionId(logger, undefined);
      expect(logger.debug).not.toHaveBeenCalled();
    });
  });
});

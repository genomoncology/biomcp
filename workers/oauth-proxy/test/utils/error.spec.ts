/**
 * Error Utility Tests
 *
 * Tests for extractErrorMessage and errorResponse functions.
 */

import { describe, it, expect } from 'vitest';
import { extractErrorMessage } from '../../src/utils/error';

describe('extractErrorMessage', () => {
  it('should extract message from Error instance', () => {
    expect(extractErrorMessage(new Error('test error'))).toBe('test error');
  });

  it('should handle Error with empty message', () => {
    expect(extractErrorMessage(new Error(''))).toBe('');
  });

  it('should convert string to itself', () => {
    expect(extractErrorMessage('string error')).toBe('string error');
  });

  it('should convert number to string', () => {
    expect(extractErrorMessage(123)).toBe('123');
    expect(extractErrorMessage(0)).toBe('0');
    expect(extractErrorMessage(-1)).toBe('-1');
  });

  it('should convert null to string', () => {
    expect(extractErrorMessage(null)).toBe('null');
  });

  it('should convert undefined to string', () => {
    expect(extractErrorMessage(undefined)).toBe('undefined');
  });

  it('should convert object to [object Object]', () => {
    expect(extractErrorMessage({ foo: 'bar' })).toBe('[object Object]');
  });

  it('should convert boolean to string', () => {
    expect(extractErrorMessage(true)).toBe('true');
    expect(extractErrorMessage(false)).toBe('false');
  });
});

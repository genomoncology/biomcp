/**
 * Type-Level Tests
 *
 * Tests for type utilities using compile-time assertions.
 * These tests verify type behavior at compile time, not runtime.
 *
 * How it works:
 * - Valid assignments should compile without error
 * - Invalid assignments use @ts-expect-error to verify they fail at compile time
 * - The runtime tests are just placeholders to make vitest happy
 */

import { describe, it, expect } from 'vitest';
import type { AlwaysStrings, Prettify } from '../src/utils/types';

// ============================================================================
// AlwaysStrings Tests
// ============================================================================

describe('AlwaysStrings', () => {
  it('converts all property types to strings', () => {
    type Input = {
      str: string;
      num: number;
      bool: boolean;
      arr: string[];
    };
    type Result = AlwaysStrings<Input>;

    // Valid: all properties should accept strings
    const valid: Result = {
      str: 'hello',
      num: '42',
      bool: 'true',
      arr: '["a","b"]',
    };
    expect(valid).toBeDefined();
  });

  it('rejects non-string values', () => {
    type Input = { num: number };
    type Result = AlwaysStrings<Input>;

    // @ts-expect-error - number should not be assignable
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    const _invalid: Result = { num: 42 };
  });

  it('preserves optional properties', () => {
    type Input = {
      required: string;
      optional?: number;
    };
    type Result = AlwaysStrings<Input>;

    // Valid: optional property can be omitted
    const withoutOptional: Result = { required: 'hello' };
    expect(withoutOptional).toBeDefined();

    // Valid: optional property can be provided
    const withOptional: Result = { required: 'hello', optional: '42' };
    expect(withOptional).toBeDefined();
  });

  it('handles nested optional with undefined', () => {
    type Input = {
      optional?: string[];
    };
    type Result = AlwaysStrings<Input>;

    // Valid: can be undefined
    const valid: Result = { optional: undefined };
    expect(valid).toBeDefined();
  });

  it('handles empty objects', () => {
    type Input = Record<string, never>;
    type Result = AlwaysStrings<Input>;

    const valid: Result = {};
    expect(valid).toBeDefined();
  });
});

// ============================================================================
// Prettify Tests
// ============================================================================

describe('Prettify', () => {
  it('flattens intersection types', () => {
    type A = { a: number };
    type B = { b: string };
    type Result = Prettify<A & B>;

    // Valid: merged type should have both properties
    const valid: Result = { a: 1, b: 'hello' };
    expect(valid).toBeDefined();
  });

  it('preserves property types', () => {
    type Input = {
      num: number;
      str: string;
      bool: boolean;
    };
    type Result = Prettify<Input>;

    // @ts-expect-error - wrong type for num
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    const _invalid1: Result = { num: 'not a number', str: 'hello', bool: true };

    // @ts-expect-error - wrong type for str
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    const _invalid2: Result = { num: 42, str: 123, bool: true };
  });

  it('preserves optional modifiers', () => {
    type Input = {
      required: string;
      optional?: number;
    };
    type Result = Prettify<Input>;

    // Valid: optional can be omitted
    const valid: Result = { required: 'hello' };
    expect(valid).toBeDefined();
  });

  it('preserves readonly modifiers', () => {
    type Input = {
      readonly immutable: string;
      mutable: string;
    };
    type Result = Prettify<Input>;

    const obj: Result = { immutable: 'cannot change', mutable: 'can change' };

    // @ts-expect-error - readonly property cannot be assigned
    obj.immutable = 'new value';

    // Valid: mutable property can be assigned
    obj.mutable = 'new value';
    expect(obj).toBeDefined();
  });
});

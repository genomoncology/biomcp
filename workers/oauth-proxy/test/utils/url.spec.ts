/**
 * URL Utility Tests
 *
 * Tests for appendPath function.
 */

import { describe, it, expect } from 'vitest';
import { appendPath } from '../../src/utils/url';

describe('appendPath', () => {
  it('appends path to base URL without trailing slash', () => {
    const url = appendPath('https://api.example.com/v1', 'users');
    expect(url.toString()).toBe('https://api.example.com/v1/users');
  });

  it('appends path to base URL with trailing slash', () => {
    const url = appendPath('https://api.example.com/v1/', 'users');
    expect(url.toString()).toBe('https://api.example.com/v1/users');
  });

  it('appends nested path', () => {
    const url = appendPath('https://api.example.com/v1', 'oauth/authenticate');
    expect(url.toString()).toBe('https://api.example.com/v1/oauth/authenticate');
  });

  it('works with base URL without path', () => {
    const url = appendPath('https://api.example.com', 'users');
    expect(url.toString()).toBe('https://api.example.com/users');
  });

  it('returns a URL object', () => {
    const url = appendPath('https://api.example.com/v1', 'users');
    expect(url).toBeInstanceOf(URL);
  });

  it('allows further manipulation of returned URL', () => {
    const url = appendPath('https://api.example.com/v1', 'users');
    url.searchParams.set('page', '1');
    expect(url.toString()).toBe('https://api.example.com/v1/users?page=1');
  });

  // Edge cases - documenting current behavior
  it('strips leading slash from path to ensure appending', () => {
    // Leading slashes are stripped to prevent the URL API footgun
    // where absolute paths replace the base path
    const url = appendPath('https://api.example.com/v1', '/users');
    expect(url.toString()).toBe('https://api.example.com/v1/users');
  });

  it('empty path returns base with trailing slash', () => {
    const url = appendPath('https://api.example.com/v1', '');
    expect(url.toString()).toBe('https://api.example.com/v1/');
  });

  it('preserves query params in path', () => {
    const url = appendPath('https://api.example.com/v1', 'users?active=true');
    expect(url.toString()).toBe('https://api.example.com/v1/users?active=true');
  });
});

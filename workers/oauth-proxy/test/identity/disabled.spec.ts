/**
 * Disabled Identity Provider Tests
 *
 * Tests for DisabledIdentityProvider, which throws on all operations.
 */

import { describe, it, expect } from 'vitest';
import { DisabledIdentityProvider } from '../../src/identity/providers/disabled';

describe('DisabledIdentityProvider', () => {
  const provider = new DisabledIdentityProvider();

  it('has name "disabled"', () => {
    expect(provider.name).toBe('disabled');
  });

  it('throws on buildAuthorizationUrl', async () => {
    await expect(
      provider.buildAuthorizationUrl({
        callbackUrl: 'https://example.com/callback',
        tx: 'test-tx',
      }),
    ).rejects.toThrow('Identity provider is disabled');
  });

  it('throws on parseCallback', () => {
    expect(() => provider.parseCallback(new URL('https://example.com/callback?code=test'))).toThrow('Identity provider is disabled');
  });

  it('throws on authenticate', async () => {
    await expect(
      provider.authenticate({
        token: 'test-token',
        redirectUri: 'https://example.com/callback',
      }),
    ).rejects.toThrow('Identity provider is disabled');
  });
});

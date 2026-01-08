/**
 * Identity Provider Factory Unit Tests
 *
 * Tests for the createIdentityProvider factory function.
 */

import { describe, it, expect } from 'vitest';
import { createIdentityProvider, type IdentityProviderFactoryEnv } from '../../src/identity/factory';
import { StytchProvider } from '../../src/identity/providers/stytch';
import { StandardOAuthProvider } from '../../src/identity/providers/oauth';
import { createLogger } from '../../src/utils/logger';

// Complete Stytch environment
const stytchEnv: IdentityProviderFactoryEnv = {
  IDENTITY_PROVIDER: 'stytch',
  STYTCH_PROJECT_ID: 'project-test-123',
  STYTCH_SECRET: 'secret-test-456',
  STYTCH_PUBLIC_TOKEN: 'public-token-test-789',
  STYTCH_OAUTH_URL: 'https://test.stytch.com/v1/public/oauth/google/start',
  STYTCH_API_URL: 'https://test.stytch.com/v1',
};

// Complete OAuth environment
const oauthEnv: IdentityProviderFactoryEnv = {
  IDENTITY_PROVIDER: 'oauth',
  OAUTH_CLIENT_ID: 'test-client-id',
  OAUTH_CLIENT_SECRET: 'test-client-secret',
  OAUTH_AUTHORIZATION_URL: 'https://idp.example.com/authorize',
  OAUTH_TOKEN_URL: 'https://idp.example.com/oauth/token',
  OAUTH_USERINFO_URL: 'https://idp.example.com/userinfo',
};

describe('createIdentityProvider', () => {
  const logger = createLogger(false, 'test');

  describe('Stytch provider', () => {
    it('creates StytchProvider when IDENTITY_PROVIDER is "stytch"', () => {
      const provider = createIdentityProvider(stytchEnv, logger);

      expect(provider).toBeInstanceOf(StytchProvider);
      expect(provider.name).toBe('stytch');
    });

    it('throws when STYTCH_PROJECT_ID is missing', () => {
      const env: IdentityProviderFactoryEnv = {
        IDENTITY_PROVIDER: 'stytch',
        STYTCH_SECRET: 'secret',
        STYTCH_PUBLIC_TOKEN: 'token',
        STYTCH_OAUTH_URL: 'https://test.stytch.com/v1/public/oauth/google/start',
        STYTCH_API_URL: 'https://test.stytch.com/v1',
      };

      expect(() => createIdentityProvider(env, logger)).toThrow('Missing required Stytch env vars');
      expect(() => createIdentityProvider(env, logger)).toThrow('STYTCH_PROJECT_ID');
    });

    it('throws when STYTCH_SECRET is missing', () => {
      const env: IdentityProviderFactoryEnv = {
        IDENTITY_PROVIDER: 'stytch',
        STYTCH_PROJECT_ID: 'project',
        STYTCH_PUBLIC_TOKEN: 'token',
        STYTCH_OAUTH_URL: 'https://test.stytch.com/v1/public/oauth/google/start',
        STYTCH_API_URL: 'https://test.stytch.com/v1',
      };

      expect(() => createIdentityProvider(env, logger)).toThrow('Missing required Stytch env vars');
      expect(() => createIdentityProvider(env, logger)).toThrow('STYTCH_SECRET');
    });

    it('throws when STYTCH_PUBLIC_TOKEN is missing', () => {
      const env: IdentityProviderFactoryEnv = {
        IDENTITY_PROVIDER: 'stytch',
        STYTCH_PROJECT_ID: 'project',
        STYTCH_SECRET: 'secret',
        STYTCH_OAUTH_URL: 'https://test.stytch.com/v1/public/oauth/google/start',
        STYTCH_API_URL: 'https://test.stytch.com/v1',
      };

      expect(() => createIdentityProvider(env, logger)).toThrow('Missing required Stytch env vars');
      expect(() => createIdentityProvider(env, logger)).toThrow('STYTCH_PUBLIC_TOKEN');
    });

    it('throws when STYTCH_OAUTH_URL is missing', () => {
      const env: IdentityProviderFactoryEnv = {
        IDENTITY_PROVIDER: 'stytch',
        STYTCH_PROJECT_ID: 'project',
        STYTCH_SECRET: 'secret',
        STYTCH_PUBLIC_TOKEN: 'token',
        STYTCH_API_URL: 'https://test.stytch.com/v1',
      };

      expect(() => createIdentityProvider(env, logger)).toThrow('Missing required Stytch env vars');
      expect(() => createIdentityProvider(env, logger)).toThrow('STYTCH_OAUTH_URL');
    });

    it('throws when STYTCH_API_URL is missing', () => {
      const env: IdentityProviderFactoryEnv = {
        IDENTITY_PROVIDER: 'stytch',
        STYTCH_PROJECT_ID: 'project',
        STYTCH_SECRET: 'secret',
        STYTCH_PUBLIC_TOKEN: 'token',
        STYTCH_OAUTH_URL: 'https://test.stytch.com/v1/public/oauth/google/start',
      };

      expect(() => createIdentityProvider(env, logger)).toThrow('Missing required Stytch env vars');
      expect(() => createIdentityProvider(env, logger)).toThrow('STYTCH_API_URL');
    });

    describe('HTTPS enforcement', () => {
      it('throws when STYTCH_OAUTH_URL is HTTP', () => {
        const env: IdentityProviderFactoryEnv = {
          ...stytchEnv,
          STYTCH_OAUTH_URL: 'http://insecure.stytch.com/v1/public/oauth/google/start',
        };

        expect(() => createIdentityProvider(env, logger)).toThrow();
      });

      it('throws when STYTCH_API_URL is HTTP', () => {
        const env: IdentityProviderFactoryEnv = {
          ...stytchEnv,
          STYTCH_API_URL: 'http://insecure.stytch.com/v1',
        };

        expect(() => createIdentityProvider(env, logger)).toThrow();
      });

      it('allows localhost HTTP for STYTCH_OAUTH_URL (local dev)', () => {
        const env: IdentityProviderFactoryEnv = {
          ...stytchEnv,
          STYTCH_OAUTH_URL: 'http://localhost:3000/oauth/start',
        };

        const provider = createIdentityProvider(env, logger);
        expect(provider).toBeInstanceOf(StytchProvider);
      });

      it('allows localhost HTTP for STYTCH_API_URL (local dev)', () => {
        const env: IdentityProviderFactoryEnv = {
          ...stytchEnv,
          STYTCH_API_URL: 'http://localhost:3000/api',
        };

        const provider = createIdentityProvider(env, logger);
        expect(provider).toBeInstanceOf(StytchProvider);
      });

      it('allows 127.0.0.1 HTTP for Stytch URLs (local dev)', () => {
        const env: IdentityProviderFactoryEnv = {
          ...stytchEnv,
          STYTCH_OAUTH_URL: 'http://127.0.0.1:3000/oauth/start',
          STYTCH_API_URL: 'http://127.0.0.1:3000/api',
        };

        const provider = createIdentityProvider(env, logger);
        expect(provider).toBeInstanceOf(StytchProvider);
      });
    });
  });

  describe('OAuth provider', () => {
    it('creates StandardOAuthProvider when IDENTITY_PROVIDER is "oauth"', () => {
      const provider = createIdentityProvider(oauthEnv, logger);

      expect(provider).toBeInstanceOf(StandardOAuthProvider);
      expect(provider.name).toBe('oauth');
    });

    it('throws when OAUTH_CLIENT_ID is missing', () => {
      const env: IdentityProviderFactoryEnv = {
        IDENTITY_PROVIDER: 'oauth',
        OAUTH_CLIENT_SECRET: 'secret',
        OAUTH_AUTHORIZATION_URL: 'https://idp.example.com/authorize',
        OAUTH_TOKEN_URL: 'https://idp.example.com/oauth/token',
      };

      expect(() => createIdentityProvider(env, logger)).toThrow('Missing required OAuth env vars');
      expect(() => createIdentityProvider(env, logger)).toThrow('OAUTH_CLIENT_ID');
    });

    it('creates provider without OAUTH_USERINFO_URL (optional)', () => {
      const env: IdentityProviderFactoryEnv = {
        IDENTITY_PROVIDER: 'oauth',
        OAUTH_CLIENT_ID: 'client-id',
        OAUTH_CLIENT_SECRET: 'secret',
        OAUTH_AUTHORIZATION_URL: 'https://idp.example.com/authorize',
        OAUTH_TOKEN_URL: 'https://idp.example.com/oauth/token',
      };

      const provider = createIdentityProvider(env, logger);
      expect(provider.name).toBe('oauth');
    });

    describe('HTTPS enforcement', () => {
      it('throws when OAUTH_AUTHORIZATION_URL is HTTP', () => {
        const env: IdentityProviderFactoryEnv = {
          ...oauthEnv,
          OAUTH_AUTHORIZATION_URL: 'http://insecure.idp.example.com/authorize',
        };

        expect(() => createIdentityProvider(env, logger)).toThrow();
      });

      it('throws when OAUTH_TOKEN_URL is HTTP', () => {
        const env: IdentityProviderFactoryEnv = {
          ...oauthEnv,
          OAUTH_TOKEN_URL: 'http://insecure.idp.example.com/token',
        };

        expect(() => createIdentityProvider(env, logger)).toThrow();
      });

      it('throws when OAUTH_USERINFO_URL is HTTP', () => {
        const env: IdentityProviderFactoryEnv = {
          ...oauthEnv,
          OAUTH_USERINFO_URL: 'http://insecure.idp.example.com/userinfo',
        };

        expect(() => createIdentityProvider(env, logger)).toThrow();
      });

      it('throws when OAUTH_ISSUER is HTTP', () => {
        const env: IdentityProviderFactoryEnv = {
          IDENTITY_PROVIDER: 'oauth',
          OAUTH_CLIENT_ID: 'client-id',
          OAUTH_CLIENT_SECRET: 'secret',
          OAUTH_ISSUER: 'http://insecure.idp.example.com',
        };

        expect(() => createIdentityProvider(env, logger)).toThrow();
      });

      it('allows localhost HTTP for OAuth URLs (local dev)', () => {
        const env: IdentityProviderFactoryEnv = {
          IDENTITY_PROVIDER: 'oauth',
          OAUTH_CLIENT_ID: 'client-id',
          OAUTH_CLIENT_SECRET: 'secret',
          OAUTH_AUTHORIZATION_URL: 'http://localhost:3000/authorize',
          OAUTH_TOKEN_URL: 'http://localhost:3000/token',
          OAUTH_USERINFO_URL: 'http://localhost:3000/userinfo',
        };

        const provider = createIdentityProvider(env, logger);
        expect(provider).toBeInstanceOf(StandardOAuthProvider);
      });

      it('allows 127.0.0.1 HTTP for OAuth URLs (local dev)', () => {
        const env: IdentityProviderFactoryEnv = {
          IDENTITY_PROVIDER: 'oauth',
          OAUTH_CLIENT_ID: 'client-id',
          OAUTH_CLIENT_SECRET: 'secret',
          OAUTH_AUTHORIZATION_URL: 'http://127.0.0.1:3000/authorize',
          OAUTH_TOKEN_URL: 'http://127.0.0.1:3000/token',
        };

        const provider = createIdentityProvider(env, logger);
        expect(provider).toBeInstanceOf(StandardOAuthProvider);
      });

      it('allows localhost HTTP for OAUTH_ISSUER (local dev)', () => {
        const env: IdentityProviderFactoryEnv = {
          IDENTITY_PROVIDER: 'oauth',
          OAUTH_CLIENT_ID: 'client-id',
          OAUTH_CLIENT_SECRET: 'secret',
          OAUTH_ISSUER: 'http://localhost:3000',
        };

        const provider = createIdentityProvider(env, logger);
        expect(provider).toBeInstanceOf(StandardOAuthProvider);
      });
    });
  });

  describe('disabled provider', () => {
    it('creates DisabledIdentityProvider when IDENTITY_PROVIDER is "disabled"', () => {
      const env: IdentityProviderFactoryEnv = {
        IDENTITY_PROVIDER: 'disabled',
      };

      const provider = createIdentityProvider(env, logger);

      expect(provider.name).toBe('disabled');
    });

    it('does not require any additional env vars', () => {
      const env: IdentityProviderFactoryEnv = {
        IDENTITY_PROVIDER: 'disabled',
      };

      // Should not throw - no validation needed
      expect(() => createIdentityProvider(env, logger)).not.toThrow();
    });
  });

  describe('unknown provider', () => {
    it('throws for unknown provider type', () => {
      const env: IdentityProviderFactoryEnv = {
        // @ts-expect-error - Testing invalid provider type
        IDENTITY_PROVIDER: 'unknown-provider',
      };

      expect(() => createIdentityProvider(env, logger)).toThrow('Unknown identity provider: unknown-provider');
    });
  });
});

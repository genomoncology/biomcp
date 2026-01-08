/**
 * JWT Validator Unit Tests
 *
 * Tests for JWT validation including audience claim validation.
 */

import { env } from 'cloudflare:test';
import { describe, it, expect } from 'vitest';
import { SignJWT } from 'jose';
import { JwtValidator } from '../../src/auth/jwt';
import { sha256Base64Url } from '../../src/utils/crypto';
import { createAccessTokenStorage } from '../../src/storage';
import { createLogger } from '../../src/utils/logger';

// Test constant - must match vitest.config.mts
const TEST_JWT_SECRET = 'test-jwt-secret-for-ci-minimum-32-chars-long';

const TEST_ISSUER = 'https://example.com';
const TEST_AUDIENCE = 'https://example.com';

/**
 * Generate a JWT token with configurable claims for testing.
 */
async function generateToken(options: {
  sub: string;
  issuer?: string;
  audience?: string | null; // null means omit the claim entirely
  expiresIn?: number;
}): Promise<string> {
  const encoder = new TextEncoder();
  const secret = encoder.encode(TEST_JWT_SECRET);

  const tokenIssuedAt = Math.floor(Date.now() / 1000);
  const tokenExpiration = tokenIssuedAt + (options.expiresIn ?? 3600);

  const jwt = new SignJWT({
    email: 'test@example.com',
    client_id: 'test-client',
    scope: 'mcp',
  })
    .setProtectedHeader({ alg: 'HS256' })
    .setSubject(options.sub)
    .setIssuedAt(tokenIssuedAt)
    .setExpirationTime(tokenExpiration);

  if (options.issuer !== undefined) {
    jwt.setIssuer(options.issuer);
  }

  // Only set audience if not explicitly null
  if (options.audience !== null) {
    jwt.setAudience(options.audience ?? TEST_AUDIENCE);
  }

  return jwt.sign(secret);
}

/**
 * Store a token in KV storage (required for validation).
 */
async function storeToken(kv: KVNamespace, token: string, resource: string = TEST_AUDIENCE): Promise<void> {
  const tokenKey = await sha256Base64Url(token);
  const accessTokenStorage = createAccessTokenStorage(kv);
  await accessTokenStorage.put(tokenKey, {
    sub: 'test-user-id',
    email: 'test@example.com',
    client_id: 'test-client',
    scope: 'mcp',
    resource: [resource], // RFC 8707: resource indicator
  });
}

function createValidator(): JwtValidator {
  const logger = createLogger(true, 'test');
  return new JwtValidator(logger, {
    jwtSecret: TEST_JWT_SECRET,
    oauthKv: env.OAUTH_KV,
    validIssuers: [TEST_ISSUER],
  });
}

describe('JwtValidator', () => {
  describe('audience validation (RFC 8707)', () => {
    it('should accept tokens with correct audience claim', async () => {
      const validator = createValidator();

      const token = await generateToken({
        sub: 'test-user',
        issuer: TEST_ISSUER,
        audience: TEST_AUDIENCE,
      });

      await storeToken(env.OAUTH_KV, token);

      const result = await validator.validate(token, { expectedResource: TEST_AUDIENCE });

      expect(result.payload.sub).toBe('test-user');
      expect(result.payload.aud).toBe(TEST_AUDIENCE);
    });

    it('should reject tokens with incorrect audience claim', async () => {
      const validator = createValidator();

      const token = await generateToken({
        sub: 'test-user',
        issuer: TEST_ISSUER,
        audience: 'https://wrong-audience.com',
      });

      await storeToken(env.OAUTH_KV, token, 'https://wrong-audience.com');

      await expect(validator.validate(token, { expectedResource: TEST_AUDIENCE })).rejects.toThrow(/unexpected "aud" claim value/);
    });

    it('should reject tokens with missing audience claim', async () => {
      const validator = createValidator();

      const token = await generateToken({
        sub: 'test-user',
        issuer: TEST_ISSUER,
        audience: null, // Explicitly omit audience
      });

      await storeToken(env.OAUTH_KV, token);

      await expect(validator.validate(token, { expectedResource: TEST_AUDIENCE })).rejects.toThrow(/"aud" claim/);
    });
  });

  describe('issuer validation', () => {
    it('should accept tokens with correct issuer claim', async () => {
      const validator = createValidator();

      const token = await generateToken({
        sub: 'test-user',
        issuer: TEST_ISSUER,
        audience: TEST_AUDIENCE,
      });

      await storeToken(env.OAUTH_KV, token);

      const result = await validator.validate(token, { expectedResource: TEST_AUDIENCE });

      expect(result.payload.iss).toBe(TEST_ISSUER);
    });

    it('should reject tokens with incorrect issuer claim', async () => {
      const validator = createValidator();

      const token = await generateToken({
        sub: 'test-user',
        issuer: 'https://wrong-issuer.com',
        audience: TEST_AUDIENCE,
      });

      await storeToken(env.OAUTH_KV, token);

      await expect(validator.validate(token, { expectedResource: TEST_AUDIENCE })).rejects.toThrow(/unexpected "iss" claim value/);
    });
  });

  describe('token storage validation', () => {
    it('should reject tokens not in KV storage', async () => {
      const validator = createValidator();

      const token = await generateToken({
        sub: 'test-user',
        issuer: TEST_ISSUER,
        audience: TEST_AUDIENCE,
      });

      // Deliberately not storing the token

      await expect(validator.validate(token, { expectedResource: TEST_AUDIENCE })).rejects.toThrow(/Token not found or revoked/);
    });
  });

  describe('basic validation', () => {
    it('should reject empty tokens', async () => {
      const validator = createValidator();

      await expect(validator.validate('', { expectedResource: TEST_AUDIENCE })).rejects.toThrow(/No token provided/);
    });
  });
});

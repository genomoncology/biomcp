import { jwtVerify, type JWTVerifyResult } from 'jose';
import { createAccessTokenStorage } from '../storage';
import { sha256Base64Url } from '../utils/crypto';
import type { Logger } from '../utils/logger';

interface JwtConfiguration {
  jwtSecret: string;
  oauthKv: KVNamespace;
  validIssuers: string[];
}

export interface ValidateOptions {
  /** RFC 8707: The resource URI that the token must be authorized for */
  expectedResource: string;
}

export class JwtValidator {
  constructor(
    private logger: Logger,
    private config: JwtConfiguration,
  ) {}

  async validate(token: string, options: ValidateOptions): Promise<JWTVerifyResult> {
    if (!token) {
      throw new Error('No token provided');
    }

    this.logger.debug('Validating token', { tokenPreview: token.substring(0, 15), expectedResource: options.expectedResource });

    const encoder = new TextEncoder();
    const secret = encoder.encode(this.config.jwtSecret);

    // RFC 8707: Validate that the token's audience includes the expected resource
    const result = await jwtVerify(token, secret, {
      issuer: this.config.validIssuers,
      audience: options.expectedResource,
    });

    // Check revocation in KV - token must exist in storage to be valid
    const tokenHash = await this.hashToken(token);
    const tokenStorage = createAccessTokenStorage(this.config.oauthKv);
    const storedTokenData = await tokenStorage.get(tokenHash);
    if (!storedTokenData) {
      this.logger.debug('Token not found in storage - may have been revoked');
      throw new Error('Token not found or revoked');
    }

    this.logger.debug('Self-issued JWT validation successful', { sub: result.payload.sub });
    return result;
  }

  private async hashToken(token: string): Promise<string> {
    return sha256Base64Url(token);
  }
}

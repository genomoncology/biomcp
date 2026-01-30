import { importPKCS8, SignJWT } from 'jose';
import type { AnalyticsEvent, AnalyticsService } from './interface';
import { extractErrorMessage } from '../utils/error';
import type { Logger } from '../utils/logger';
import { BigQueryEnvironment } from '../env';

// TODO: type the bigquery response
export type BigQueryInsertResult = any;

/**
 * Module-level token cache for BigQuery OAuth tokens.
 *
 * Keyed by service account email to support multiple BigQuery configurations.
 * Persists across requests within the same Worker isolate (~99.99% warm starts).
 *
 * This is a valuable optimization because:
 * - Analytics events are logged on every request (high frequency)
 * - Token generation is expensive (~200-400ms: JSON parse, PKCS8 import, JWT sign, Google token exchange)
 * - Tokens are valid for 1 hour (high cache hit rate)
 *
 * See docs/workers-caching.md for guidance on Workers caching patterns.
 */
const tokenCache = new Map<string, { token: string; expiresAt: number }>();

/**
 * Clear the module-level token cache. For testing only.
 */
export function clearBigQueryTokenCache(): void {
  tokenCache.clear();
}

export class BigQueryAnalytics implements AnalyticsService {
  constructor(
    private env: BigQueryEnvironment,
    private logger: Logger,
  ) {}

  /**
   * Insert a single row into BigQuery via streaming insert.
   */
  async logEvent(event: AnalyticsEvent): Promise<void> {
    try {
      const token = await this.getToken();

      const url = new URL(
        `/bigquery/v2/projects/${this.env.BQ_PROJECT_ID}/datasets/${this.env.BQ_DATASET}/tables/${this.env.BQ_TABLE}/insertAll`,
        'https://bigquery.googleapis.com',
      );

      const response = await fetch(url, {
        method: 'POST',
        headers: {
          Authorization: `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          rows: [{ json: { ...event, timestamp: new Date().toISOString() } }],
        }),
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(`BigQuery API error: ${response.status} - ${errorText}`);
      }

      const result = await response.json<BigQueryInsertResult>();
      if (result.insertErrors) {
        throw new Error(`BigQuery insert errors: ${JSON.stringify(result.insertErrors)}`);
      }
    } catch (error) {
      this.logger.error('Insert failed', { error: extractErrorMessage(error) });
      throw error;
    }
  }

  /**
   * Fetch (and cache) a BigQuery OAuth token.
   *
   * Uses module-level cache keyed by service account email.
   * Tokens are cached for 1 hour with a 60s buffer before expiry.
   */
  private async getToken(): Promise<string> {
    const now = Math.floor(Date.now() / 1000);

    // Parse the service‐account JSON key (needed for cache key and token generation)
    const key = JSON.parse(this.env.BQ_SA_KEY_JSON);
    const cacheKey = key.client_email;

    // Return cached token if valid (with 60s buffer)
    const cached = tokenCache.get(cacheKey);
    if (cached && cached.expiresAt > now + 60) {
      return cached.token;
    }

    // Convert PEM private key string into a CryptoKey
    const privateKey = await importPKCS8(key.private_key, 'RS256');

    // Build the JWT assertion
    const assertion = await new SignJWT({
      iss: key.client_email,
      scope: 'https://www.googleapis.com/auth/bigquery.insertdata',
      aud: 'https://oauth2.googleapis.com/token',
      iat: now,
      exp: now + 3600,
    })
      .setProtectedHeader({ alg: 'RS256', kid: key.private_key_id })
      .sign(privateKey);

    // Exchange the assertion for an access token
    const resp = await fetch('https://oauth2.googleapis.com/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: new URLSearchParams({
        grant_type: 'urn:ietf:params:oauth:grant-type:jwt-bearer',
        assertion,
      }),
    });

    if (!resp.ok) {
      this.logger.warn('BigQuery token exchange failed', { status: resp.status });
      throw new Error(`BigQuery token exchange failed: ${resp.status}`);
    }

    const { access_token } = await resp.json<{
      access_token: string;
    }>();

    // Cache the token at module level (persists across requests)
    tokenCache.set(cacheKey, { token: access_token, expiresAt: now + 3600 });
    return access_token;
  }
}

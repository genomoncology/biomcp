/**
 * Analytics Factory Tests
 *
 * Tests for createAnalytics function covering provider selection and
 * validation fallbacks.
 */

import { describe, expect, it, vi } from 'vitest';
import { createAnalytics } from '../src/analytics/factory';
import { BigQueryAnalytics } from '../src/analytics/bigquery';
import { CloudflareAnalytics } from '../src/analytics/cloudflare';
import { NoopAnalytics } from '../src/analytics/noop';
import type { Logger } from '../src/utils/logger';

function createMockLogger(): Logger {
  const logger: Logger = {
    debug: vi.fn(),
    info: vi.fn(),
    warn: vi.fn(),
    error: vi.fn(),
    event: vi.fn(),
    child: vi.fn(() => logger),
  };
  return logger;
}

describe('createAnalytics', () => {
  describe('cloudflare provider', () => {
    it('creates CloudflareAnalytics', () => {
      const logger = createMockLogger();

      const analytics = createAnalytics(
        {
          ANALYTICS_PROVIDER: 'cloudflare',
        },
        logger,
      );

      expect(analytics).toBeInstanceOf(CloudflareAnalytics);
    });
  });

  describe('bigquery provider', () => {
    it('creates BigQueryAnalytics when all env vars are valid', () => {
      const logger = createMockLogger();

      const analytics = createAnalytics(
        {
          ANALYTICS_PROVIDER: 'bigquery',
          BQ_SA_KEY_JSON: '{"valid": "json"}',
          BQ_PROJECT_ID: 'my-project',
          BQ_DATASET: 'my-dataset',
          BQ_TABLE: 'my-table',
        },
        logger,
      );

      expect(analytics).toBeInstanceOf(BigQueryAnalytics);
    });

    it('falls back to NoopAnalytics when BQ_SA_KEY_JSON is missing', () => {
      const logger = createMockLogger();

      const analytics = createAnalytics(
        {
          ANALYTICS_PROVIDER: 'bigquery',
          BQ_PROJECT_ID: 'my-project',
          BQ_DATASET: 'my-dataset',
          BQ_TABLE: 'my-table',
        },
        logger,
      );

      expect(analytics).toBeInstanceOf(NoopAnalytics);
    });

    it('falls back to NoopAnalytics when BQ_PROJECT_ID is missing', () => {
      const logger = createMockLogger();

      const analytics = createAnalytics(
        {
          ANALYTICS_PROVIDER: 'bigquery',
          BQ_SA_KEY_JSON: '{"valid": "json"}',
          BQ_DATASET: 'my-dataset',
          BQ_TABLE: 'my-table',
        },
        logger,
      );

      expect(analytics).toBeInstanceOf(NoopAnalytics);
    });

    it('falls back to NoopAnalytics when BQ_DATASET is missing', () => {
      const logger = createMockLogger();

      const analytics = createAnalytics(
        {
          ANALYTICS_PROVIDER: 'bigquery',
          BQ_SA_KEY_JSON: '{"valid": "json"}',
          BQ_PROJECT_ID: 'my-project',
          BQ_TABLE: 'my-table',
        },
        logger,
      );

      expect(analytics).toBeInstanceOf(NoopAnalytics);
    });

    it('falls back to NoopAnalytics when BQ_TABLE is missing', () => {
      const logger = createMockLogger();

      const analytics = createAnalytics(
        {
          ANALYTICS_PROVIDER: 'bigquery',
          BQ_SA_KEY_JSON: '{"valid": "json"}',
          BQ_PROJECT_ID: 'my-project',
          BQ_DATASET: 'my-dataset',
        },
        logger,
      );

      expect(analytics).toBeInstanceOf(NoopAnalytics);
    });

    it('falls back to NoopAnalytics when all BigQuery env vars are missing', () => {
      const logger = createMockLogger();

      const analytics = createAnalytics(
        {
          ANALYTICS_PROVIDER: 'bigquery',
        },
        logger,
      );

      expect(analytics).toBeInstanceOf(NoopAnalytics);
    });
  });

  describe('none provider', () => {
    it('creates NoopAnalytics', () => {
      const logger = createMockLogger();

      const analytics = createAnalytics(
        {
          ANALYTICS_PROVIDER: 'none',
        },
        logger,
      );

      expect(analytics).toBeInstanceOf(NoopAnalytics);
    });
  });
});

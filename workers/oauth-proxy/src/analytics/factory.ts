import * as v from 'valibot';
import { AnalyticsProviderType, BigQueryEnvironment, BigQueryEnvironmentSchema } from '../env';
import type { Logger } from '../utils/logger';
import { BigQueryAnalytics } from './bigquery';
import { CloudflareAnalytics } from './cloudflare';
import type { AnalyticsService } from './interface';
import { NoopAnalytics } from './noop';

type AnalyticsEnv = {
  ANALYTICS_PROVIDER: AnalyticsProviderType;
} & Partial<BigQueryEnvironment>;

export function createAnalytics(env: AnalyticsEnv, logger: Logger): AnalyticsService {
  const provider = env.ANALYTICS_PROVIDER;

  switch (provider) {
    case 'cloudflare':
      return new CloudflareAnalytics(logger.child('analytics'));

    case 'bigquery': {
      const parseResult = v.safeParse(BigQueryEnvironmentSchema, env);
      if (!parseResult.success) {
        logger.warn('BigQuery env vars missing or invalid, falling back to noop');
        return new NoopAnalytics(logger);
      }
      return new BigQueryAnalytics(parseResult.output, logger.child('bigquery'));
    }

    case 'none':
    default:
      return new NoopAnalytics(logger);
  }
}

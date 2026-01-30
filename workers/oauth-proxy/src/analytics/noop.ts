import type { AnalyticsEvent, AnalyticsService } from './interface';
import type { Logger } from '../utils/logger';

export class NoopAnalytics implements AnalyticsService {
  constructor(private logger: Logger) {}

  async logEvent(event: AnalyticsEvent): Promise<void> {
    this.logger.debug('Event logged (noop)', { method: event.method, userSub: event.userSub });
  }
}

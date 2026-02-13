import type { AnalyticsEvent, AnalyticsService } from './interface';
import type { Logger } from '../utils/logger';

export class CloudflareAnalytics implements AnalyticsService {
  constructor(private logger: Logger) {}

  async logEvent(event: AnalyticsEvent): Promise<void> {
    this.logger.event('mcp_request', {
      method: event.method,
      toolName: event.toolName,
      requestId: event.requestId,
      sessionId: event.sessionId,
      userSub: event.userSub,
    });
  }
}

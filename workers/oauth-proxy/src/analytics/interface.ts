export interface AnalyticsEvent {
  method: string;
  toolName?: string;
  requestId?: string | number | null;
  sessionId: string;
  userSub: string;
}

export interface AnalyticsService {
  logEvent(event: AnalyticsEvent): Promise<void>;
}

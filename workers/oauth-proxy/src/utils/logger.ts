import { sanitizeObject } from './sanitize';

type LogLevel = 'debug' | 'info' | 'warn' | 'error';

const methodMap: Record<LogLevel, (...args: unknown[]) => void> = {
  debug: console.log,
  info: console.log,
  warn: console.warn,
  error: console.error,
};

export interface Logger {
  debug(message: string, context?: Record<string, unknown>): void;
  info(message: string, context?: Record<string, unknown>): void;
  warn(message: string, context?: Record<string, unknown>): void;
  error(message: string, context?: Record<string, unknown>): void;
  /** Log an analytics event. Always logs regardless of debug setting. */
  event(eventType: string, data: Record<string, unknown>): void;
  child(submodule: string): Logger;
}

class LoggerImpl implements Logger {
  constructor(
    private _debug: boolean,
    private module: string,
  ) {}

  private log(level: LogLevel, message: string, context?: Record<string, unknown>): void {
    if (level === 'debug' && !this._debug) return;

    // Sanitize context to prevent accidental logging of sensitive fields
    const sanitizedContext = context ? (sanitizeObject(context) as Record<string, unknown>) : undefined;

    const entry = {
      level,
      module: this.module,
      message,
      timestamp: new Date().toISOString(),
      ...sanitizedContext,
    };

    const method = methodMap[level];

    method(entry);
  }

  debug(message: string, context?: Record<string, unknown>): void {
    this.log('debug', message, context);
  }

  info(message: string, context?: Record<string, unknown>): void {
    this.log('info', message, context);
  }

  warn(message: string, context?: Record<string, unknown>): void {
    this.log('warn', message, context);
  }

  error(message: string, context?: Record<string, unknown>): void {
    this.log('error', message, context);
  }

  event(eventType: string, data: Record<string, unknown>): void {
    // Sanitize data to prevent accidental logging of sensitive fields
    const sanitizedData = sanitizeObject(data) as Record<string, unknown>;

    const entry = {
      level: 'event',
      module: this.module,
      eventType,
      timestamp: new Date().toISOString(),
      ...sanitizedData,
    };
    console.log(entry);
  }

  child(submodule: string): Logger {
    return new LoggerImpl(this._debug, `${this.module}.${submodule}`);
  }
}

export function createLogger(debug: boolean, module: string): Logger {
  return new LoggerImpl(debug, module);
}

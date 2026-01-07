import * as v from 'valibot';
import type { Logger } from './logger';

/**
 * Schema for valid session IDs.
 *
 * We use UUID format because:
 * - We generate all session IDs ourselves using crypto.randomUUID()
 * - Clients only send back the session ID we gave them
 * - UUID validation is stricter and avoids security concerns with special characters
 *   (logging injection, URL encoding issues, etc.)
 *
 * MCP spec reference (for future consideration):
 * The MCP protocol allows visible ASCII characters (0x21-0x7E), max 128 chars.
 * If we ever need to accept session IDs from MCP clients that generate their own,
 * we'd need to broaden this to: v.pipe(v.string(), v.maxLength(128), v.regex(/^[\x21-\x7E]+$/))
 * However, that would require careful consideration of the security implications.
 */
const SessionIdSchema = v.pipe(v.string(), v.uuid());

/**
 * Validate and sanitize session ID
 * @param logger - Logger instance
 * @param sessionId - Session ID from query parameter
 * @returns Validated session ID or null if invalid
 */
export const validateSessionId = (logger: Logger, sessionId: string | undefined): string | null => {
  if (!sessionId) return null;

  const result = v.safeParse(SessionIdSchema, sessionId);
  if (!result.success) {
    logger.debug('Invalid session ID', { error: result.issues[0].message });
    return null;
  }

  return result.output;
};

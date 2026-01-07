// List of sensitive fields that should be redacted in logs
export const SENSITIVE_FIELDS = ['api_key', 'apiKey', 'api-key', 'token', 'secret', 'password', 'code'];

export const TOKEN_EXPIRY_SECONDS = 3600; // 1 hour
export const REFRESH_TOKEN_EXPIRY_SECONDS = 30 * 24 * 60 * 60; // 30 days

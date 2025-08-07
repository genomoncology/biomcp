# FDA Integration Security Documentation

## Overview

This document outlines the security measures implemented in the BioMCP FDA integration to ensure safe handling of medical data and protection against common vulnerabilities.

## Security Features

### 1. Input Validation & Sanitization

All user inputs are validated and sanitized before being sent to the FDA API:

- **Injection Prevention**: Removes characters that could be used for SQL injection, XSS, or command injection (`<>\"';&|\\`)
- **Length Limits**: Enforces maximum lengths on all input fields
- **Type Validation**: Ensures parameters match expected types (dates, numbers, etc.)
- **Format Validation**: Validates specific formats (e.g., YYYY-MM-DD for dates)

**Implementation**: `src/biomcp/openfda/input_validation.py`

```python
# Example usage
from biomcp.openfda.input_validation import sanitize_input, validate_drug_name

safe_drug = validate_drug_name("Aspirin<script>")  # Returns "Aspirin"
safe_input = sanitize_input("'; DROP TABLE;")  # SQL injection blocked
```

### 2. API Key Protection

API keys are protected at multiple levels:

- **Cache Key Exclusion**: API keys are removed before generating cache keys
- **No Logging**: API keys are never logged, even in debug mode
- **Environment Variables**: Keys stored in environment variables, not in code
- **Validation**: API key format is validated before use

**Implementation**: `src/biomcp/openfda/cache.py`, `src/biomcp/openfda/utils.py`

### 3. Rate Limiting

Client-side rate limiting prevents API quota exhaustion:

- **Token Bucket Algorithm**: Allows bursts while maintaining average rate
- **Configurable Limits**: 40 requests/minute without key, 240 with key
- **Concurrent Request Limiting**: Maximum 10 concurrent requests via semaphore
- **Automatic Backoff**: Delays requests when approaching limits

**Implementation**: `src/biomcp/openfda/rate_limiter.py`

### 4. Circuit Breaker Pattern

Prevents cascading failures when FDA API is unavailable:

- **Failure Threshold**: Opens after 5 consecutive failures
- **Recovery Timeout**: Waits 60 seconds before retry attempts
- **Half-Open State**: Tests recovery with limited requests
- **Automatic Recovery**: Returns to normal operation when API recovers

**States**:

- **CLOSED**: Normal operation
- **OPEN**: Blocking all requests (API is down)
- **HALF_OPEN**: Testing if API has recovered

### 5. Memory Protection

Prevents memory exhaustion from large responses:

- **Response Size Limits**: Maximum 1MB per cached response
- **Cache Size Limits**: Maximum 100 entries in cache
- **FIFO Eviction**: Oldest entries removed when cache is full
- **Size Validation**: Large responses rejected before caching

**Configuration**:

```bash
export BIOMCP_FDA_MAX_RESPONSE_SIZE=1048576  # 1MB
export BIOMCP_FDA_MAX_CACHE_SIZE=100
```

### 6. File Operation Security

Secure handling of cache files:

- **File Locking**: Uses `fcntl` for exclusive/shared locks
- **Atomic Operations**: Writes to temp files then renames
- **Race Condition Prevention**: Locks prevent concurrent modifications
- **Permission Control**: Files created without world-write permissions

**Implementation**: `src/biomcp/openfda/drug_shortages.py`

## Security Best Practices

### For Developers

1. **Never Log Sensitive Data**

   ```python
   # BAD
   logger.debug(f"API key: {api_key}")

   # GOOD
   logger.debug("API key configured" if api_key else "No API key")
   ```

2. **Always Validate Input**

   ```python
   from biomcp.openfda.input_validation import validate_drug_name

   # Always validate before using
   safe_drug = validate_drug_name(user_input)
   if safe_drug:
       # Use safe_drug, not user_input
       await search_adverse_events(drug=safe_drug)
   ```

3. **Use Rate Limiting**

   ```python
   from biomcp.openfda.rate_limiter import rate_limited_request

   # Wrap API calls with rate limiting
   result = await rate_limited_request(make_api_call, params)
   ```

### For System Administrators

1. **API Key Management**

   - Store API keys in environment variables
   - Rotate keys regularly (recommended: every 90 days)
   - Use different keys for dev/staging/production
   - Monitor key usage for anomalies

2. **Monitoring**

   - Set up alerts for circuit breaker state changes
   - Monitor rate limit consumption
   - Track cache hit/miss ratios
   - Log validation failures (potential attacks)

3. **Resource Limits**
   ```bash
   # Configure limits based on your environment
   export BIOMCP_FDA_CACHE_TTL=15  # Minutes
   export BIOMCP_FDA_MAX_CACHE_SIZE=100
   export BIOMCP_FDA_MAX_RESPONSE_SIZE=1048576  # 1MB
   ```

## Threat Model

### Threats Addressed

| Threat              | Mitigation                  | Implementation         |
| ------------------- | --------------------------- | ---------------------- |
| SQL Injection       | Input sanitization          | `input_validation.py`  |
| XSS Attacks         | HTML/JS character removal   | `sanitize_input()`     |
| Command Injection   | Shell metacharacter removal | `sanitize_input()`     |
| API Key Exposure    | Exclusion from logs/cache   | `cache.py`, `utils.py` |
| DoS via Rate Limits | Client-side rate limiting   | `rate_limiter.py`      |
| Cascading Failures  | Circuit breaker pattern     | `CircuitBreaker` class |
| Memory Exhaustion   | Response size limits        | `MAX_RESPONSE_SIZE`    |
| Race Conditions     | File locking                | `fcntl` usage          |
| Cache Poisoning     | Input validation            | `build_safe_query()`   |

### Residual Risks

1. **API Key Compromise**: If environment is compromised, keys are accessible

   - **Mitigation**: Use secret management systems in production

2. **Zero-Day FDA API Vulnerabilities**: Unknown vulnerabilities in FDA API

   - **Mitigation**: Monitor FDA security advisories

3. **Distributed DoS**: Multiple clients could still overwhelm FDA API
   - **Mitigation**: Implement global rate limiting at gateway level

## Compliance Considerations

### HIPAA (If Applicable)

While FDA's public APIs don't contain PHI, if extended to include patient data:

1. **Encryption**: Use TLS for all API communications
2. **Audit Logging**: Log all data access (but not the data itself)
3. **Access Controls**: Implement user authentication/authorization
4. **Data Retention**: Define and enforce retention policies

### FDA Data Usage

1. **Attribution**: Always include FDA disclaimers in responses
2. **Data Currency**: Warn users that data may not be real-time
3. **Medical Decisions**: Explicitly state data is not for clinical decisions
4. **Rate Limits**: Respect FDA's terms of service

## Security Testing

### Automated Tests

Run security tests with:

```bash
pytest tests/tdd/openfda/test_security.py -v
```

Tests cover:

- Input validation
- Cache key security
- Rate limiting
- Circuit breaker
- File operations

### Manual Security Review

Checklist for security review:

- [ ] No sensitive data in logs
- [ ] All inputs validated
- [ ] Rate limiting functional
- [ ] Circuit breaker triggers correctly
- [ ] Cache size limited
- [ ] File operations are atomic
- [ ] API keys not in cache keys
- [ ] Error messages don't leak information

## Incident Response

### If API Key is Compromised

1. **Immediate**: Revoke compromised key at FDA portal
2. **Generate**: Create new API key
3. **Update**: Update environment variables
4. **Restart**: Restart services to load new key
5. **Audit**: Review logs for unauthorized usage

### If Rate Limits Exceeded

1. **Check**: Verify circuit breaker state
2. **Wait**: Allow circuit breaker recovery timeout
3. **Reduce**: Lower request rate if needed
4. **Monitor**: Check for abnormal usage patterns

### If Security Vulnerability Found

1. **Assess**: Determine severity and exploitability
2. **Patch**: Develop and test fix
3. **Deploy**: Roll out fix with monitoring
4. **Document**: Update this security documentation
5. **Notify**: Inform users if data was at risk

## Configuration Reference

### Environment Variables

| Variable                       | Default | Description                        |
| ------------------------------ | ------- | ---------------------------------- |
| `OPENFDA_API_KEY`              | None    | FDA API key for higher rate limits |
| `BIOMCP_FDA_CACHE_TTL`         | 15      | Cache TTL in minutes               |
| `BIOMCP_FDA_MAX_CACHE_SIZE`    | 100     | Maximum cache entries              |
| `BIOMCP_FDA_MAX_RESPONSE_SIZE` | 1048576 | Maximum response size in bytes     |
| `BIOMCP_SHORTAGE_CACHE_TTL`    | 24      | Drug shortage cache TTL in hours   |

### Security Headers

When deploying as a web service, add these headers:

```python
headers = {
    "X-Content-Type-Options": "nosniff",
    "X-Frame-Options": "DENY",
    "X-XSS-Protection": "1; mode=block",
    "Strict-Transport-Security": "max-age=31536000; includeSubDomains",
    "Content-Security-Policy": "default-src 'self'"
}
```

## Contact

For security issues, contact: security@biomcp.org (create this address)

For FDA API issues, see: https://open.fda.gov/apis/

---

_Last Updated: 2025-08-07_
_Version: 1.0_

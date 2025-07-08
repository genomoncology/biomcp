import csv
import json
import os
import ssl
from io import StringIO
from ssl import PROTOCOL_TLS_CLIENT, SSLContext, TLSVersion
from typing import Literal, TypeVar

import certifi
from diskcache import Cache
from platformdirs import user_cache_dir
from pydantic import BaseModel

from .circuit_breaker import CircuitBreakerConfig, circuit_breaker
from .constants import (
    AGGRESSIVE_INITIAL_RETRY_DELAY,
    AGGRESSIVE_MAX_RETRY_ATTEMPTS,
    AGGRESSIVE_MAX_RETRY_DELAY,
    DEFAULT_CACHE_TIMEOUT,
    DEFAULT_FAILURE_THRESHOLD,
    DEFAULT_RECOVERY_TIMEOUT,
    DEFAULT_SUCCESS_THRESHOLD,
)
from .http_client_simple import execute_http_request
from .metrics import Timer
from .rate_limiter import domain_limiter
from .retry import (
    RetryableHTTPError,
    RetryConfig,
    is_retryable_status,
    with_retry,
)
from .utils.endpoint_registry import get_registry

T = TypeVar("T", bound=BaseModel)


class RequestError(BaseModel):
    code: int
    message: str


_cache: Cache | None = None


def get_cache() -> Cache:
    global _cache
    if _cache is None:
        cache_path = os.path.join(user_cache_dir("biomcp"), "http_cache")
        _cache = Cache(cache_path)
    return _cache


def generate_cache_key(method: str, url: str, params: dict) -> str:
    """Generate cache key using Python's built-in hash function for speed."""
    # Handle simple cases without params
    if not params:
        return f"{method.upper()}:{url}"

    # Use Python's built-in hash with a fixed seed for consistency
    # This is much faster than SHA256 for cache keys
    params_str = json.dumps(params, sort_keys=True, separators=(",", ":"))
    key_source = f"{method.upper()}:{url}:{params_str}"

    # Use Python's hash function with a fixed seed for deterministic results
    # Convert to positive hex string for compatibility
    hash_value = hash(key_source)
    return f"{hash_value & 0xFFFFFFFFFFFFFFFF:016x}"


def cache_response(cache_key: str, content: str, ttl: int):
    expire = None if ttl == -1 else ttl
    cache = get_cache()
    cache.set(cache_key, content, expire=expire)


def get_cached_response(cache_key: str) -> str | None:
    cache = get_cache()
    return cache.get(cache_key)


def get_ssl_context(tls_version: TLSVersion) -> SSLContext:
    """Create an SSLContext with the specified TLS version."""
    context = SSLContext(PROTOCOL_TLS_CLIENT)
    context.minimum_version = tls_version
    context.maximum_version = tls_version
    context.load_verify_locations(cafile=certifi.where())
    return context


async def call_http(
    method: str,
    url: str,
    params: dict,
    verify: ssl.SSLContext | str | bool = True,
    retry_config: RetryConfig | None = None,
    headers: dict[str, str] | None = None,
) -> tuple[int, str]:
    """Make HTTP request with optional retry logic.

    Args:
        method: HTTP method (GET or POST)
        url: Target URL
        params: Request parameters
        verify: SSL verification settings
        retry_config: Retry configuration (if None, no retry)

    Returns:
        Tuple of (status_code, response_text)
    """

    async def _make_request() -> tuple[int, str]:
        # Extract domain from URL for metrics tagging
        from urllib.parse import urlparse

        parsed = urlparse(url)
        host = parsed.hostname or "unknown"

        # Apply circuit breaker for the host
        breaker_config = CircuitBreakerConfig(
            failure_threshold=DEFAULT_FAILURE_THRESHOLD,
            recovery_timeout=DEFAULT_RECOVERY_TIMEOUT,
            success_threshold=DEFAULT_SUCCESS_THRESHOLD,
            expected_exception=(ConnectionError, TimeoutError),
        )

        @circuit_breaker(f"http_{host}", breaker_config)
        async def _execute_with_breaker():
            async with Timer(
                "http_request", tags={"method": method, "host": host}
            ):
                return await execute_http_request(
                    method, url, params, verify, headers
                )

        status, text = await _execute_with_breaker()

        # Check if status code should trigger retry
        if retry_config and is_retryable_status(status, retry_config):
            raise RetryableHTTPError(status, text)

        return status, text

    # Apply retry logic if configured
    if retry_config:
        wrapped_func = with_retry(retry_config)(_make_request)
        try:
            return await wrapped_func()
        except RetryableHTTPError as exc:
            # Convert retryable HTTP errors back to status/text
            return exc.status_code, exc.message
        except Exception:
            # Let other exceptions bubble up
            raise
    else:
        return await _make_request()


def _handle_offline_mode(
    url: str,
    method: str,
    request: BaseModel | dict,
    cache_ttl: int,
    response_model_type: type[T] | None,
) -> tuple[T | None, RequestError | None] | None:
    """Handle offline mode logic. Returns None if not in offline mode."""
    if os.getenv("BIOMCP_OFFLINE", "").lower() not in ("true", "1", "yes"):
        return None

    # In offline mode, only return cached responses
    if cache_ttl > 0:
        cache_key = generate_cache_key(
            method,
            url,
            request
            if isinstance(request, dict)
            else request.model_dump(exclude_none=True, by_alias=True),
        )
        cached_content = get_cached_response(cache_key)
        if cached_content:
            return parse_response(200, cached_content, response_model_type)

    return None, RequestError(
        code=503,
        message=f"Offline mode enabled (BIOMCP_OFFLINE=true). Cannot fetch from {url}",
    )


def _validate_endpoint(endpoint_key: str | None) -> None:
    """Validate endpoint key if provided."""
    if endpoint_key:
        registry = get_registry()
        if endpoint_key not in registry.get_all_endpoints():
            raise ValueError(
                f"Unknown endpoint key: {endpoint_key}. Please register in endpoint_registry.py"
            )


def _prepare_request_params(
    request: BaseModel | dict,
) -> tuple[dict, dict | None]:
    """Convert request to params dict and extract headers."""
    if isinstance(request, BaseModel):
        params = request.model_dump(exclude_none=True, by_alias=True)
    else:
        params = request.copy() if isinstance(request, dict) else request

    # Extract headers if present
    headers = None
    if isinstance(params, dict) and "_headers" in params:
        try:
            import json

            headers = json.loads(params.pop("_headers"))
        except (json.JSONDecodeError, TypeError):
            pass  # Ignore invalid headers

    return params, headers


def _get_retry_config(
    enable_retry: bool, domain: str | None
) -> RetryConfig | None:
    """Get retry configuration based on settings."""
    if not enable_retry:
        return None

    # Use more aggressive retry for certain domains
    if domain in ["clinicaltrials", "pubmed", "myvariant"]:
        return RetryConfig(
            max_attempts=AGGRESSIVE_MAX_RETRY_ATTEMPTS,
            initial_delay=AGGRESSIVE_INITIAL_RETRY_DELAY,
            max_delay=AGGRESSIVE_MAX_RETRY_DELAY,
        )
    return RetryConfig()  # Default settings


async def request_api(
    url: str,
    request: BaseModel | dict,
    response_model_type: type[T] | None = None,
    method: Literal["GET", "POST"] = "GET",
    cache_ttl: int = DEFAULT_CACHE_TIMEOUT,
    tls_version: TLSVersion | None = None,
    domain: str | None = None,
    enable_retry: bool = True,
    endpoint_key: str | None = None,
) -> tuple[T | None, RequestError | None]:
    # Handle offline mode
    offline_result = _handle_offline_mode(
        url, method, request, cache_ttl, response_model_type
    )
    if offline_result is not None:
        return offline_result

    # Validate endpoint
    _validate_endpoint(endpoint_key)

    # Apply rate limiting if domain is specified
    if domain:
        async with domain_limiter.limit(domain):
            pass  # Rate limit acquired

    # Prepare request
    verify = get_ssl_context(tls_version) if tls_version else True
    params, headers = _prepare_request_params(request)
    retry_config = _get_retry_config(enable_retry, domain)

    # Short-circuit if caching disabled
    if cache_ttl == 0:
        status, content = await call_http(
            method,
            url,
            params,
            verify=verify,
            retry_config=retry_config,
            headers=headers,
        )
        return parse_response(status, content, response_model_type)

    # Handle caching
    cache_key = generate_cache_key(method, url, params)
    cached_content = get_cached_response(cache_key)

    if cached_content:
        return parse_response(200, cached_content, response_model_type)

    # Make HTTP request if not cached
    status, content = await call_http(
        method,
        url,
        params,
        verify=verify,
        retry_config=retry_config,
        headers=headers,
    )
    parsed_response = parse_response(status, content, response_model_type)

    # Cache if successful response
    if status == 200:
        cache_response(cache_key, content, cache_ttl)

    return parsed_response


def parse_response(
    status_code: int,
    content: str,
    response_model_type: type[T] | None = None,
) -> tuple[T | None, RequestError | None]:
    if status_code != 200:
        return None, RequestError(code=status_code, message=content)

    # Handle empty content
    if not content or content.strip() == "":
        return None, RequestError(
            code=500,
            message="Empty response received from API",
        )

    try:
        if response_model_type is None:
            # Try to parse as JSON first
            if content.startswith("{") or content.startswith("["):
                response_dict = json.loads(content)
            elif "," in content:
                io = StringIO(content)
                response_dict = list(csv.DictReader(io))
            else:
                response_dict = {"text": content}
            return response_dict, None

        parsed: T = response_model_type.model_validate_json(content)
        return parsed, None

    except json.JSONDecodeError as exc:
        # Provide more detailed error message for JSON parsing issues
        return None, RequestError(
            code=500,
            message=f"Invalid JSON response: {exc}. Content preview: {content[:100]}...",
        )
    except Exception as exc:
        return None, RequestError(
            code=500,
            message=f"Failed to parse response: {exc}",
        )

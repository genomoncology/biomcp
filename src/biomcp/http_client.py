import csv
import hashlib
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


# noinspection PyTypeChecker
def generate_cache_key(method: str, url: str, params: dict) -> str:
    sha256_hash = hashlib.sha256()
    params_dump: str = json.dumps(params, sort_keys=True)
    key_source: str = f"{method.upper()}:{url}:{params_dump}"
    data: bytes = key_source.encode("utf-8")
    sha256_hash.update(data)
    return sha256_hash.hexdigest()


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
                return await execute_http_request(method, url, params, verify)

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


async def request_api(
    url: str,
    request: BaseModel | dict,
    response_model_type: type[T] | None = None,
    method: Literal["GET", "POST"] = "GET",
    cache_ttl: int = DEFAULT_CACHE_TIMEOUT,
    tls_version: TLSVersion | None = None,
    domain: str | None = None,
    enable_retry: bool = True,
) -> tuple[T | None, RequestError | None]:
    # Apply rate limiting if domain is specified
    if domain:
        async with domain_limiter.limit(domain):
            pass  # Rate limit acquired

    verify = get_ssl_context(tls_version) if tls_version else True

    # Convert request to params dic
    if isinstance(request, BaseModel):
        params = request.model_dump(exclude_none=True, by_alias=True)
    else:
        params = request

    # Configure retry logic if enabled
    retry_config = None
    if enable_retry:
        # Use more aggressive retry for certain domains
        if domain in ["clinicaltrials", "pubmed", "myvariant"]:
            retry_config = RetryConfig(
                max_attempts=AGGRESSIVE_MAX_RETRY_ATTEMPTS,
                initial_delay=AGGRESSIVE_INITIAL_RETRY_DELAY,
                max_delay=AGGRESSIVE_MAX_RETRY_DELAY,
            )
        else:
            retry_config = RetryConfig()  # Default settings

    # Short-circuit if caching disabled
    if cache_ttl == 0:
        status, content = await call_http(
            method, url, params, verify=verify, retry_config=retry_config
        )
        return parse_response(status, content, response_model_type)

    # Else caching enabled:
    cache_key = generate_cache_key(method, url, params)
    cached_content = get_cached_response(cache_key)

    if cached_content:
        return parse_response(200, cached_content, response_model_type)

    # Make HTTP request if not cached
    status, content = await call_http(
        method, url, params, verify=verify, retry_config=retry_config
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

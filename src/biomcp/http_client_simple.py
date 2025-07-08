"""Helper functions for simpler HTTP client operations."""

import asyncio
import contextlib
import json
import os
import ssl

import httpx

# Global connection pools per SSL context
_connection_pools: dict[str, httpx.AsyncClient] = {}
_pool_lock = asyncio.Lock()


def close_all_pools():
    """Close all connection pools. Useful for cleanup in tests."""
    global _connection_pools
    for pool in _connection_pools.values():
        if pool and not pool.is_closed:
            # Schedule the close in a safe way
            try:
                # Store task reference to avoid garbage collection
                close_task = asyncio.create_task(pool.aclose())
                # Optionally add a callback to handle completion
                close_task.add_done_callback(lambda t: None)
            except RuntimeError:
                # If no event loop is running, close synchronously
                pool._transport.close()
    _connection_pools.clear()


async def get_connection_pool(
    verify: ssl.SSLContext | str | bool,
    timeout: httpx.Timeout,
) -> httpx.AsyncClient:
    """Get or create a shared connection pool for the given SSL context."""
    global _connection_pools

    # Create a key for the pool based on verify setting
    if isinstance(verify, ssl.SSLContext):
        pool_key = f"ssl_{id(verify)}"
    else:
        pool_key = str(verify)

    async with _pool_lock:
        pool = _connection_pools.get(pool_key)
        if pool is None or pool.is_closed:
            # Create a new connection pool with optimized settings
            pool = httpx.AsyncClient(
                verify=verify,
                http2=False,  # HTTP/2 can add overhead for simple requests
                timeout=timeout,
                limits=httpx.Limits(
                    max_keepalive_connections=20,  # Reuse connections
                    max_connections=100,  # Total connection limit
                    keepalive_expiry=30,  # Keep connections alive for 30s
                ),
                # Enable connection pooling
                transport=httpx.AsyncHTTPTransport(
                    retries=0,  # We handle retries at a higher level
                ),
            )
            _connection_pools[pool_key] = pool
        return pool


async def execute_http_request(  # noqa: C901
    method: str,
    url: str,
    params: dict,
    verify: ssl.SSLContext | str | bool,
    headers: dict[str, str] | None = None,
) -> tuple[int, str]:
    """Execute the actual HTTP request using connection pooling.

    Args:
        method: HTTP method (GET or POST)
        url: Target URL
        params: Request parameters
        verify: SSL verification settings
        headers: Optional custom headers

    Returns:
        Tuple of (status_code, response_text)

    Raises:
        ConnectionError: For connection failures
        TimeoutError: For timeout errors
    """
    from .constants import HTTP_TIMEOUT_SECONDS

    try:
        # Extract custom headers from params if present
        custom_headers = headers or {}
        if "_headers" in params:
            with contextlib.suppress(json.JSONDecodeError, TypeError):
                custom_headers.update(json.loads(params.pop("_headers")))

        # Use the configured timeout from constants
        timeout = httpx.Timeout(HTTP_TIMEOUT_SECONDS)

        # Use connection pooling with proper error handling
        use_pool = (
            os.getenv("BIOMCP_USE_CONNECTION_POOL", "true").lower() == "true"
        )

        if use_pool:
            try:
                # Use the new connection pool manager
                from ..connection_pool import get_connection_pool as get_pool

                client = await get_pool(verify, timeout)
                should_close = False
            except Exception:
                # Fallback to creating a new client
                client = httpx.AsyncClient(
                    verify=verify, http2=False, timeout=timeout
                )
                should_close = True
        else:
            # Create a new client for each request
            client = httpx.AsyncClient(
                verify=verify, http2=False, timeout=timeout
            )
            should_close = True

        try:
            # Make the request
            if method.upper() == "GET":
                resp = await client.get(
                    url, params=params, headers=custom_headers
                )
            elif method.upper() == "POST":
                resp = await client.post(
                    url, json=params, headers=custom_headers
                )
            else:
                from .constants import HTTP_ERROR_CODE_UNSUPPORTED_METHOD

                return (
                    HTTP_ERROR_CODE_UNSUPPORTED_METHOD,
                    f"Unsupported method {method}",
                )

            # Check for empty response
            if not resp.text:
                return resp.status_code, "{}"

            return resp.status_code, resp.text
        finally:
            # Only close if we created a new client
            if should_close:
                await client.aclose()

    except httpx.ConnectError as exc:
        raise ConnectionError(f"Failed to connect to {url}: {exc}") from exc
    except httpx.TimeoutException as exc:
        raise TimeoutError(f"Request to {url} timed out: {exc}") from exc
    except httpx.HTTPError as exc:
        error_msg = str(exc) if str(exc) else "Network connectivity error"
        from .constants import HTTP_ERROR_CODE_NETWORK

        return HTTP_ERROR_CODE_NETWORK, error_msg

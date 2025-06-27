"""Helper functions for simpler HTTP client operations."""

import ssl

import httpx


async def execute_http_request(
    method: str,
    url: str,
    params: dict,
    verify: ssl.SSLContext | str | bool,
) -> tuple[int, str]:
    """Execute the actual HTTP request.

    Args:
        method: HTTP method (GET or POST)
        url: Target URL
        params: Request parameters
        verify: SSL verification settings

    Returns:
        Tuple of (status_code, response_text)

    Raises:
        ConnectionError: For connection failures
        TimeoutError: For timeout errors
    """
    from .constants import HTTP_TIMEOUT_SECONDS

    try:
        # Use the configured timeout from constants
        timeout = httpx.Timeout(HTTP_TIMEOUT_SECONDS)
        async with httpx.AsyncClient(
            verify=verify, http2=False, timeout=timeout
        ) as client:
            if method.upper() == "GET":
                resp = await client.get(url, params=params)
            elif method.upper() == "POST":
                resp = await client.post(url, json=params)
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

    except httpx.ConnectError as exc:
        raise ConnectionError(f"Failed to connect to {url}: {exc}") from exc
    except httpx.TimeoutException as exc:
        raise TimeoutError(f"Request to {url} timed out: {exc}") from exc
    except httpx.HTTPError as exc:
        error_msg = str(exc) if str(exc) else "Network connectivity error"
        from .constants import HTTP_ERROR_CODE_NETWORK

        return HTTP_ERROR_CODE_NETWORK, error_msg

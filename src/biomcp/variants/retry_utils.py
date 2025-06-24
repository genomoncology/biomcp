"""Retry utilities for API calls."""

import asyncio
import logging
from collections.abc import Callable, Coroutine
from typing import Any, TypeVar

import httpx

logger = logging.getLogger(__name__)

T = TypeVar("T")

# Retry configuration
MAX_RETRIES = 3
INITIAL_BACKOFF = 1.0  # seconds
MAX_BACKOFF = 16.0  # seconds
BACKOFF_MULTIPLIER = 2.0

# HTTP status codes that should trigger retries
RETRIABLE_STATUS_CODES = {
    408,  # Request Timeout
    429,  # Too Many Requests
    500,  # Internal Server Error
    502,  # Bad Gateway
    503,  # Service Unavailable
    504,  # Gateway Timeout
}


async def retry_with_backoff(
    func: Callable[..., Coroutine[Any, Any, T]],
    *args,
    max_retries: int = MAX_RETRIES,
    initial_backoff: float = INITIAL_BACKOFF,
    **kwargs,
) -> T | None:
    """Execute an async function with exponential backoff retry logic.

    Args:
        func: Async function to execute
        *args: Positional arguments for func
        max_retries: Maximum number of retry attempts
        initial_backoff: Initial backoff time in seconds
        **kwargs: Keyword arguments for func

    Returns:
        Function result or None if all retries failed
    """
    backoff = initial_backoff
    last_exception: Exception | None = None

    for attempt in range(max_retries + 1):
        try:
            return await func(*args, **kwargs)
        except httpx.HTTPStatusError as e:
            if e.response.status_code in RETRIABLE_STATUS_CODES:
                last_exception = e
                if attempt < max_retries:
                    wait_time = min(backoff, MAX_BACKOFF)
                    logger.warning(
                        f"HTTP {e.response.status_code} error, retrying in {wait_time}s "
                        f"(attempt {attempt + 1}/{max_retries})"
                    )
                    await asyncio.sleep(wait_time)
                    backoff *= BACKOFF_MULTIPLIER
                else:
                    logger.error(
                        f"HTTP {e.response.status_code} error after {max_retries} retries"
                    )
            else:
                # Non-retriable status code
                raise
        except (
            httpx.ConnectTimeout,
            httpx.ReadTimeout,
            httpx.ConnectError,
        ) as e:
            last_exception = e
            if attempt < max_retries:
                wait_time = min(backoff, MAX_BACKOFF)
                logger.warning(
                    f"Network error ({type(e).__name__}), retrying in {wait_time}s "
                    f"(attempt {attempt + 1}/{max_retries})"
                )
                await asyncio.sleep(wait_time)
                backoff *= BACKOFF_MULTIPLIER
            else:
                logger.error(
                    f"Network error ({type(e).__name__}) after {max_retries} retries"
                )
        except Exception as e:
            # Non-retriable exception
            logger.error(f"Non-retriable error: {type(e).__name__}: {e}")
            raise

    # All retries exhausted
    if last_exception:
        logger.error(
            f"All retry attempts failed. Last error: {type(last_exception).__name__}"
        )

    return None


class RetryableHTTPClient:
    """HTTP client wrapper with retry logic."""

    def __init__(
        self,
        timeout: float = 30.0,
        max_retries: int = MAX_RETRIES,
        initial_backoff: float = INITIAL_BACKOFF,
    ):
        self.timeout = timeout
        self.max_retries = max_retries
        self.initial_backoff = initial_backoff

    async def get_with_retry(
        self,
        client: httpx.AsyncClient,
        url: str,
        **kwargs: Any,
    ) -> httpx.Response | None:
        """Execute GET request with retry logic.

        Args:
            client: httpx AsyncClient instance
            url: URL to request
            **kwargs: Additional arguments for client.get()

        Returns:
            Response object or None if all retries failed
        """

        async def _get() -> httpx.Response:
            response = await client.get(url, **kwargs)
            response.raise_for_status()
            return response

        return await retry_with_backoff(
            _get,
            max_retries=self.max_retries,
            initial_backoff=self.initial_backoff,
        )

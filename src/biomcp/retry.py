"""Retry logic with exponential backoff for handling transient failures."""

import asyncio
import functools
import logging
import secrets
from collections.abc import Callable, Coroutine
from typing import Any, TypeVar

from .constants import (
    DEFAULT_EXPONENTIAL_BASE,
    DEFAULT_INITIAL_RETRY_DELAY,
    DEFAULT_MAX_RETRY_ATTEMPTS,
    DEFAULT_MAX_RETRY_DELAY,
    METRIC_JITTER_RANGE,
)

logger = logging.getLogger(__name__)

T = TypeVar("T")


class RetryConfig:
    """Configuration for retry behavior."""

    def __init__(
        self,
        max_attempts: int = DEFAULT_MAX_RETRY_ATTEMPTS,
        initial_delay: float = DEFAULT_INITIAL_RETRY_DELAY,
        max_delay: float = DEFAULT_MAX_RETRY_DELAY,
        exponential_base: float = DEFAULT_EXPONENTIAL_BASE,
        jitter: bool = True,
        retryable_exceptions: tuple[type[Exception], ...] = (
            ConnectionError,
            TimeoutError,
            OSError,
        ),
        retryable_status_codes: tuple[int, ...] = (429, 502, 503, 504),
    ):
        """Initialize retry configuration.

        Args:
            max_attempts: Maximum number of retry attempts
            initial_delay: Initial delay between retries in seconds
            max_delay: Maximum delay between retries in seconds
            exponential_base: Base for exponential backoff calculation
            jitter: Whether to add random jitter to delays
            retryable_exceptions: Exception types that should trigger retry
            retryable_status_codes: HTTP status codes that should trigger retry
        """
        self.max_attempts = max_attempts
        self.initial_delay = initial_delay
        self.max_delay = max_delay
        self.exponential_base = exponential_base
        self.jitter = jitter
        self.retryable_exceptions = retryable_exceptions
        self.retryable_status_codes = retryable_status_codes


def calculate_delay(attempt: int, config: RetryConfig) -> float:
    """Calculate delay for the next retry attempt.

    Args:
        attempt: Current attempt number (0-based)
        config: Retry configuration

    Returns:
        Delay in seconds before the next retry
    """
    # Exponential backoff: delay = initial_delay * (base ^ attempt)
    delay = config.initial_delay * (config.exponential_base**attempt)

    # Cap at maximum delay
    delay = min(delay, config.max_delay)

    # Add jitter to prevent thundering herd
    if config.jitter:
        jitter_range = delay * METRIC_JITTER_RANGE  # 10% jitter
        # Use secrets for cryptographically secure randomness
        # Generate random float between -1 and 1, then scale
        random_factor = (secrets.randbits(32) / (2**32 - 1)) * 2 - 1
        jitter = random_factor * jitter_range
        delay += jitter

    return max(0, delay)  # Ensure non-negative


def is_retryable_exception(exc: Exception, config: RetryConfig) -> bool:
    """Check if an exception should trigger a retry.

    Args:
        exc: The exception that occurred
        config: Retry configuration

    Returns:
        True if the exception is retryable
    """
    return isinstance(exc, config.retryable_exceptions)


def is_retryable_status(status_code: int, config: RetryConfig) -> bool:
    """Check if an HTTP status code should trigger a retry.

    Args:
        status_code: HTTP status code
        config: Retry configuration

    Returns:
        True if the status code is retryable
    """
    return status_code in config.retryable_status_codes


def with_retry(
    config: RetryConfig | None = None,
) -> Callable[
    [Callable[..., Coroutine[Any, Any, T]]],
    Callable[..., Coroutine[Any, Any, T]],
]:
    """Decorator to add retry logic to async functions.

    Args:
        config: Retry configuration (uses defaults if not provided)

    Returns:
        Decorated function with retry logic
    """
    if config is None:
        config = RetryConfig()

    def decorator(
        func: Callable[..., Coroutine[Any, Any, T]],
    ) -> Callable[..., Coroutine[Any, Any, T]]:
        @functools.wraps(func)
        async def wrapper(*args: Any, **kwargs: Any) -> T:
            last_exception = None

            for attempt in range(config.max_attempts):
                try:
                    return await func(*args, **kwargs)
                except Exception as exc:
                    last_exception = exc

                    # Check if this is the last attempt
                    if attempt == config.max_attempts - 1:
                        logger.error(
                            f"Max retry attempts ({config.max_attempts}) "
                            f"reached for {func.__name__}: {exc}"
                        )
                        raise

                    # Check if the exception is retryable
                    if not is_retryable_exception(exc, config):
                        logger.debug(
                            f"Non-retryable exception in {func.__name__}: {exc}"
                        )
                        raise

                    # Calculate delay for next attempt
                    delay = calculate_delay(attempt, config)
                    logger.warning(
                        f"Retry attempt {attempt + 1}/{config.max_attempts} "
                        f"for {func.__name__} after {delay:.2f}s delay. "
                        f"Error: {exc}"
                    )

                    # Wait before retrying
                    await asyncio.sleep(delay)

            # This should never be reached due to the raise in the loop
            if last_exception:
                raise last_exception
            raise RuntimeError("Unexpected retry loop exit")

        return wrapper

    return decorator


class RetryableHTTPError(Exception):
    """Exception wrapper for HTTP errors that should be retried."""

    def __init__(self, status_code: int, message: str):
        self.status_code = status_code
        self.message = message
        super().__init__(f"HTTP {status_code}: {message}")


async def retry_with_backoff(
    func: Callable[..., Coroutine[Any, Any, T]],
    *args: Any,
    config: RetryConfig | None = None,
    **kwargs: Any,
) -> T:
    """Execute a function with retry logic and exponential backoff.

    This is an alternative to the decorator for cases where you need
    more control over retry behavior.

    Args:
        func: Async function to execute
        *args: Positional arguments for the function
        config: Retry configuration (uses defaults if not provided)
        **kwargs: Keyword arguments for the function

    Returns:
        Result of the function call

    Raises:
        The last exception if all retries fail
    """
    if config is None:
        config = RetryConfig()

    last_exception = None

    for attempt in range(config.max_attempts):
        try:
            return await func(*args, **kwargs)
        except Exception as exc:
            last_exception = exc

            # Check if this is the last attempt
            if attempt == config.max_attempts - 1:
                logger.error(
                    f"Max retry attempts ({config.max_attempts}) "
                    f"reached for {func.__name__}: {exc}"
                )
                raise

            # Check if the exception is retryable
            if not is_retryable_exception(exc, config):
                logger.debug(
                    f"Non-retryable exception in {func.__name__}: {exc}"
                )
                raise

            # Calculate delay for next attempt
            delay = calculate_delay(attempt, config)
            logger.warning(
                f"Retry attempt {attempt + 1}/{config.max_attempts} "
                f"for {func.__name__} after {delay:.2f}s delay. "
                f"Error: {exc}"
            )

            # Wait before retrying
            await asyncio.sleep(delay)

    # This should never be reached due to the raise in the loop
    if last_exception:
        raise last_exception
    raise RuntimeError("Unexpected retry loop exit")

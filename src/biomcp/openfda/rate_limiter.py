"""
Rate limiting and circuit breaker for OpenFDA API requests.

This module provides client-side rate limiting to prevent API quota exhaustion
and circuit breaker pattern to handle API failures gracefully.
"""

import asyncio
import logging
import os
import time
from collections.abc import Callable
from datetime import datetime
from enum import Enum
from typing import Any

logger = logging.getLogger(__name__)


class CircuitState(Enum):
    """Circuit breaker states."""

    CLOSED = "closed"  # Normal operation
    OPEN = "open"  # Blocking requests
    HALF_OPEN = "half_open"  # Testing recovery


class RateLimiter:
    """
    Token bucket rate limiter for FDA API requests.
    """

    def __init__(self, rate: int = 10, per: float = 1.0):
        """
        Initialize rate limiter.

        Args:
            rate: Number of requests allowed
            per: Time period in seconds
        """
        self.rate = rate
        self.per = per
        self.allowance = float(rate)
        self.last_check = time.monotonic()
        self._lock = asyncio.Lock()

    async def acquire(self) -> None:
        """
        Acquire permission to make a request.
        Blocks if rate limit would be exceeded.
        """
        async with self._lock:
            current = time.monotonic()
            time_passed = current - self.last_check
            self.last_check = current

            # Add tokens based on time passed
            self.allowance += time_passed * (self.rate / self.per)

            # Cap at maximum rate
            if self.allowance > self.rate:
                self.allowance = float(self.rate)

            # Check if we can proceed
            if self.allowance < 1.0:
                # Calculate wait time
                deficit = 1.0 - self.allowance
                wait_time = deficit * (self.per / self.rate)

                logger.debug(f"Rate limit: waiting {wait_time:.2f}s")
                await asyncio.sleep(wait_time)

                # Update allowance after waiting
                self.allowance = 0.0
            else:
                # Consume one token
                self.allowance -= 1.0


class CircuitBreaker:
    """
    Circuit breaker to prevent cascading failures.
    """

    def __init__(
        self,
        failure_threshold: int = 5,
        recovery_timeout: int = 60,
        half_open_max_calls: int = 3,
    ):
        """
        Initialize circuit breaker.

        Args:
            failure_threshold: Number of failures before opening circuit
            recovery_timeout: Seconds to wait before attempting recovery
            half_open_max_calls: Max calls allowed in half-open state
        """
        self.failure_threshold = failure_threshold
        self.recovery_timeout = recovery_timeout
        self.half_open_max_calls = half_open_max_calls

        self.failure_count = 0
        self.last_failure_time: float | None = None
        self.state = CircuitState.CLOSED
        self.half_open_calls = 0
        self._lock = asyncio.Lock()

    async def call(self, func: Callable, *args, **kwargs) -> Any:
        """
        Execute function through circuit breaker.

        Args:
            func: Async function to call
            *args: Function arguments
            **kwargs: Function keyword arguments

        Returns:
            Function result

        Raises:
            Exception: If circuit is open or function fails
        """
        async with self._lock:
            # Check circuit state
            if self.state == CircuitState.OPEN:
                if self._should_attempt_reset():
                    self.state = CircuitState.HALF_OPEN
                    self.half_open_calls = 0
                    logger.info(
                        "Circuit breaker: attempting recovery (half-open)"
                    )
                else:
                    if self.last_failure_time is not None:
                        time_left = self.recovery_timeout - (
                            time.time() - self.last_failure_time
                        )
                        raise Exception(
                            f"Circuit breaker is OPEN. Retry in {time_left:.0f} seconds"
                        )
                    else:
                        raise Exception("Circuit breaker is OPEN")

            elif self.state == CircuitState.HALF_OPEN:
                if self.half_open_calls >= self.half_open_max_calls:
                    # Don't allow more calls in half-open state
                    raise Exception(
                        "Circuit breaker is HALF_OPEN. Max test calls reached"
                    )
                self.half_open_calls += 1

        # Execute the function
        try:
            result = await func(*args, **kwargs)
            await self._on_success()
            return result
        except Exception as e:
            await self._on_failure()
            raise e

    async def _on_success(self) -> None:
        """Handle successful call."""
        async with self._lock:
            if self.state == CircuitState.HALF_OPEN:
                # Recovery succeeded
                self.state = CircuitState.CLOSED
                self.failure_count = 0
                logger.info("Circuit breaker: recovered (closed)")
            else:
                # Reset failure count on success
                self.failure_count = 0

    async def _on_failure(self) -> None:
        """Handle failed call."""
        async with self._lock:
            self.failure_count += 1
            self.last_failure_time = time.time()

            if self.state == CircuitState.HALF_OPEN:
                # Recovery failed, reopen circuit
                self.state = CircuitState.OPEN
                logger.warning("Circuit breaker: recovery failed (open)")
            elif self.failure_count >= self.failure_threshold:
                # Too many failures, open circuit
                self.state = CircuitState.OPEN
                logger.warning(
                    f"Circuit breaker: opened after {self.failure_count} failures"
                )

    def _should_attempt_reset(self) -> bool:
        """Check if enough time has passed to attempt reset."""
        return (
            self.last_failure_time is not None
            and time.time() - self.last_failure_time >= self.recovery_timeout
        )

    @property
    def is_closed(self) -> bool:
        """Check if circuit is closed (normal operation)."""
        return self.state == CircuitState.CLOSED

    @property
    def is_open(self) -> bool:
        """Check if circuit is open (blocking requests)."""
        return self.state == CircuitState.OPEN

    def get_state(self) -> dict[str, Any]:
        """Get current circuit breaker state."""
        return {
            "state": self.state.value,
            "failure_count": self.failure_count,
            "last_failure": (
                datetime.fromtimestamp(self.last_failure_time).isoformat()
                if self.last_failure_time
                else None
            ),
        }


# Global instances
# Configure based on API key availability
_has_api_key = bool(os.environ.get("OPENFDA_API_KEY"))
_rate_limit = 240 if _has_api_key else 40  # per minute

# Create rate limiter (convert to per-second rate)
FDA_RATE_LIMITER = RateLimiter(rate=_rate_limit, per=60.0)

# Create circuit breaker
FDA_CIRCUIT_BREAKER = CircuitBreaker(
    failure_threshold=5, recovery_timeout=60, half_open_max_calls=3
)

# Semaphore for concurrent request limiting
FDA_SEMAPHORE = asyncio.Semaphore(10)  # Max 10 concurrent requests


async def rate_limited_request(func: Callable, *args, **kwargs) -> Any:
    """
    Execute FDA API request with rate limiting and circuit breaker.

    Args:
        func: Async function to call
        *args: Function arguments
        **kwargs: Function keyword arguments

    Returns:
        Function result
    """
    # Apply semaphore for concurrent limiting
    async with FDA_SEMAPHORE:
        # Apply rate limiting
        await FDA_RATE_LIMITER.acquire()

        # Apply circuit breaker
        return await FDA_CIRCUIT_BREAKER.call(func, *args, **kwargs)

"""Circuit breaker pattern implementation for fault tolerance."""

import asyncio
import enum
import logging
from collections.abc import Callable
from dataclasses import dataclass, field
from datetime import datetime
from typing import Any

logger = logging.getLogger(__name__)


class CircuitState(enum.Enum):
    """Circuit breaker states."""

    CLOSED = "closed"  # Normal operation, requests pass through
    OPEN = "open"  # Circuit tripped, requests fail fast
    HALF_OPEN = "half_open"  # Testing if service recovered


@dataclass
class CircuitBreakerConfig:
    """Configuration for circuit breaker behavior."""

    failure_threshold: int = 5
    """Number of failures before opening circuit"""

    recovery_timeout: float = 60.0
    """Seconds to wait before attempting recovery"""

    success_threshold: int = 2
    """Successes needed in half-open state to close circuit"""

    expected_exception: type[Exception] | tuple[type[Exception], ...] = (
        Exception
    )
    """Exception types that count as failures"""

    exclude_exceptions: tuple[type[Exception], ...] = ()
    """Exception types that don't count as failures"""


@dataclass
class CircuitBreakerState:
    """Mutable state for a circuit breaker."""

    state: CircuitState = CircuitState.CLOSED
    failure_count: int = 0
    success_count: int = 0
    last_failure_time: datetime | None = None
    last_state_change: datetime = field(default_factory=datetime.now)
    _lock: asyncio.Lock = field(default_factory=asyncio.Lock)


class CircuitBreakerError(Exception):
    """Raised when circuit breaker is open."""

    def __init__(
        self, message: str, last_failure_time: datetime | None = None
    ):
        super().__init__(message)
        self.last_failure_time = last_failure_time


class CircuitBreaker:
    """Circuit breaker implementation."""

    def __init__(
        self,
        name: str,
        config: CircuitBreakerConfig | None = None,
    ):
        """Initialize circuit breaker.

        Args:
            name: Circuit breaker name for logging
            config: Configuration (uses defaults if not provided)
        """
        self.name = name
        self.config = config or CircuitBreakerConfig()
        self._state = CircuitBreakerState()

    async def call(
        self,
        func: Callable[..., Any],
        *args: Any,
        **kwargs: Any,
    ) -> Any:
        """Execute function through circuit breaker.

        Args:
            func: Async function to execute
            *args: Positional arguments for func
            **kwargs: Keyword arguments for func

        Returns:
            Result of function call

        Raises:
            CircuitBreakerError: If circuit is open
            Exception: If function raises exception
        """
        async with self._state._lock:
            # Check if we should transition from open to half-open
            if self._state.state == CircuitState.OPEN:
                if self._should_attempt_reset():
                    self._state.state = CircuitState.HALF_OPEN
                    self._state.success_count = 0
                    self._state.last_state_change = datetime.now()
                    logger.info(
                        f"Circuit breaker '{self.name}' entering half-open state"
                    )
                else:
                    raise CircuitBreakerError(
                        f"Circuit breaker '{self.name}' is open",
                        self._state.last_failure_time,
                    )

        # Execute the function
        try:
            result = await func(*args, **kwargs)
            await self._on_success()
            return result
        except Exception as exc:
            if await self._on_failure(exc):
                raise
            # If exception doesn't count as failure, re-raise it
            raise

    async def _on_success(self) -> None:
        """Handle successful call."""
        async with self._state._lock:
            if self._state.state == CircuitState.HALF_OPEN:
                self._state.success_count += 1
                if self._state.success_count >= self.config.success_threshold:
                    self._state.state = CircuitState.CLOSED
                    self._state.failure_count = 0
                    self._state.success_count = 0
                    self._state.last_state_change = datetime.now()
                    logger.info(
                        f"Circuit breaker '{self.name}' closed after recovery"
                    )
            elif self._state.state == CircuitState.CLOSED:
                # Reset failure count on success
                self._state.failure_count = 0

    async def _on_failure(self, exc: Exception) -> bool:
        """Handle failed call.

        Args:
            exc: The exception that was raised

        Returns:
            True if exception counts as failure
        """
        # Check if exception should be counted
        if not self._is_counted_exception(exc):
            return False

        async with self._state._lock:
            self._state.failure_count += 1
            self._state.last_failure_time = datetime.now()

            if self._state.state == CircuitState.HALF_OPEN:
                # Single failure in half-open state reopens circuit
                self._state.state = CircuitState.OPEN
                self._state.last_state_change = datetime.now()
                logger.warning(
                    f"Circuit breaker '{self.name}' reopened due to failure in half-open state"
                )
            elif (
                self._state.state == CircuitState.CLOSED
                and self._state.failure_count >= self.config.failure_threshold
            ):
                # Threshold exceeded, open circuit
                self._state.state = CircuitState.OPEN
                self._state.last_state_change = datetime.now()
                logger.error(
                    f"Circuit breaker '{self.name}' opened after {self._state.failure_count} failures"
                )

        return True

    def _should_attempt_reset(self) -> bool:
        """Check if enough time has passed to attempt reset."""
        if self._state.last_failure_time is None:
            return True

        time_since_failure = datetime.now() - self._state.last_failure_time
        return (
            time_since_failure.total_seconds() >= self.config.recovery_timeout
        )

    def _is_counted_exception(self, exc: Exception) -> bool:
        """Check if exception should count as failure."""
        # Check excluded exceptions first
        if isinstance(exc, self.config.exclude_exceptions):
            return False

        # Check expected exceptions
        return isinstance(exc, self.config.expected_exception)

    @property
    def state(self) -> CircuitState:
        """Get current circuit state."""
        return self._state.state

    @property
    def is_open(self) -> bool:
        """Check if circuit is open."""
        return self._state.state == CircuitState.OPEN

    @property
    def is_closed(self) -> bool:
        """Check if circuit is closed."""
        return self._state.state == CircuitState.CLOSED

    async def reset(self) -> None:
        """Manually reset circuit to closed state."""
        async with self._state._lock:
            self._state.state = CircuitState.CLOSED
            self._state.failure_count = 0
            self._state.success_count = 0
            self._state.last_failure_time = None
            self._state.last_state_change = datetime.now()
            logger.info(f"Circuit breaker '{self.name}' manually reset")


# Global registry of circuit breakers
_circuit_breakers: dict[str, CircuitBreaker] = {}


def get_circuit_breaker(
    name: str,
    config: CircuitBreakerConfig | None = None,
) -> CircuitBreaker:
    """Get or create a circuit breaker.

    Args:
        name: Circuit breaker name
        config: Configuration (used only on creation)

    Returns:
        Circuit breaker instance
    """
    if name not in _circuit_breakers:
        _circuit_breakers[name] = CircuitBreaker(name, config)
    return _circuit_breakers[name]


def circuit_breaker(
    name: str | None = None,
    config: CircuitBreakerConfig | None = None,
):
    """Decorator to apply circuit breaker to function.

    Args:
        name: Circuit breaker name (defaults to function name)
        config: Circuit breaker configuration

    Returns:
        Decorated function
    """

    def decorator(func):
        breaker_name = name or f"{func.__module__}.{func.__name__}"
        breaker = get_circuit_breaker(breaker_name, config)

        async def wrapper(*args, **kwargs):
            return await breaker.call(func, *args, **kwargs)

        # Preserve function metadata
        wrapper.__name__ = func.__name__
        wrapper.__doc__ = func.__doc__
        wrapper._circuit_breaker = breaker  # Expose breaker for testing

        return wrapper

    return decorator

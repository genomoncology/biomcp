"""Tests for circuit breaker pattern."""

import asyncio

import pytest

from biomcp.circuit_breaker import (
    CircuitBreaker,
    CircuitBreakerConfig,
    CircuitBreakerError,
    CircuitState,
    circuit_breaker,
    get_circuit_breaker,
)


class CircuitBreakerTestException(Exception):
    """Test exception for circuit breaker tests."""

    pass


class IgnoredException(Exception):
    """Exception that should be ignored by circuit breaker."""

    pass


@pytest.mark.asyncio
async def test_circuit_breaker_closed_state():
    """Test circuit breaker in closed state allows calls."""
    breaker = CircuitBreaker("test_closed")
    call_count = 0

    async def test_func():
        nonlocal call_count
        call_count += 1
        return "success"

    # Should allow calls in closed state
    assert breaker.is_closed
    result = await breaker.call(test_func)
    assert result == "success"
    assert call_count == 1


@pytest.mark.asyncio
async def test_circuit_breaker_opens_on_threshold():
    """Test circuit breaker opens after failure threshold."""
    config = CircuitBreakerConfig(
        failure_threshold=3,
        expected_exception=CircuitBreakerTestException,
    )
    breaker = CircuitBreaker("test_threshold", config)

    async def failing_func():
        raise CircuitBreakerTestException("Test failure")

    # First 2 failures should pass through
    for _i in range(2):
        with pytest.raises(CircuitBreakerTestException):
            await breaker.call(failing_func)
        assert breaker.is_closed

    # Third failure should open the circuit
    with pytest.raises(CircuitBreakerTestException):
        await breaker.call(failing_func)
    assert breaker.is_open

    # Subsequent calls should fail fast
    with pytest.raises(CircuitBreakerError):
        await breaker.call(failing_func)


@pytest.mark.asyncio
async def test_circuit_breaker_half_open_recovery():
    """Test circuit breaker recovery through half-open state."""
    config = CircuitBreakerConfig(
        failure_threshold=2,
        recovery_timeout=0.1,  # 100ms for testing
        success_threshold=2,
    )
    breaker = CircuitBreaker("test_recovery", config)

    call_count = 0
    should_fail = True

    async def test_func():
        nonlocal call_count
        call_count += 1
        if should_fail:
            raise CircuitBreakerTestException("Failure")
        return "success"

    # Open the circuit
    for _ in range(2):
        with pytest.raises(CircuitBreakerTestException):
            await breaker.call(test_func)
    assert breaker.is_open

    # Wait for recovery timeout
    await asyncio.sleep(0.15)

    # Next call should attempt (half-open state)
    should_fail = False
    result = await breaker.call(test_func)
    assert result == "success"
    assert breaker.state == CircuitState.HALF_OPEN

    # Need one more success to close
    result = await breaker.call(test_func)
    assert result == "success"
    assert breaker.is_closed


@pytest.mark.asyncio
async def test_circuit_breaker_half_open_failure():
    """Test circuit breaker reopens on failure in half-open state."""
    config = CircuitBreakerConfig(
        failure_threshold=2,
        recovery_timeout=0.1,
    )
    breaker = CircuitBreaker("test_half_open_fail", config)

    async def failing_func():
        raise CircuitBreakerTestException("Failure")

    # Open the circuit
    for _ in range(2):
        with pytest.raises(CircuitBreakerTestException):
            await breaker.call(failing_func)
    assert breaker.is_open

    # Wait for recovery timeout
    await asyncio.sleep(0.15)

    # Failure in half-open should reopen immediately
    with pytest.raises(CircuitBreakerTestException):
        await breaker.call(failing_func)
    assert breaker.is_open


@pytest.mark.asyncio
async def test_circuit_breaker_ignored_exceptions():
    """Test that certain exceptions don't trigger circuit breaker."""
    config = CircuitBreakerConfig(
        failure_threshold=2,
        expected_exception=Exception,
        exclude_exceptions=(IgnoredException,),
    )
    breaker = CircuitBreaker("test_ignored", config)

    async def func_with_ignored_exception():
        raise IgnoredException("Should be ignored")

    # These exceptions shouldn't count
    for _ in range(5):
        with pytest.raises(IgnoredException):
            await breaker.call(func_with_ignored_exception)
        assert breaker.is_closed


@pytest.mark.asyncio
async def test_circuit_breaker_reset():
    """Test manual reset of circuit breaker."""
    config = CircuitBreakerConfig(failure_threshold=1)
    breaker = CircuitBreaker("test_reset", config)

    async def failing_func():
        raise CircuitBreakerTestException("Failure")

    # Open the circuit
    with pytest.raises(CircuitBreakerTestException):
        await breaker.call(failing_func)
    assert breaker.is_open

    # Manual reset
    await breaker.reset()
    assert breaker.is_closed

    # Should allow calls again
    async def success_func():
        return "success"

    result = await breaker.call(success_func)
    assert result == "success"


@pytest.mark.asyncio
async def test_circuit_breaker_decorator():
    """Test circuit breaker decorator."""
    call_count = 0

    @circuit_breaker(
        "test_decorator", CircuitBreakerConfig(failure_threshold=2)
    )
    async def decorated_func(should_fail=False):
        nonlocal call_count
        call_count += 1
        if should_fail:
            raise CircuitBreakerTestException("Failure")
        return "success"

    # Success calls
    result = await decorated_func()
    assert result == "success"

    # Open circuit with failures
    for _ in range(2):
        with pytest.raises(CircuitBreakerTestException):
            await decorated_func(should_fail=True)

    # Circuit should be open
    with pytest.raises(CircuitBreakerError):
        await decorated_func()


def test_get_circuit_breaker():
    """Test getting circuit breaker from registry."""
    # First call creates breaker
    breaker1 = get_circuit_breaker("test_registry")
    assert breaker1.name == "test_registry"

    # Second call returns same instance
    breaker2 = get_circuit_breaker("test_registry")
    assert breaker1 is breaker2

    # Different name creates different breaker
    breaker3 = get_circuit_breaker("test_registry_2")
    assert breaker3 is not breaker1


@pytest.mark.asyncio
async def test_circuit_breaker_concurrent_calls():
    """Test circuit breaker handles concurrent calls correctly."""
    config = CircuitBreakerConfig(
        failure_threshold=5,
        expected_exception=CircuitBreakerTestException,
    )
    breaker = CircuitBreaker("test_concurrent", config)

    failure_count = 0

    async def failing_func():
        nonlocal failure_count
        failure_count += 1
        if failure_count <= 5:
            raise CircuitBreakerTestException("Failure")
        return "success"

    # Run concurrent failing calls
    tasks = []
    for _ in range(10):
        tasks.append(breaker.call(failing_func))

    results = await asyncio.gather(*tasks, return_exceptions=True)

    # Should have some CircuitBreakerTestExceptions and some CircuitBreakerErrors
    test_exceptions = sum(
        1 for r in results if isinstance(r, CircuitBreakerTestException)
    )
    breaker_errors = sum(
        1 for r in results if isinstance(r, CircuitBreakerError)
    )

    # At least failure_threshold CircuitBreakerTestExceptions
    assert test_exceptions >= config.failure_threshold
    # Some calls should have been blocked
    assert breaker_errors > 0
    # Circuit should be open
    assert breaker.is_open


@pytest.mark.asyncio
async def test_circuit_breaker_success_resets_failures():
    """Test that successes reset failure count in closed state."""
    config = CircuitBreakerConfig(failure_threshold=3)
    breaker = CircuitBreaker("test_success_reset", config)

    async def sometimes_failing_func(fail=False):
        if fail:
            raise CircuitBreakerTestException("Failure")
        return "success"

    # Two failures
    for _ in range(2):
        with pytest.raises(CircuitBreakerTestException):
            await breaker.call(sometimes_failing_func, fail=True)

    # Success should reset failure count
    result = await breaker.call(sometimes_failing_func, fail=False)
    assert result == "success"
    assert breaker.is_closed

    # Can now fail 2 more times without opening
    for _ in range(2):
        with pytest.raises(CircuitBreakerTestException):
            await breaker.call(sometimes_failing_func, fail=True)
    assert breaker.is_closed

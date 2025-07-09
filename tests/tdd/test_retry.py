"""Tests for retry logic with exponential backoff."""

import asyncio
from unittest.mock import AsyncMock, MagicMock, patch

import httpx
import pytest

from biomcp.retry import (
    RetryableHTTPError,
    RetryConfig,
    calculate_delay,
    is_retryable_exception,
    is_retryable_status,
    retry_with_backoff,
    with_retry,
)


def test_calculate_delay_exponential_backoff():
    """Test that delay increases exponentially."""
    config = RetryConfig(initial_delay=1.0, exponential_base=2.0, jitter=False)

    # Test exponential increase
    assert calculate_delay(0, config) == 1.0  # 1 * 2^0
    assert calculate_delay(1, config) == 2.0  # 1 * 2^1
    assert calculate_delay(2, config) == 4.0  # 1 * 2^2
    assert calculate_delay(3, config) == 8.0  # 1 * 2^3


def test_calculate_delay_max_cap():
    """Test that delay is capped at max_delay."""
    config = RetryConfig(
        initial_delay=1.0, exponential_base=2.0, max_delay=5.0, jitter=False
    )

    # Test that delay is capped
    assert calculate_delay(0, config) == 1.0
    assert calculate_delay(1, config) == 2.0
    assert calculate_delay(2, config) == 4.0
    assert calculate_delay(3, config) == 5.0  # Capped at max_delay
    assert calculate_delay(10, config) == 5.0  # Still capped


def test_calculate_delay_with_jitter():
    """Test that jitter adds randomness to delay."""
    config = RetryConfig(initial_delay=10.0, jitter=True)

    # Generate multiple delays and check they're different
    delays = [calculate_delay(1, config) for _ in range(10)]

    # All should be around 20.0 (10 * 2^1) with jitter
    for delay in delays:
        assert 18.0 <= delay <= 22.0  # Within 10% jitter range

    # Should have some variation
    assert len(set(delays)) > 1


def test_is_retryable_exception():
    """Test exception retryability check."""
    config = RetryConfig(retryable_exceptions=(ConnectionError, TimeoutError))

    # Retryable exceptions
    assert is_retryable_exception(ConnectionError("test"), config)
    assert is_retryable_exception(TimeoutError("test"), config)

    # Non-retryable exceptions
    assert not is_retryable_exception(ValueError("test"), config)
    assert not is_retryable_exception(KeyError("test"), config)


def test_is_retryable_status():
    """Test HTTP status code retryability check."""
    config = RetryConfig(retryable_status_codes=(429, 502, 503, 504))

    # Retryable status codes
    assert is_retryable_status(429, config)
    assert is_retryable_status(502, config)
    assert is_retryable_status(503, config)
    assert is_retryable_status(504, config)

    # Non-retryable status codes
    assert not is_retryable_status(200, config)
    assert not is_retryable_status(404, config)
    assert not is_retryable_status(500, config)


@pytest.mark.asyncio
async def test_with_retry_decorator_success():
    """Test retry decorator with successful call."""
    call_count = 0

    @with_retry(RetryConfig(max_attempts=3))
    async def test_func():
        nonlocal call_count
        call_count += 1
        return "success"

    result = await test_func()
    assert result == "success"
    assert call_count == 1  # Should succeed on first try


@pytest.mark.asyncio
async def test_with_retry_decorator_eventual_success():
    """Test retry decorator with eventual success."""
    call_count = 0

    @with_retry(
        RetryConfig(
            max_attempts=3,
            initial_delay=0.01,  # Fast for testing
            retryable_exceptions=(ValueError,),
        )
    )
    async def test_func():
        nonlocal call_count
        call_count += 1
        if call_count < 3:
            raise ValueError("Transient error")
        return "success"

    result = await test_func()
    assert result == "success"
    assert call_count == 3


@pytest.mark.asyncio
async def test_with_retry_decorator_max_attempts_exceeded():
    """Test retry decorator when max attempts exceeded."""
    call_count = 0

    @with_retry(
        RetryConfig(
            max_attempts=3,
            initial_delay=0.01,
            retryable_exceptions=(ConnectionError,),
        )
    )
    async def test_func():
        nonlocal call_count
        call_count += 1
        raise ConnectionError("Persistent error")

    with pytest.raises(ConnectionError, match="Persistent error"):
        await test_func()

    assert call_count == 3


@pytest.mark.asyncio
async def test_with_retry_non_retryable_exception():
    """Test retry decorator with non-retryable exception."""
    call_count = 0

    @with_retry(
        RetryConfig(max_attempts=3, retryable_exceptions=(ConnectionError,))
    )
    async def test_func():
        nonlocal call_count
        call_count += 1
        raise ValueError("Non-retryable error")

    with pytest.raises(ValueError, match="Non-retryable error"):
        await test_func()

    assert call_count == 1  # Should not retry


@pytest.mark.asyncio
async def test_retry_with_backoff_function():
    """Test retry_with_backoff function."""
    call_count = 0

    async def test_func(value):
        nonlocal call_count
        call_count += 1
        if call_count < 2:
            raise TimeoutError("Timeout")
        return f"result: {value}"

    config = RetryConfig(
        max_attempts=3,
        initial_delay=0.01,
        retryable_exceptions=(TimeoutError,),
    )

    result = await retry_with_backoff(test_func, "test", config=config)
    assert result == "result: test"
    assert call_count == 2


def test_retryable_http_error():
    """Test RetryableHTTPError."""
    error = RetryableHTTPError(503, "Service Unavailable")
    assert error.status_code == 503
    assert error.message == "Service Unavailable"
    assert str(error) == "HTTP 503: Service Unavailable"


@pytest.mark.asyncio
async def test_retry_with_delay_progression():
    """Test that retries happen with correct delay progression."""
    call_times = []

    @with_retry(
        RetryConfig(
            max_attempts=3,
            initial_delay=0.1,
            exponential_base=2.0,
            jitter=False,
            retryable_exceptions=(ValueError,),
        )
    )
    async def test_func():
        call_times.append(asyncio.get_event_loop().time())
        if len(call_times) < 3:
            raise ValueError("Retry me")
        return "success"

    asyncio.get_event_loop().time()
    result = await test_func()

    assert result == "success"
    assert len(call_times) == 3

    # Check delays between attempts (allowing some tolerance)
    first_delay = call_times[1] - call_times[0]
    second_delay = call_times[2] - call_times[1]

    assert 0.08 <= first_delay <= 0.12  # ~0.1s
    assert 0.18 <= second_delay <= 0.22  # ~0.2s


@pytest.mark.asyncio
async def test_integration_with_http_client(monkeypatch):
    """Test retry integration with HTTP client."""
    from biomcp.http_client import call_http

    # Disable connection pooling for this test
    monkeypatch.setenv("BIOMCP_USE_CONNECTION_POOL", "false")

    # Test 1: Connection error retry
    with patch(
        "biomcp.http_client_simple.httpx.AsyncClient"
    ) as mock_client_class:
        mock_client = AsyncMock()
        mock_client_class.return_value = mock_client
        mock_client.aclose = AsyncMock()  # Mock aclose method

        # Simulate connection errors then success
        call_count = 0

        async def mock_get(*args, **kwargs):
            nonlocal call_count
            call_count += 1
            if call_count < 3:
                raise httpx.ConnectError("Connection failed")
            # Return success on third try
            mock_response = MagicMock()
            mock_response.status_code = 200
            mock_response.text = '{"result": "success"}'
            return mock_response

        mock_client.get = mock_get

        config = RetryConfig(
            max_attempts=3,
            initial_delay=0.01,
        )

        status, content = await call_http(
            "GET", "https://api.example.com/test", {}, retry_config=config
        )

        assert status == 200
        assert content == '{"result": "success"}'
        assert call_count == 3

    # Test 2: Timeout error retry
    with patch(
        "biomcp.http_client_simple.httpx.AsyncClient"
    ) as mock_client_class:
        mock_client = AsyncMock()
        mock_client_class.return_value = mock_client
        mock_client.aclose = AsyncMock()  # Mock aclose method

        # Simulate timeout errors
        mock_client.get.side_effect = httpx.TimeoutException(
            "Request timed out"
        )

        config = RetryConfig(
            max_attempts=2,
            initial_delay=0.01,
        )

        # This should raise TimeoutError after retries fail
        with pytest.raises(TimeoutError):
            await call_http(
                "GET", "https://api.example.com/test", {}, retry_config=config
            )

        assert mock_client.get.call_count == 2

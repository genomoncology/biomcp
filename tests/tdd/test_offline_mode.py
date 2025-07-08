"""Tests for offline mode functionality."""

import os
from unittest.mock import patch

import pytest

from biomcp.http_client import RequestError, request_api


@pytest.mark.asyncio
async def test_offline_mode_blocks_requests():
    """Test that offline mode prevents HTTP requests."""
    # Set offline mode
    with patch.dict(os.environ, {"BIOMCP_OFFLINE": "true"}):
        # Try to make a request
        result, error = await request_api(
            url="https://api.example.com/test",
            request={"test": "data"},
            cache_ttl=0,  # Disable caching for this test
        )

        # Should get an error
        assert result is None
        assert error is not None
        assert isinstance(error, RequestError)
        assert error.code == 503
        assert "Offline mode enabled" in error.message


@pytest.mark.asyncio
async def test_offline_mode_allows_cached_responses():
    """Test that offline mode still returns cached responses."""
    # First, cache a response (with offline mode disabled)
    with (
        patch.dict(os.environ, {"BIOMCP_OFFLINE": "false"}),
        patch("biomcp.http_client.call_http") as mock_call,
    ):
        mock_call.return_value = (200, '{"data": "cached"}')

        # Make a request to cache it
        result, error = await request_api(
            url="https://api.example.com/cached",
            request={"test": "data"},
            cache_ttl=3600,  # Cache for 1 hour
        )

        assert result == {"data": "cached"}
        assert error is None

    # Now enable offline mode
    with patch.dict(os.environ, {"BIOMCP_OFFLINE": "true"}):
        # Try to get the same request - should return cached result
        result, error = await request_api(
            url="https://api.example.com/cached",
            request={"test": "data"},
            cache_ttl=3600,
        )

        # Should get the cached response
        assert result == {"data": "cached"}
        assert error is None


@pytest.mark.asyncio
async def test_offline_mode_case_insensitive():
    """Test that offline mode environment variable is case insensitive."""
    test_values = ["TRUE", "True", "1", "yes", "YES", "Yes"]

    for value in test_values:
        with patch.dict(os.environ, {"BIOMCP_OFFLINE": value}):
            result, error = await request_api(
                url="https://api.example.com/test",
                request={"test": "data"},
                cache_ttl=0,
            )

            assert result is None
            assert error is not None
            assert error.code == 503
            assert "Offline mode enabled" in error.message


@pytest.mark.asyncio
async def test_offline_mode_disabled_by_default():
    """Test that offline mode is disabled by default."""
    # Clear the environment variable
    with (
        patch.dict(os.environ, {}, clear=True),
        patch("biomcp.http_client.call_http") as mock_call,
    ):
        mock_call.return_value = (200, '{"data": "response"}')

        result, error = await request_api(
            url="https://api.example.com/test",
            request={"test": "data"},
            cache_ttl=0,
        )

        # Should make the request successfully
        assert result == {"data": "response"}
        assert error is None
        mock_call.assert_called_once()


@pytest.mark.asyncio
async def test_offline_mode_with_endpoint_tracking():
    """Test that offline mode works with endpoint tracking."""
    with patch.dict(os.environ, {"BIOMCP_OFFLINE": "true"}):
        result, error = await request_api(
            url="https://www.ncbi.nlm.nih.gov/research/pubtator3-api/search/",
            request={"text": "BRAF"},
            endpoint_key="pubtator3_search",
            cache_ttl=0,
        )

        assert result is None
        assert error is not None
        assert error.code == 503
        assert "pubtator3-api/search/" in error.message

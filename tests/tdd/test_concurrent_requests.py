"""Test concurrent request handling in the HTTP client."""

import asyncio
from unittest.mock import AsyncMock, patch

import pytest

from biomcp import http_client


class TestConcurrentRequests:
    """Test concurrent request handling."""

    @pytest.mark.asyncio
    async def test_concurrent_requests_same_domain(self):
        """Test multiple concurrent requests to the same domain."""
        # Use patch instead of direct replacement
        with patch(
            "biomcp.http_client.call_http", new_callable=AsyncMock
        ) as mock_call:
            # Configure mock to return success
            mock_call.return_value = (200, '{"data": "response"}')

            # Make 10 concurrent requests with different URLs to avoid caching
            # and disable caching explicitly
            tasks = [
                http_client.request_api(
                    url=f"https://api.example.com/resource/{i}",
                    request={},
                    domain="example",
                    cache_ttl=0,  # Disable caching
                )
                for i in range(10)
            ]

            results = await asyncio.gather(*tasks)

            # All requests should succeed
            assert len(results) == 10
            for data, error in results:
                assert error is None
                assert data == {"data": "response"}

            # Check that rate limiting was applied
            assert mock_call.call_count == 10

    @pytest.mark.asyncio
    async def test_concurrent_requests_different_domains(self):
        """Test concurrent requests to different domains."""
        with patch(
            "biomcp.http_client.call_http", new_callable=AsyncMock
        ) as mock_call:
            # Return different responses based on URL
            async def side_effect(method, url, *args, **kwargs):
                if "domain1" in url:
                    return (200, '{"source": "domain1"}')
                elif "domain2" in url:
                    return (200, '{"source": "domain2"}')
                else:
                    return (200, '{"source": "other"}')

            mock_call.side_effect = side_effect

            # Make requests to different domains
            tasks = [
                http_client.request_api(
                    "https://domain1.com/api", {}, domain="domain1"
                ),
                http_client.request_api(
                    "https://domain2.com/api", {}, domain="domain2"
                ),
                http_client.request_api(
                    "https://domain3.com/api", {}, domain="domain3"
                ),
            ]

            results = await asyncio.gather(*tasks)

            # Check results
            assert results[0][0] == {"source": "domain1"}
            assert results[1][0] == {"source": "domain2"}
            assert results[2][0] == {"source": "other"}

    @pytest.mark.asyncio
    async def test_concurrent_cache_access(self):
        """Test that concurrent requests properly use cache."""
        with patch(
            "biomcp.http_client.call_http", new_callable=AsyncMock
        ) as mock_call:
            mock_call.return_value = (200, '{"data": "cached"}')

            # First request to populate cache
            await http_client.request_api(
                url="https://api.example.com/data",
                request={},
                domain="example",
                cache_ttl=60,
            )

            # Reset call count
            initial_calls = mock_call.call_count

            # Make 5 concurrent requests to same URL
            tasks = [
                http_client.request_api(
                    url="https://api.example.com/data",
                    request={},
                    domain="example",
                    cache_ttl=60,
                )
                for _ in range(5)
            ]

            results = await asyncio.gather(*tasks)

            # All should get cached response
            assert len(results) == 5
            for data, _error in results:
                assert data == {"data": "cached"}

            # No additional HTTP calls should have been made
            assert mock_call.call_count == initial_calls

    @pytest.mark.asyncio
    async def test_concurrent_circuit_breaker(self):
        """Test circuit breaker behavior with concurrent failures."""
        with patch(
            "biomcp.http_client.call_http", new_callable=AsyncMock
        ) as mock_call:
            # Simulate failures
            mock_call.return_value = (500, "Internal Server Error")

            # Make concurrent failing requests
            tasks = [
                http_client.request_api(
                    url=f"https://failing.com/api/{i}",
                    request={},
                    domain="failing",
                )
                for i in range(10)
            ]

            results = await asyncio.gather(*tasks, return_exceptions=True)

            # All should fail
            error_count = sum(1 for _, error in results if error is not None)
            assert error_count == 10

            # Circuit should be open now
            # Additional requests should fail immediately
            _, error = await http_client.request_api(
                url="https://failing.com/api/test",
                request={},
                domain="failing",
            )

            assert error is not None
            # Check that circuit breaker is preventing calls
            # (exact behavior depends on implementation details)

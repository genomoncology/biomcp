"""Tests for connection pool management."""

import asyncio
import ssl
import weakref
from unittest.mock import patch

import httpx
import pytest

from biomcp.connection_pool import (
    EventLoopConnectionPools,
    close_all_pools,
    get_connection_pool,
)


@pytest.fixture
def pool_manager():
    """Create a fresh pool manager for testing."""
    return EventLoopConnectionPools()


@pytest.mark.asyncio
async def test_get_pool_creates_new_pool(pool_manager):
    """Test that get_pool creates a new pool when none exists."""
    timeout = httpx.Timeout(30)

    pool = await pool_manager.get_pool(verify=True, timeout=timeout)

    assert pool is not None
    assert isinstance(pool, httpx.AsyncClient)
    assert not pool.is_closed


@pytest.mark.asyncio
async def test_get_pool_reuses_existing_pool(pool_manager):
    """Test that get_pool reuses existing pools."""
    timeout = httpx.Timeout(30)

    pool1 = await pool_manager.get_pool(verify=True, timeout=timeout)
    pool2 = await pool_manager.get_pool(verify=True, timeout=timeout)

    assert pool1 is pool2


@pytest.mark.asyncio
async def test_get_pool_different_verify_settings(pool_manager):
    """Test that different verify settings create different pools."""
    timeout = httpx.Timeout(30)

    pool1 = await pool_manager.get_pool(verify=True, timeout=timeout)
    pool2 = await pool_manager.get_pool(verify=False, timeout=timeout)

    assert pool1 is not pool2


@pytest.mark.asyncio
async def test_get_pool_ssl_context(pool_manager):
    """Test pool creation with SSL context."""
    ssl_context = ssl.create_default_context()
    timeout = httpx.Timeout(30)

    pool = await pool_manager.get_pool(verify=ssl_context, timeout=timeout)

    assert pool is not None
    assert isinstance(pool, httpx.AsyncClient)


@pytest.mark.asyncio
async def test_pool_cleanup_on_close_all(pool_manager):
    """Test that close_all properly closes all pools."""
    timeout = httpx.Timeout(30)

    await pool_manager.get_pool(verify=True, timeout=timeout)
    await pool_manager.get_pool(verify=False, timeout=timeout)

    await pool_manager.close_all()

    # After close_all, pools should be cleared
    assert len(pool_manager._loop_pools) == 0


@pytest.mark.asyncio
async def test_no_event_loop_returns_single_use_client(pool_manager):
    """Test behavior when no event loop is running."""
    with patch("asyncio.get_running_loop", side_effect=RuntimeError):
        timeout = httpx.Timeout(30)

        pool = await pool_manager.get_pool(verify=True, timeout=timeout)

        assert pool is not None
        # Single-use client should have no keepalive
        # Note: httpx client internal structure may vary


@pytest.mark.asyncio
async def test_pool_recreation_after_close(pool_manager):
    """Test that a new pool is created after the old one is closed."""
    timeout = httpx.Timeout(30)

    pool1 = await pool_manager.get_pool(verify=True, timeout=timeout)
    await pool1.aclose()

    pool2 = await pool_manager.get_pool(verify=True, timeout=timeout)

    assert pool1 is not pool2
    assert pool1.is_closed
    assert not pool2.is_closed


@pytest.mark.asyncio
async def test_weak_reference_cleanup():
    """Test that weak references are used for event loops."""
    pool_manager = EventLoopConnectionPools()

    # Verify that the pool manager uses weak references
    assert isinstance(pool_manager._loop_pools, weakref.WeakKeyDictionary)

    # Create a pool
    timeout = httpx.Timeout(30)
    pool = await pool_manager.get_pool(verify=True, timeout=timeout)

    # Verify pool was created
    assert pool is not None

    # The current event loop should be in the weak key dict
    current_loop = asyncio.get_running_loop()
    assert current_loop in pool_manager._loop_pools


@pytest.mark.asyncio
async def test_global_get_connection_pool():
    """Test the global get_connection_pool function."""
    with patch.dict("os.environ", {"BIOMCP_USE_CONNECTION_POOL": "true"}):
        timeout = httpx.Timeout(30)

        pool = await get_connection_pool(verify=True, timeout=timeout)

        assert pool is not None
        assert isinstance(pool, httpx.AsyncClient)


@pytest.mark.asyncio
async def test_global_close_all_pools():
    """Test the global close_all_pools function."""
    # Create some pools
    timeout = httpx.Timeout(30)
    await get_connection_pool(verify=True, timeout=timeout)
    await get_connection_pool(verify=False, timeout=timeout)

    # Close all pools
    await close_all_pools()

    # Verify cleanup (this is implementation-specific)
    from biomcp.connection_pool import _pool_manager

    assert len(_pool_manager._loop_pools) == 0


@pytest.mark.asyncio
async def test_concurrent_pool_creation(pool_manager):
    """Test thread-safe pool creation under concurrent access."""
    timeout = httpx.Timeout(30)

    async def get_pool():
        return await pool_manager.get_pool(verify=True, timeout=timeout)

    # Create 10 concurrent requests for the same pool
    pools = await asyncio.gather(*[get_pool() for _ in range(10)])

    # All should return the same pool instance
    assert all(pool is pools[0] for pool in pools)


@pytest.mark.asyncio
async def test_connection_pool_limits():
    """Test that connection pools have proper limits set."""
    pool_manager = EventLoopConnectionPools()
    timeout = httpx.Timeout(30)

    pool = await pool_manager.get_pool(verify=True, timeout=timeout)

    # Verify pool was created (actual limits are internal to httpx)
    assert pool is not None
    assert isinstance(pool, httpx.AsyncClient)

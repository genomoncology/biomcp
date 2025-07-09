"""Connection pool manager with proper event loop lifecycle management.

This module provides HTTP connection pooling that is properly integrated
with asyncio event loops. It ensures that connection pools are:
- Created per event loop to avoid cross-loop usage
- Automatically cleaned up when event loops are garbage collected
- Reused across requests for better performance

Key Features:
- Event loop isolation - each loop gets its own pools
- Weak references prevent memory leaks
- Automatic cleanup on loop destruction
- Thread-safe pool management

Example:
    ```python
    # Get a connection pool for the current event loop
    pool = await get_connection_pool(verify=True, timeout=httpx.Timeout(30))

    # Use the pool for multiple requests (no need to close)
    response = await pool.get("https://api.example.com/data")
    ```

Environment Variables:
    BIOMCP_USE_CONNECTION_POOL: Enable/disable pooling (default: "true")
"""

import asyncio
import ssl
import weakref

# NOTE: httpx import is allowed in this file for connection pooling infrastructure
import httpx


class EventLoopConnectionPools:
    """Manages connection pools per event loop.

    This class ensures that each asyncio event loop has its own set of
    connection pools, preventing cross-loop contamination and ensuring
    proper cleanup when event loops are destroyed.

    Attributes:
        _loop_pools: Weak key dictionary mapping event loops to their pools
        _lock: Asyncio lock for thread-safe pool creation
    """

    def __init__(self):
        # Use weak references to avoid keeping event loops alive
        self._loop_pools: weakref.WeakKeyDictionary = (
            weakref.WeakKeyDictionary()
        )
        self._lock = asyncio.Lock()

    async def get_pool(
        self, verify: ssl.SSLContext | str | bool, timeout: httpx.Timeout
    ) -> httpx.AsyncClient:
        """Get or create a connection pool for the current event loop."""
        try:
            loop = asyncio.get_running_loop()
        except RuntimeError:
            # No event loop running, return a single-use client
            return self._create_client(verify, timeout, pooled=False)

        # Get or create pools dict for this event loop
        async with self._lock:
            if loop not in self._loop_pools:
                self._loop_pools[loop] = {}
                # Register cleanup when loop is garbage collected
                self._register_loop_cleanup(loop)

            pools = self._loop_pools[loop]
            pool_key = self._get_pool_key(verify)

            # Check if we have a valid pool
            if pool_key in pools and not pools[pool_key].is_closed:
                return pools[pool_key]

            # Create new pool
            client = self._create_client(verify, timeout, pooled=True)
            pools[pool_key] = client
            return client

    def _get_pool_key(self, verify: ssl.SSLContext | str | bool) -> str:
        """Generate a key for the connection pool."""
        if isinstance(verify, ssl.SSLContext):
            return f"ssl_{id(verify)}"
        return str(verify)

    def _create_client(
        self,
        verify: ssl.SSLContext | str | bool,
        timeout: httpx.Timeout,
        pooled: bool = True,
    ) -> httpx.AsyncClient:
        """Create a new HTTP client."""
        if pooled:
            limits = httpx.Limits(
                max_keepalive_connections=20,
                max_connections=100,
                keepalive_expiry=30,
            )
        else:
            # Single-use client
            limits = httpx.Limits(max_keepalive_connections=0)

        return httpx.AsyncClient(
            verify=verify,
            http2=False,  # HTTP/2 can add overhead
            timeout=timeout,
            limits=limits,
        )

    def _register_loop_cleanup(self, loop: asyncio.AbstractEventLoop):
        """Register cleanup when event loop is garbage collected."""
        # Store pools to close when loop is garbage collected
        # Note: We can't create weak references to dicts, so we'll
        # clean up pools when the loop itself is garbage collected

        def cleanup():
            # Get pools for this loop if they still exist
            pools = self._loop_pools.get(loop, {})
            if pools:
                # Try to close all clients gracefully
                for client in list(pools.values()):
                    if client and not client.is_closed:
                        # Close synchronously since loop might be gone
                        import contextlib

                        with contextlib.suppress(Exception):
                            client._transport.close()

        # Register finalizer on the loop itself
        weakref.finalize(loop, cleanup)

    async def close_all(self):
        """Close all connection pools."""
        async with self._lock:
            all_clients = []
            for pools in self._loop_pools.values():
                all_clients.extend(pools.values())

            # Close all clients
            close_tasks = []
            for client in all_clients:
                if client and not client.is_closed:
                    close_tasks.append(client.aclose())

            if close_tasks:
                await asyncio.gather(*close_tasks, return_exceptions=True)

            self._loop_pools.clear()


# Global instance
_pool_manager = EventLoopConnectionPools()


async def get_connection_pool(
    verify: ssl.SSLContext | str | bool,
    timeout: httpx.Timeout,
) -> httpx.AsyncClient:
    """Get a connection pool for the current event loop."""
    return await _pool_manager.get_pool(verify, timeout)


async def close_all_pools():
    """Close all connection pools."""
    await _pool_manager.close_all()

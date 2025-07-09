"""Request batching utility for combining multiple small requests.

This module provides a request batcher that accumulates multiple requests
and processes them together in batches, reducing the number of API calls
and improving performance for bulk operations.

Key Features:
- Automatic batching based on size or time threshold
- Configurable batch size and timeout
- Thread-safe request accumulation
- Error propagation to individual requests

Example:
    ```python
    async def batch_api_call(params_list):
        # Make a single API call with multiple parameters
        return await api.bulk_request(params_list)

    batcher = RequestBatcher(
        batch_func=batch_api_call,
        batch_size=10,
        batch_timeout=0.1
    )

    # Individual requests are automatically batched
    result1 = await batcher.request({"id": 1})
    result2 = await batcher.request({"id": 2})
    ```
"""

import asyncio
from collections.abc import Callable, Coroutine
from typing import Any, TypeVar

T = TypeVar("T")


class RequestBatcher:
    """Batches multiple requests together to reduce overhead.

    This is particularly useful for APIs that support batch operations
    or when network latency dominates over processing time.

    The batcher accumulates requests until either:
    1. The batch size threshold is reached
    2. The batch timeout expires

    At which point all accumulated requests are processed together.
    """

    def __init__(
        self,
        batch_func: Callable[[list[Any]], Coroutine[Any, Any, list[Any]]],
        batch_size: int = 10,
        batch_timeout: float = 0.05,  # 50ms
    ):
        """Initialize the batcher.

        Args:
            batch_func: Async function that processes a batch of requests
            batch_size: Maximum number of requests to batch together
            batch_timeout: Maximum time to wait for batch to fill (seconds)
        """
        self.batch_func = batch_func
        self.batch_size = batch_size
        self.batch_timeout = batch_timeout
        self.pending_requests: list[tuple[Any, asyncio.Future]] = []
        self.batch_task: asyncio.Task | None = None
        self._lock = asyncio.Lock()

    async def request(self, params: Any) -> Any:
        """Add a request to the batch and wait for result."""
        future: asyncio.Future[Any] = asyncio.Future()

        async with self._lock:
            self.pending_requests.append((params, future))

            # Check if we should flush immediately
            if len(self.pending_requests) >= self.batch_size:
                await self._flush_batch()
            elif not self.batch_task or self.batch_task.done():
                # Start a timer to flush the batch
                self.batch_task = asyncio.create_task(self._batch_timer())

        return await future

    async def _batch_timer(self):
        """Timer that flushes the batch after timeout."""
        await asyncio.sleep(self.batch_timeout)
        async with self._lock:
            await self._flush_batch()

    async def _flush_batch(self):
        """Process all pending requests as a batch."""
        if not self.pending_requests:
            return

        # Extract current batch
        batch = self.pending_requests.copy()
        self.pending_requests.clear()

        # Cancel timer if running
        if self.batch_task and not self.batch_task.done():
            self.batch_task.cancel()

        # Process batch
        try:
            params_list = [params for params, _ in batch]
            results = await self.batch_func(params_list)

            # Distribute results to futures
            for i, (_, future) in enumerate(batch):
                if not future.done():
                    if i < len(results):
                        future.set_result(results[i])
                    else:
                        future.set_exception(
                            Exception(f"No result for request at index {i}")
                        )
        except Exception as e:
            # Propagate error to all futures
            for _, future in batch:
                if not future.done():
                    future.set_exception(e)


# Example usage for autocomplete batching
async def batch_autocomplete_requests(requests: list[dict]) -> list[Any]:
    """Process multiple autocomplete requests in parallel.

    This is an example implementation that could be used to batch
    autocomplete requests more efficiently.
    """
    from .articles.autocomplete import EntityRequest, autocomplete

    tasks = []
    for req in requests:
        entity_req = EntityRequest(**req)
        tasks.append(autocomplete(entity_req))

    return await asyncio.gather(*tasks)

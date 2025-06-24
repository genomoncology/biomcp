"""Metrics and monitoring utilities."""

import asyncio
import logging
import time
from collections.abc import Callable
from functools import wraps
from typing import Any, TypeVar, cast

logger = logging.getLogger(__name__)

T = TypeVar("T")


def track_api_call(api_name: str):
    """Track API call metrics.

    Args:
        api_name: Name of the API being called

    Returns:
        Decorator function
    """

    def decorator(func: Callable[..., T]) -> Callable[..., T]:
        @wraps(func)
        async def async_wrapper(*args: Any, **kwargs: Any) -> T:
            start_time = time.time()
            try:
                result = await func(*args, **kwargs)  # type: ignore[misc]
                duration = time.time() - start_time
                logger.info(
                    f"{api_name} call succeeded",
                    extra={
                        "api": api_name,
                        "duration": duration,
                        "status": "success",
                    },
                )
                return result
            except Exception as e:
                duration = time.time() - start_time
                logger.error(
                    f"{api_name} call failed: {e}",
                    extra={
                        "api": api_name,
                        "duration": duration,
                        "status": "error",
                        "error_type": type(e).__name__,
                    },
                )
                raise

        @wraps(func)
        def sync_wrapper(*args: Any, **kwargs: Any) -> T:
            start_time = time.time()
            try:
                result = func(*args, **kwargs)
                duration = time.time() - start_time
                logger.info(
                    f"{api_name} call succeeded",
                    extra={
                        "api": api_name,
                        "duration": duration,
                        "status": "success",
                    },
                )
                return result
            except Exception as e:
                duration = time.time() - start_time
                logger.error(
                    f"{api_name} call failed: {e}",
                    extra={
                        "api": api_name,
                        "duration": duration,
                        "status": "error",
                        "error_type": type(e).__name__,
                    },
                )
                raise

        # Return appropriate wrapper based on function type
        if asyncio.iscoroutinefunction(func):
            return cast(Callable[..., T], async_wrapper)
        else:
            return cast(Callable[..., T], sync_wrapper)

    return decorator

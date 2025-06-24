"""Simple rate limiting utilities for API calls."""

import asyncio
import time
from collections import defaultdict


class RateLimiter:
    """Simple token bucket rate limiter."""

    def __init__(self, rate: int = 10, per_seconds: int = 1):
        """Initialize rate limiter.

        Args:
            rate: Number of allowed requests
            per_seconds: Time window in seconds
        """
        self.rate = rate
        self.per_seconds = per_seconds
        self.allowance: dict[str, float] = defaultdict(lambda: float(rate))
        self.last_check: dict[str, float] = defaultdict(float)
        self._lock = asyncio.Lock()

    async def check_rate_limit(
        self, key: str = "default"
    ) -> tuple[bool, float | None]:
        """Check if request is allowed under rate limit.

        Args:
            key: Identifier for rate limit bucket

        Returns:
            Tuple of (allowed, wait_time_if_not_allowed)
        """
        async with self._lock:
            current = time.time()
            time_passed = current - self.last_check[key]
            self.last_check[key] = current

            # Replenish tokens
            self.allowance[key] += time_passed * (self.rate / self.per_seconds)

            # Cap at maximum rate
            if self.allowance[key] > self.rate:
                self.allowance[key] = float(self.rate)

            # Check if request allowed
            if self.allowance[key] >= 1.0:
                self.allowance[key] -= 1.0
                return True, None
            else:
                # Calculate wait time
                wait_time = (1.0 - self.allowance[key]) * (
                    self.per_seconds / self.rate
                )
                return False, wait_time

    async def wait_if_needed(self, key: str = "default") -> None:
        """Wait if rate limited before allowing request."""
        allowed, wait_time = await self.check_rate_limit(key)
        if not allowed and wait_time:
            await asyncio.sleep(wait_time)


# Global rate limiter for cBioPortal API
# Conservative: 5 requests per second
cbioportal_limiter = RateLimiter(rate=5, per_seconds=1)

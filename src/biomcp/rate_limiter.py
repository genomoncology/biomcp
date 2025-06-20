"""Rate limiting implementation for BioMCP API calls."""

import asyncio
import time
from collections import defaultdict
from contextlib import asynccontextmanager

from .constants import (
    DEFAULT_BURST_SIZE,
    DEFAULT_RATE_LIMIT_PER_SECOND,
)
from .exceptions import BioMCPError


class RateLimitExceeded(BioMCPError):
    """Raised when rate limit is exceeded."""

    def __init__(self, domain: str, limit: int, window: int):
        message = f"Rate limit exceeded for {domain}: {limit} requests per {window} seconds"
        super().__init__(
            message, {"domain": domain, "limit": limit, "window": window}
        )


class RateLimiter:
    """Token bucket rate limiter implementation."""

    def __init__(
        self,
        requests_per_second: float = DEFAULT_RATE_LIMIT_PER_SECOND,
        burst_size: int = DEFAULT_BURST_SIZE,
    ):
        """Initialize rate limiter.

        Args:
            requests_per_second: Sustained request rate
            burst_size: Maximum burst capacity
        """
        self.rate = requests_per_second
        self.burst_size = burst_size
        self.tokens = float(burst_size)
        self.last_update = time.monotonic()
        self._lock = asyncio.Lock()

    async def acquire(self, tokens: int = 1) -> None:
        """Acquire tokens from the bucket."""
        async with self._lock:
            now = time.monotonic()
            elapsed = now - self.last_update
            self.last_update = now

            # Add tokens based on elapsed time
            self.tokens = min(
                self.burst_size, self.tokens + elapsed * self.rate
            )

            if self.tokens < tokens:
                # Calculate wait time
                wait_time = (tokens - self.tokens) / self.rate
                await asyncio.sleep(wait_time)
                self.tokens = 0
            else:
                self.tokens -= tokens

    @asynccontextmanager
    async def limit(self):
        """Context manager for rate limiting."""
        await self.acquire()
        yield


class DomainRateLimiter:
    """Rate limiter with per-domain limits."""

    def __init__(self, default_rps: float = 10.0, default_burst: int = 20):
        """Initialize domain rate limiter.

        Args:
            default_rps: Default requests per second
            default_burst: Default burst size
        """
        self.default_rps = default_rps
        self.default_burst = default_burst
        self.limiters: dict[str, RateLimiter] = {}
        self.domain_configs = {
            "article": {"rps": 20.0, "burst": 40},  # PubMed can handle more
            "trial": {"rps": 10.0, "burst": 20},  # ClinicalTrials.gov standard
            "variant": {"rps": 15.0, "burst": 30},  # MyVariant.info moderate
            "thinking": {"rps": 50.0, "burst": 100},  # Local processing
        }

    def get_limiter(self, domain: str) -> RateLimiter:
        """Get or create rate limiter for domain."""
        if domain not in self.limiters:
            config = self.domain_configs.get(domain, {})
            rps = config.get("rps", self.default_rps)
            burst = config.get("burst", self.default_burst)
            self.limiters[domain] = RateLimiter(rps, int(burst))
        return self.limiters[domain]

    @asynccontextmanager
    async def limit(self, domain: str):
        """Rate limit context manager for a domain."""
        limiter = self.get_limiter(domain)
        async with limiter.limit():
            yield


class SlidingWindowRateLimiter:
    """Sliding window rate limiter for user/IP based limiting."""

    def __init__(self, requests: int = 100, window_seconds: int = 60):
        """Initialize sliding window rate limiter.

        Args:
            requests: Maximum requests per window
            window_seconds: Window size in seconds
        """
        self.max_requests = requests
        self.window_seconds = window_seconds
        self.requests: dict[str, list[float]] = defaultdict(list)
        self._lock = asyncio.Lock()

    async def check_limit(self, key: str) -> bool:
        """Check if request is allowed for key."""
        async with self._lock:
            now = time.time()
            cutoff = now - self.window_seconds

            # Remove old requests
            self.requests[key] = [
                req_time
                for req_time in self.requests[key]
                if req_time > cutoff
            ]

            # Check limit
            if len(self.requests[key]) >= self.max_requests:
                return False

            # Add current request
            self.requests[key].append(now)
            return True

    async def acquire(self, key: str) -> None:
        """Acquire permission to make request."""
        if not await self.check_limit(key):
            raise RateLimitExceeded(
                key, self.max_requests, self.window_seconds
            )


# Global instances
domain_limiter = DomainRateLimiter()
user_limiter = SlidingWindowRateLimiter(
    requests=1000, window_seconds=3600
)  # 1000 req/hour


async def rate_limit_domain(domain: str) -> None:
    """Apply rate limiting for a domain."""
    async with domain_limiter.limit(domain):
        pass


async def rate_limit_user(user_id: str | None = None) -> None:
    """Apply rate limiting for a user."""
    if user_id:
        await user_limiter.acquire(user_id)

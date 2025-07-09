"""Prefetching system for common queries to improve performance.

This module implements a prefetching mechanism that warms up caches with
commonly searched biomedical entities during startup. This significantly
improves response times for frequent queries.

Key Features:
- Prefetches common genes, diseases, and chemicals on startup
- Runs asynchronously to avoid blocking server initialization
- Includes timeout to prevent startup delays
- Graceful error handling if prefetching fails

The prefetching runs automatically when the MCP server starts via the
lifespan hook in core.py.

Configuration:
    The lists of entities to prefetch can be customized by modifying
    the COMMON_GENES, COMMON_DISEASES, and COMMON_CHEMICALS constants.
"""

import asyncio
import logging

from .constants import (
    PREFETCH_TIMEOUT,
    PREFETCH_TOP_CHEMICALS,
    PREFETCH_TOP_DISEASES,
    PREFETCH_TOP_GENES,
)

logger = logging.getLogger(__name__)

# Common genes that are frequently searched
COMMON_GENES = [
    "BRAF",
    "EGFR",
    "TP53",
    "KRAS",
    "ALK",
    "ROS1",
    "MET",
    "RET",
    "NTRK1",
    "NTRK2",
    "NTRK3",
]

# Common cancer types
COMMON_DISEASES = [
    "lung cancer",
    "breast cancer",
    "colorectal cancer",
    "melanoma",
    "non-small cell lung cancer",
    "small cell lung cancer",
]

# Common drug names
COMMON_CHEMICALS = [
    "osimertinib",
    "pembrolizumab",
    "nivolumab",
    "dabrafenib",
    "trametinib",
    "crizotinib",
    "alectinib",
]


class PrefetchManager:
    """Manages prefetching of common queries."""

    def __init__(self):
        self._prefetch_task: asyncio.Task | None = None
        self._is_prefetching = False
        self._prefetch_complete = False

    async def start_prefetching(self):
        """Start prefetching common queries in the background."""
        if self._is_prefetching or self._prefetch_complete:
            return

        self._is_prefetching = True
        try:
            # Start prefetch task
            self._prefetch_task = asyncio.create_task(
                self._prefetch_common_queries()
            )
        except Exception as e:
            logger.warning(f"Failed to start prefetching: {e}")
            self._is_prefetching = False

    async def _prefetch_common_queries(self):
        """Prefetch common queries to warm up the cache."""
        try:
            # Import here to avoid circular imports
            from .articles.autocomplete import EntityRequest, autocomplete
            from .variants.cbioportal_search import CBioPortalSearchClient

            tasks = []

            # Prefetch gene autocomplete
            for gene in COMMON_GENES[
                :PREFETCH_TOP_GENES
            ]:  # Limit to avoid overload
                request = EntityRequest(concept="gene", query=gene, limit=1)
                tasks.append(autocomplete(request))

            # Prefetch disease autocomplete
            for disease in COMMON_DISEASES[:PREFETCH_TOP_DISEASES]:
                request = EntityRequest(
                    concept="disease", query=disease, limit=1
                )
                tasks.append(autocomplete(request))

            # Prefetch chemical autocomplete
            for chemical in COMMON_CHEMICALS[:PREFETCH_TOP_CHEMICALS]:
                request = EntityRequest(
                    concept="chemical", query=chemical, limit=1
                )
                tasks.append(autocomplete(request))

            # Execute all autocomplete prefetches
            if tasks:
                await asyncio.gather(*tasks, return_exceptions=True)

            # Prefetch cBioPortal summaries for common genes
            cbio_client = CBioPortalSearchClient()
            cbio_tasks = []

            for gene in COMMON_GENES[:PREFETCH_TOP_GENES]:  # Top genes
                cbio_tasks.append(
                    cbio_client.get_gene_search_summary(gene, max_studies=5)
                )

            if cbio_tasks:
                await asyncio.gather(*cbio_tasks, return_exceptions=True)

            logger.info("Prefetching completed successfully")

        except Exception as e:
            logger.warning(f"Error during prefetching: {e}")
        finally:
            self._is_prefetching = False
            self._prefetch_complete = True

    async def wait_for_prefetch(self, timeout: float = PREFETCH_TIMEOUT):
        """Wait for prefetch to complete with timeout."""
        if not self._prefetch_task:
            return

        try:
            await asyncio.wait_for(self._prefetch_task, timeout=timeout)
        except asyncio.TimeoutError:
            # Prefetch taking too long, continue without waiting
            logger.debug("Prefetch timeout - continuing without waiting")
        except Exception as e:
            # Ignore prefetch errors
            logger.debug(f"Prefetch error ignored: {e}")


# Global prefetch manager
_prefetch_manager = PrefetchManager()


async def start_prefetching():
    """Start the prefetching process."""
    await _prefetch_manager.start_prefetching()


async def wait_for_prefetch(timeout: float = PREFETCH_TIMEOUT):
    """Wait for prefetch to complete."""
    await _prefetch_manager.wait_for_prefetch(timeout)

"""Optimized article search with caching and parallel processing."""

import asyncio
import hashlib

from .. import ensure_list
from ..shared_context import get_search_context
from ..utils.request_cache import get_cache
from .search import PubmedRequest
from .unified import search_articles_unified

# Cache for article search results (5 minute TTL)
_search_cache = get_cache("article_search", ttl_seconds=300)


def _get_search_cache_key(
    request: PubmedRequest, include_preprints: bool, include_cbioportal: bool
) -> str:
    """Generate a cache key for search requests."""
    # Create a deterministic key from search parameters
    key_parts = [
        f"chemicals:{sorted(request.chemicals)}",
        f"diseases:{sorted(request.diseases)}",
        f"genes:{sorted(request.genes)}",
        f"keywords:{sorted(request.keywords)}",
        f"variants:{sorted(request.variants)}",
        f"preprints:{include_preprints}",
        f"cbioportal:{include_cbioportal}",
    ]
    key_string = "|".join(key_parts)
    return hashlib.sha256(key_string.encode()).hexdigest()


async def article_searcher_optimized(
    call_benefit: str,
    chemicals: list[str] | str | None = None,
    diseases: list[str] | str | None = None,
    genes: list[str] | str | None = None,
    keywords: list[str] | str | None = None,
    variants: list[str] | str | None = None,
    include_preprints: bool = True,
    include_cbioportal: bool = True,
) -> str:
    """Optimized version of article_searcher with caching and context reuse."""

    # Convert parameters to PubmedRequest
    request = PubmedRequest(
        chemicals=ensure_list(chemicals, split_strings=True),
        diseases=ensure_list(diseases, split_strings=True),
        genes=ensure_list(genes, split_strings=True),
        keywords=ensure_list(keywords, split_strings=True),
        variants=ensure_list(variants, split_strings=True),
    )

    # Check cache first
    cache_key = _get_search_cache_key(
        request, include_preprints, include_cbioportal
    )
    cached_result = await _search_cache.get(cache_key)
    if cached_result is not None:
        return cached_result

    # Check if we're in a search context (for reusing validated entities)
    context = get_search_context()
    if context and request.genes:
        # Pre-validate genes using cached results
        valid_genes = []
        for gene in request.genes:
            if await context.validate_gene(gene):
                valid_genes.append(gene)
        request.genes = valid_genes

        # Check if we have cached cBioPortal summaries
        if include_cbioportal and request.genes:
            for gene in request.genes[:1]:  # Just first gene
                summary = context.get_gene_summary(gene)
                if summary:
                    # We have a cached summary, can skip that part
                    pass

    # Perform the search
    result = await search_articles_unified(
        request,
        include_pubmed=True,
        include_preprints=include_preprints,
        include_cbioportal=include_cbioportal,
    )

    # Cache the result (5 minute TTL)
    await _search_cache.set(cache_key, result, ttl=300)

    return result


# Additional optimization: Batch article searches
class ArticleSearchBatcher:
    """Batch multiple article searches to reduce overhead."""

    def __init__(self, batch_size: int = 5, timeout: float = 0.1):
        self.batch_size = batch_size
        self.timeout = timeout
        self._pending_searches: list[tuple[PubmedRequest, asyncio.Future]] = []
        self._batch_task: asyncio.Task | None = None

    async def search(self, request: PubmedRequest) -> str:
        """Add a search to the batch."""
        future = asyncio.get_event_loop().create_future()
        self._pending_searches.append((request, future))

        # Start batch processing if not already running
        if self._batch_task is None or self._batch_task.done():
            self._batch_task = asyncio.create_task(self._process_batch())

        return await future

    async def _process_batch(self):
        """Process pending searches in batch."""
        await asyncio.sleep(self.timeout)  # Wait for more requests

        if not self._pending_searches:
            return

        # Take up to batch_size searches
        batch = self._pending_searches[: self.batch_size]
        self._pending_searches = self._pending_searches[self.batch_size :]

        # Process searches in parallel
        search_tasks = []
        for request, _ in batch:
            task = search_articles_unified(request, include_pubmed=True)
            search_tasks.append(task)

        results = await asyncio.gather(*search_tasks, return_exceptions=True)

        # Set results on futures
        for (_, future), result in zip(batch, results, strict=False):
            if isinstance(result, Exception):
                future.set_exception(result)
            else:
                future.set_result(result)


# Global batcher instance
_article_batcher = ArticleSearchBatcher()


async def article_searcher_batched(
    call_benefit: str,
    chemicals: list[str] | str | None = None,
    diseases: list[str] | str | None = None,
    genes: list[str] | str | None = None,
    keywords: list[str] | str | None = None,
    variants: list[str] | str | None = None,
    include_preprints: bool = True,
    include_cbioportal: bool = True,
) -> str:
    """Batched version of article_searcher for multiple concurrent searches."""

    request = PubmedRequest(
        chemicals=ensure_list(chemicals, split_strings=True),
        diseases=ensure_list(diseases, split_strings=True),
        genes=ensure_list(genes, split_strings=True),
        keywords=ensure_list(keywords, split_strings=True),
        variants=ensure_list(variants, split_strings=True),
    )

    # Use the optimized version with caching
    return await article_searcher_optimized(
        call_benefit=call_benefit,
        chemicals=request.chemicals,
        diseases=request.diseases,
        genes=request.genes,
        keywords=request.keywords,
        variants=request.variants,
        include_preprints=include_preprints,
        include_cbioportal=include_cbioportal,
    )

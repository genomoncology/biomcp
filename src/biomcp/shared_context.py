"""Shared context for search operations to avoid redundant validations.

This module provides a context manager that maintains validated entities
(genes, diseases, chemicals) across multiple search operations to improve
performance by eliminating redundant API calls.

Example:
    ```python
    from biomcp.shared_context import SearchContextManager

    with SearchContextManager() as context:
        # First validation hits the API
        is_valid = await context.validate_gene("BRAF")

        # Subsequent validation uses cache
        is_valid_again = await context.validate_gene("BRAF")
    ```
"""

from typing import Any


class SearchContext:
    """Shared context to avoid redundant operations across searches.

    This class maintains a cache of validated entities to prevent
    redundant API calls during a search session.

    Attributes:
        validated_genes: Cache of gene validation results
        validated_cache: General validation cache for other entities
    """

    def __init__(self):
        self.validated_genes: dict[str, bool] = {}
        self.gene_summaries: dict[str, Any] = {}
        self.cancer_types: dict[str, Any] | None = None
        self._validation_cache: dict[str, Any] = {}

    async def validate_gene(self, gene: str) -> bool:
        """Validate gene symbol with caching."""
        if gene in self.validated_genes:
            return self.validated_genes[gene]

        # Import here to avoid circular imports
        from .utils.gene_validator import is_valid_gene_symbol

        is_valid = is_valid_gene_symbol(gene)
        self.validated_genes[gene] = is_valid
        return is_valid

    def get_gene_summary(self, gene: str) -> Any | None:
        """Get cached gene summary if available."""
        return self.gene_summaries.get(gene)

    def set_gene_summary(self, gene: str, summary: Any):
        """Cache gene summary."""
        self.gene_summaries[gene] = summary

    def cache_validation(self, key: str, value: Any):
        """Cache arbitrary validation results."""
        self._validation_cache[key] = value

    def get_cached_validation(self, key: str) -> Any | None:
        """Get cached validation result."""
        return self._validation_cache.get(key)


# Thread-local context for current search operation
_search_context: SearchContext | None = None


def get_search_context() -> SearchContext | None:
    """Get the current search context."""
    return _search_context


def set_search_context(context: SearchContext | None):
    """Set the current search context."""
    global _search_context
    _search_context = context


class SearchContextManager:
    """Context manager for search operations."""

    _instance = None

    def __init__(self):
        self.context = None
        self.previous_context = None

    def __enter__(self):
        # Use singleton pattern within context
        if SearchContextManager._instance is None:
            SearchContextManager._instance = SearchContext()
        self.context = SearchContextManager._instance
        self.previous_context = get_search_context()
        set_search_context(self.context)
        return self.context

    def __exit__(self, exc_type, exc_val, exc_tb):
        set_search_context(self.previous_context)
        # Clear singleton when last context exits
        if self.previous_context is None:
            SearchContextManager._instance = None
        return False

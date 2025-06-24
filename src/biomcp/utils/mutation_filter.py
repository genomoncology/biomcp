"""Mutation filtering utilities."""

import re
from collections.abc import Sequence
from typing import Protocol


class MutationHitProtocol(Protocol):
    """Protocol for mutation hit objects."""

    protein_change: str


class MutationFilter:
    """Filter mutations based on specific mutation or pattern."""

    def __init__(
        self, specific_mutation: str | None = None, pattern: str | None = None
    ):
        """Initialize the filter.

        Args:
            specific_mutation: Exact mutation to match (e.g., "V600E")
            pattern: Pattern to match (e.g., "V600*" for any V600 mutation)
        """
        self.specific_mutation = specific_mutation
        self.pattern = pattern

    def matches(self, protein_change: str) -> bool:
        """Check if a protein change matches the filter criteria.

        Args:
            protein_change: The protein change to check

        Returns:
            True if matches, False otherwise
        """
        if not protein_change:
            return False

        if self.specific_mutation:
            return protein_change == self.specific_mutation

        if self.pattern:
            return self._matches_pattern(protein_change)

        # No filter specified, match all
        return True

    def _matches_pattern(self, protein_change: str) -> bool:
        """Check if protein change matches pattern.

        Args:
            protein_change: The protein change to check

        Returns:
            True if matches pattern, False otherwise
        """
        if not self.pattern:
            return False

        if self.pattern.endswith("*"):
            # Wildcard pattern (e.g., "V600*" matches "V600E", "V600K", etc.)
            prefix = self.pattern[:-1]
            return protein_change.startswith(prefix)

        # Try regex match
        try:
            # Escape special regex characters except *
            escaped_pattern = re.escape(self.pattern).replace(r"\*", ".*")
            return bool(re.match(f"^{escaped_pattern}$", protein_change))
        except re.error:
            # Fallback to simple prefix match
            return protein_change.startswith(self.pattern)

    def filter_mutations(
        self, mutations: Sequence[MutationHitProtocol]
    ) -> list[MutationHitProtocol]:
        """Filter a list of mutations.

        Args:
            mutations: List of mutation objects with protein_change attribute

        Returns:
            Filtered list of mutations
        """
        if not self.specific_mutation and not self.pattern:
            return list(mutations)

        return [mut for mut in mutations if self.matches(mut.protein_change)]

    def __str__(self) -> str:
        """String representation of the filter."""
        if self.specific_mutation:
            return f"MutationFilter(specific={self.specific_mutation})"
        elif self.pattern:
            return f"MutationFilter(pattern={self.pattern})"
        else:
            return "MutationFilter(no_filter)"

    def __repr__(self) -> str:
        """Detailed representation of the filter."""
        return f"MutationFilter(specific_mutation={self.specific_mutation!r}, pattern={self.pattern!r})"

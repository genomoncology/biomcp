"""Tests for mutation filter utility."""

from biomcp.utils.mutation_filter import MutationFilter


class MockMutation:
    """Mock mutation object for testing."""

    def __init__(self, protein_change: str):
        self.protein_change = protein_change


class TestMutationFilter:
    """Test mutation filtering functionality."""

    def test_specific_mutation_filter(self):
        """Test filtering for specific mutations."""
        mutation_filter = MutationFilter(specific_mutation="V600E")

        assert mutation_filter.matches("V600E")
        assert not mutation_filter.matches("V600K")
        assert not mutation_filter.matches("V600")
        assert not mutation_filter.matches("")

    def test_wildcard_pattern_filter(self):
        """Test filtering with wildcard patterns."""
        mutation_filter = MutationFilter(pattern="V600*")

        assert mutation_filter.matches("V600E")
        assert mutation_filter.matches("V600K")
        assert mutation_filter.matches("V600D")
        assert not mutation_filter.matches("V601E")
        assert not mutation_filter.matches("K600E")

    def test_pattern_without_wildcard(self):
        """Test pattern matching without wildcard."""
        # Pattern does exact match via regex (no prefix matching without *)
        mutation_filter = MutationFilter(pattern="F57")

        # Exact match works
        assert mutation_filter.matches("F57")
        # No prefix matching without wildcard
        assert not mutation_filter.matches("F57Y")
        assert not mutation_filter.matches("F57L")
        assert not mutation_filter.matches("F58Y")

    def test_no_filter(self):
        """Test when no filter is specified."""
        mutation_filter = MutationFilter()

        assert mutation_filter.matches("V600E")
        assert mutation_filter.matches("anything")
        # Empty protein change returns False even with no filter
        assert not mutation_filter.matches("")

    def test_filter_mutations_list(self):
        """Test filtering a list of mutations."""
        mutations = [
            MockMutation("V600E"),
            MockMutation("V600K"),
            MockMutation("V600D"),
            MockMutation("T790M"),
            MockMutation("L858R"),
        ]

        # Test specific mutation
        mutation_filter1 = MutationFilter(specific_mutation="V600E")
        filtered1 = mutation_filter1.filter_mutations(mutations)
        assert len(filtered1) == 1
        assert filtered1[0].protein_change == "V600E"

        # Test pattern
        mutation_filter2 = MutationFilter(pattern="V600*")
        filtered2 = mutation_filter2.filter_mutations(mutations)
        assert len(filtered2) == 3
        assert all(m.protein_change.startswith("V600") for m in filtered2)

        # Test no filter
        mutation_filter3 = MutationFilter()
        filtered3 = mutation_filter3.filter_mutations(mutations)
        assert len(filtered3) == 5

    def test_string_representations(self):
        """Test string representations of filters."""
        mutation_filter1 = MutationFilter(specific_mutation="V600E")
        assert str(mutation_filter1) == "MutationFilter(specific=V600E)"
        assert (
            repr(mutation_filter1)
            == "MutationFilter(specific_mutation='V600E', pattern=None)"
        )

        mutation_filter2 = MutationFilter(pattern="V600*")
        assert str(mutation_filter2) == "MutationFilter(pattern=V600*)"

        mutation_filter3 = MutationFilter()
        assert str(mutation_filter3) == "MutationFilter(no_filter)"

    def test_edge_cases(self):
        """Test edge cases in mutation matching."""
        # Empty protein change
        mutation_filter = MutationFilter(specific_mutation="V600E")
        assert not mutation_filter.matches("")
        assert not mutation_filter.matches(None)

        # Complex patterns
        mutation_filter2 = MutationFilter(pattern="[VL]600*")
        # This will use regex escaping, so won't work as expected
        # But should not crash
        assert not mutation_filter2.matches("V600E")  # Because [ is escaped

    def test_filter_mutations_preserves_type(self):
        """Test that filter preserves the original list type."""
        mutations = [
            MockMutation("V600E"),
            MockMutation("V600K"),
        ]

        mutation_filter = MutationFilter(pattern="V600*")
        result = mutation_filter.filter_mutations(mutations)

        # Result should be a list
        assert isinstance(result, list)
        assert len(result) == 2

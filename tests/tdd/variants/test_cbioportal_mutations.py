"""Tests for cBioPortal mutation-specific search functionality."""

import pytest

from biomcp.utils.mutation_filter import MutationFilter
from biomcp.variants.cbioportal_mutations import (
    CBioPortalMutationClient,
    MutationHit,
    StudyMutationSummary,
    format_mutation_search_result,
)


class TestCBioPortalMutationSearch:
    """Test mutation-specific search functionality."""

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_search_specific_mutation_srsf2_f57y(self):
        """Test searching for SRSF2 F57Y mutation."""
        client = CBioPortalMutationClient()

        result = await client.search_specific_mutation(
            gene="SRSF2", mutation="F57Y", max_studies=10
        )

        assert result is not None
        assert result.gene == "SRSF2"
        assert result.specific_mutation == "F57Y"
        assert result.studies_with_mutation >= 0

        # If mutations found, check structure
        if result.studies_with_mutation > 0:
            assert len(result.top_studies) > 0
            top_study = result.top_studies[0]
            assert isinstance(top_study, StudyMutationSummary)
            assert top_study.mutation_count > 0

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_search_mutation_pattern_srsf2_f57(self):
        """Test searching for SRSF2 F57* mutations."""
        client = CBioPortalMutationClient()

        result = await client.search_specific_mutation(
            gene="SRSF2", pattern="F57*", max_studies=10
        )

        assert result is not None
        assert result.gene == "SRSF2"
        assert result.pattern == "F57*"

        # F57* should match F57Y, F57C, etc.
        if result.total_mutations > 0:
            assert result.mutation_types is not None
            # Check that we found some F57 mutations
            f57_mutations = [
                mut for mut in result.mutation_types if mut.startswith("F57")
            ]
            assert len(f57_mutations) > 0

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_search_braf_v600e(self):
        """Test searching for BRAF V600E - a very common mutation."""
        client = CBioPortalMutationClient()

        result = await client.search_specific_mutation(
            gene="BRAF", mutation="V600E", max_studies=20
        )

        assert result is not None
        assert result.gene == "BRAF"
        assert result.specific_mutation == "V600E"
        # V600E is very common, should have many studies
        assert result.studies_with_mutation > 10
        assert len(result.top_studies) > 0

        # Check melanoma is in top cancer types
        cancer_types = [s.cancer_type for s in result.top_studies]
        # At least some melanoma studies should have V600E
        assert any("melanoma" in ct.lower() for ct in cancer_types)

    def test_filter_mutations_specific(self):
        """Test filtering for specific mutations."""
        mutations = [
            MutationHit(
                study_id="study1",
                molecular_profile_id="study1_mutations",
                protein_change="F57Y",
                mutation_type="Missense",
            ),
            MutationHit(
                study_id="study1",
                molecular_profile_id="study1_mutations",
                protein_change="F57C",
                mutation_type="Missense",
            ),
            MutationHit(
                study_id="study2",
                molecular_profile_id="study2_mutations",
                protein_change="R88Q",
                mutation_type="Missense",
            ),
        ]

        # Filter for F57Y
        mutation_filter = MutationFilter(specific_mutation="F57Y")
        filtered = mutation_filter.filter_mutations(mutations)
        assert len(filtered) == 1
        assert filtered[0].protein_change == "F57Y"

    def test_filter_mutations_pattern(self):
        """Test filtering with wildcard patterns."""
        mutations = [
            MutationHit(
                study_id="study1",
                molecular_profile_id="study1_mutations",
                protein_change="F57Y",
                mutation_type="Missense",
            ),
            MutationHit(
                study_id="study1",
                molecular_profile_id="study1_mutations",
                protein_change="F57C",
                mutation_type="Missense",
            ),
            MutationHit(
                study_id="study2",
                molecular_profile_id="study2_mutations",
                protein_change="R88Q",
                mutation_type="Missense",
            ),
        ]

        # Filter for F57*
        mutation_filter = MutationFilter(pattern="F57*")
        filtered = mutation_filter.filter_mutations(mutations)
        assert len(filtered) == 2
        assert all(m.protein_change.startswith("F57") for m in filtered)

    def test_format_mutation_search_result(self):
        """Test formatting of mutation search results."""
        from biomcp.variants.cbioportal_mutations import MutationSearchResult

        result = MutationSearchResult(
            gene="SRSF2",
            specific_mutation="F57Y",
            total_studies=100,
            studies_with_mutation=3,
            total_mutations=5,
            top_studies=[
                StudyMutationSummary(
                    study_id="msk_ch_2023",
                    study_name="Cancer Therapy and Clonal Hematopoiesis",
                    cancer_type="mixed",
                    mutation_count=5,
                    sample_count=100,
                ),
                StudyMutationSummary(
                    study_id="mds_mskcc_2020",
                    study_name="Myelodysplastic Syndrome Study",
                    cancer_type="mds",
                    mutation_count=2,
                    sample_count=50,
                ),
            ],
            mutation_types={"F57Y": 5},
        )

        formatted = format_mutation_search_result(result)

        assert "SRSF2" in formatted
        assert "F57Y" in formatted
        assert "**Studies with Mutation**: 3" in formatted
        assert "msk_ch_2023" in formatted
        assert "|     5 |" in formatted  # mutation count

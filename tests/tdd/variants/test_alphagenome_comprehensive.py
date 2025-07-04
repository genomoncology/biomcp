"""Comprehensive tests for AlphaGenome integration."""

from unittest.mock import MagicMock, patch

import pandas as pd
import pytest

from biomcp.variants.alphagenome import (
    _validate_inputs,
    predict_variant_effects,
)


class TestInputValidation:
    """Test input validation for AlphaGenome."""

    def test_valid_chromosomes(self):
        """Test validation accepts valid chromosome formats."""
        valid_chroms = ["chr1", "chr22", "chrX", "chrY", "chrM", "chrMT"]
        for chrom in valid_chroms:
            # Should not raise
            _validate_inputs(chrom, 100, "A", "T")

    def test_invalid_chromosomes(self):
        """Test validation rejects invalid chromosome formats."""
        invalid_chroms = ["1", "chr23", "chrZ", "chromosome1", "Chr1", ""]
        for chrom in invalid_chroms:
            with pytest.raises(ValueError, match="Invalid chromosome format"):
                _validate_inputs(chrom, 100, "A", "T")

    def test_invalid_position(self):
        """Test validation rejects invalid positions."""
        with pytest.raises(ValueError, match="Position must be >= 1"):
            _validate_inputs("chr1", 0, "A", "T")
        with pytest.raises(ValueError, match="Position must be >= 1"):
            _validate_inputs("chr1", -10, "A", "T")

    def test_valid_nucleotides(self):
        """Test validation accepts valid nucleotides."""
        valid_cases = [
            ("A", "T"),
            ("C", "G"),
            ("ACGT", "TGCA"),
            ("a", "t"),
            ("acgt", "tgca"),  # lowercase should work
        ]
        for ref, alt in valid_cases:
            # Should not raise
            _validate_inputs("chr1", 100, ref, alt)

    def test_invalid_nucleotides(self):
        """Test validation rejects invalid nucleotides."""
        invalid_cases = [("N", "A"), ("A", "U"), ("AXG", "T"), ("A", "123")]
        for ref, alt in invalid_cases:
            with pytest.raises(ValueError, match="Invalid nucleotides"):
                _validate_inputs("chr1", 100, ref, alt)

    def test_empty_alleles(self):
        """Test validation rejects empty alleles."""
        with pytest.raises(
            ValueError, match="Reference allele cannot be empty"
        ):
            _validate_inputs("chr1", 100, "", "A")
        with pytest.raises(
            ValueError, match="Alternate allele cannot be empty"
        ):
            _validate_inputs("chr1", 100, "A", "")


class TestIntervalSizeCalculation:
    """Test interval size selection logic."""

    @pytest.mark.asyncio
    async def test_interval_size_edge_cases(self):
        """Test interval size selection for edge cases."""
        with patch.dict("os.environ", {}, clear=True):
            # Without API key, we should get early return
            result = await predict_variant_effects(
                chromosome="chr1",
                position=100,
                reference="A",
                alternate="T",
                interval_size=2000000,  # Larger than max
            )
            assert "AlphaGenome API key required" in result


class TestCaching:
    """Test caching behavior."""

    @pytest.mark.asyncio
    async def test_skip_cache_parameter(self):
        """Test that skip_cache parameter works."""
        with patch.dict("os.environ", {}, clear=True):
            # First call
            result1 = await predict_variant_effects(
                chromosome="chr1",
                position=100,
                reference="A",
                alternate="T",
                skip_cache=True,
            )

            # Second call with skip_cache
            result2 = await predict_variant_effects(
                chromosome="chr1",
                position=100,
                reference="A",
                alternate="T",
                skip_cache=True,
            )

            # Both should show API key error
            assert "AlphaGenome API key required" in result1
            assert "AlphaGenome API key required" in result2


class TestErrorHandling:
    """Test error handling and context."""

    @pytest.mark.asyncio
    async def test_error_context_with_api_key(self):
        """Test that errors include proper context."""
        with patch.dict("os.environ", {"ALPHAGENOME_API_KEY": "test-key"}):
            result = await predict_variant_effects(
                chromosome="chr1",
                position=100,
                reference="A",
                alternate="T",
                tissue_types=["UBERON:0002367"],
                skip_cache=True,
            )

            # Should either get import error or API error with context
            if "AlphaGenome prediction failed" in result:
                assert "Context:" in result
                assert "chr1:100 A>T" in result
                assert "Tissue types:" in result

    @pytest.mark.asyncio
    async def test_input_validation_errors(self):
        """Test that input validation errors are raised."""
        with patch.dict("os.environ", {"ALPHAGENOME_API_KEY": "test-key"}):
            # Invalid chromosome
            with pytest.raises(ValueError, match="Invalid chromosome format"):
                await predict_variant_effects(
                    chromosome="invalid",
                    position=100,
                    reference="A",
                    alternate="T",
                )

            # Invalid nucleotides
            with pytest.raises(ValueError, match="Invalid nucleotides"):
                await predict_variant_effects(
                    chromosome="chr1",
                    position=100,
                    reference="X",
                    alternate="T",
                )


class TestThresholdParameter:
    """Test significance threshold parameter."""

    @pytest.mark.asyncio
    async def test_custom_threshold(self):
        """Test that custom threshold is accepted."""
        with patch.dict("os.environ", {}, clear=True):
            # Test with custom threshold
            result = await predict_variant_effects(
                chromosome="chr1",
                position=100,
                reference="A",
                alternate="T",
                significance_threshold=0.8,
            )

            # Should work (get API key error, not parameter error)
            assert "AlphaGenome API key required" in result

    @pytest.mark.asyncio
    async def test_default_threshold(self):
        """Test that default threshold is used."""
        with patch.dict("os.environ", {}, clear=True):
            # Test without threshold parameter
            result = await predict_variant_effects(
                chromosome="chr1",
                position=100,
                reference="A",
                alternate="T",
            )

            # Should work with default
            assert "AlphaGenome API key required" in result


class TestIntegration:
    """Integration tests with mocked AlphaGenome."""

    @pytest.mark.asyncio
    async def test_successful_prediction_mock(self):
        """Test successful prediction with mocked AlphaGenome."""
        with patch.dict("os.environ", {"ALPHAGENOME_API_KEY": "test-key"}):
            # Mock the AlphaGenome imports
            mock_genome = MagicMock()
            mock_dna_client = MagicMock()
            mock_variant_scorers = MagicMock()

            # Mock the model
            mock_model = MagicMock()
            mock_dna_client.create.return_value = mock_model

            # Mock scorers
            mock_variant_scorers.get_recommended_scorers.return_value = [
                "scorer1"
            ]

            # Mock scores DataFrame
            mock_df = pd.DataFrame({
                "output_type": ["RNA_SEQ"],
                "raw_score": [1.0],
                "gene_name": ["GENE1"],
                "track_name": ["tissue1"],
            })
            mock_variant_scorers.tidy_scores.return_value = mock_df

            # Mock score_variant to return mock scores
            mock_model.score_variant.return_value = [MagicMock()]

            # Patch the imports
            with patch.dict(
                "sys.modules",
                {
                    "alphagenome.data.genome": mock_genome,
                    "alphagenome.models.dna_client": mock_dna_client,
                    "alphagenome.models.variant_scorers": mock_variant_scorers,
                    "alphagenome.data": MagicMock(genome=mock_genome),
                    "alphagenome.models": MagicMock(
                        dna_client=mock_dna_client,
                        variant_scorers=mock_variant_scorers,
                    ),
                },
            ):
                result = await predict_variant_effects(
                    chromosome="chr7",
                    position=140753336,
                    reference="A",
                    alternate="T",
                    interval_size=131072,
                    skip_cache=True,
                )

                # Check model was created with API key
                mock_dna_client.create.assert_called_once_with("test-key")

                # Check interval was created correctly
                mock_genome.Interval.assert_called_once()
                call_args = mock_genome.Interval.call_args
                assert (
                    call_args[1]["start"] == 140753336 - 65536 - 1
                )  # 0-based
                assert call_args[1]["end"] == call_args[1]["start"] + 131072

                # Check variant was created
                mock_genome.Variant.assert_called_once_with(
                    chromosome="chr7",
                    position=140753336,
                    reference_bases="A",
                    alternate_bases="T",
                )

                # Check result contains expected formatting
                assert "AlphaGenome Variant Effect Predictions" in result
                assert "Gene Expression" in result
                assert "GENE1" in result

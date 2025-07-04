"""Test AlphaGenome per-request API key functionality."""

import os
from unittest.mock import MagicMock, patch

import pandas as pd
import pytest

from biomcp.variants.alphagenome import predict_variant_effects


@pytest.mark.asyncio
async def test_api_key_parameter_overrides_env_var():
    """Test that api_key parameter takes precedence over environment variable."""
    # Set up environment variable
    with patch.dict("os.environ", {"ALPHAGENOME_API_KEY": "env-key"}):
        # Mock AlphaGenome modules
        mock_genome = MagicMock()
        mock_client = MagicMock()
        mock_scorers = MagicMock()

        # Track which API key was used
        api_keys_used = []

        def track_create(api_key):
            api_keys_used.append(api_key)
            return MagicMock()

        mock_client.create.side_effect = track_create

        # Mock successful prediction
        test_scores_df = pd.DataFrame({
            "output_type": ["RNA_SEQ"],
            "raw_score": [1.5],
            "gene_name": ["BRAF"],
            "track_name": [None],
        })

        mock_scorers.tidy_scores.return_value = test_scores_df
        mock_scorers.get_recommended_scorers.return_value = []

        with patch.dict(
            "sys.modules",
            {
                "alphagenome.data": MagicMock(genome=mock_genome),
                "alphagenome.models": MagicMock(
                    dna_client=mock_client, variant_scorers=mock_scorers
                ),
            },
        ):
            # Test with parameter API key
            result = await predict_variant_effects(
                "chr7", 140753336, "A", "T", api_key="param-key"
            )

            # Verify the parameter key was used, not the env var
            assert len(api_keys_used) == 1
            assert api_keys_used[0] == "param-key"
            assert "BRAF" in result


@pytest.mark.asyncio
async def test_no_api_key_shows_instructions():
    """Test that missing API key shows helpful instructions."""
    # Ensure no environment variable is set
    with patch.dict("os.environ", {}, clear=True):
        # Remove ALPHAGENOME_API_KEY if it exists
        os.environ.pop("ALPHAGENOME_API_KEY", None)

        result = await predict_variant_effects(
            "chr7", 140753336, "A", "T", skip_cache=True
        )

        # Check for instructions
        assert "AlphaGenome API key required" in result
        assert "My AlphaGenome API key is" in result
        assert "ACTION REQUIRED" in result
        assert "https://deepmind.google.com/science/alphagenome" in result


@pytest.mark.asyncio
async def test_env_var_used_when_no_parameter():
    """Test that environment variable is used when no parameter is provided."""
    # Set up environment variable
    with patch.dict("os.environ", {"ALPHAGENOME_API_KEY": "env-key"}):
        # Mock AlphaGenome modules
        mock_genome = MagicMock()
        mock_client = MagicMock()
        mock_scorers = MagicMock()

        # Track which API key was used
        api_keys_used = []

        def track_create(api_key):
            api_keys_used.append(api_key)
            return MagicMock()

        mock_client.create.side_effect = track_create

        # Mock successful prediction
        test_scores_df = pd.DataFrame({
            "output_type": ["RNA_SEQ"],
            "raw_score": [1.5],
            "gene_name": ["BRAF"],
            "track_name": [None],
        })

        mock_scorers.tidy_scores.return_value = test_scores_df
        mock_scorers.get_recommended_scorers.return_value = []

        with patch.dict(
            "sys.modules",
            {
                "alphagenome.data": MagicMock(genome=mock_genome),
                "alphagenome.models": MagicMock(
                    dna_client=mock_client, variant_scorers=mock_scorers
                ),
            },
        ):
            # Test without parameter API key
            result = await predict_variant_effects("chr7", 140753336, "A", "T")

            # Verify the env var key was used
            assert len(api_keys_used) == 1
            assert api_keys_used[0] == "env-key"
            assert "BRAF" in result

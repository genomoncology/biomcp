"""Tests for AlphaGenome integration."""

from unittest.mock import patch

import pytest

from biomcp.variants.alphagenome import predict_variant_effects


@pytest.mark.asyncio
async def test_predict_variant_effects_no_api_key():
    """Test that missing API key returns helpful error message."""
    with patch.dict("os.environ", {}, clear=True):
        result = await predict_variant_effects(
            chromosome="chr7",
            position=140753336,
            reference="A",
            alternate="T",
        )

        assert "AlphaGenome API key required" in result
        assert "https://deepmind.google.com/science/alphagenome" in result
        assert "ALPHAGENOME_API_KEY" in result


@pytest.mark.asyncio
async def test_predict_variant_effects_not_installed():
    """Test that missing AlphaGenome package returns installation instructions or API error."""
    # Since AlphaGenome might be installed in test environments, we need to test both cases
    # We'll set a dummy API key and check what error we get
    import os

    original_key = os.environ.get("ALPHAGENOME_API_KEY")
    try:
        os.environ["ALPHAGENOME_API_KEY"] = "test-key"

        result = await predict_variant_effects(
            chromosome="chr7",
            position=140753336,
            reference="A",
            alternate="T",
            skip_cache=True,  # Skip cache to ensure fresh results
        )

        # The function should either:
        # 1. Handle ImportError if AlphaGenome is not installed
        # 2. Return API error if AlphaGenome is installed but API key is invalid
        # 3. Return a prediction failure for other errors
        assert any([
            "AlphaGenome not installed" in result,
            "AlphaGenome prediction failed" in result,
            "API key not valid"
            in result,  # This can happen with invalid test keys
        ])

        if "AlphaGenome not installed" in result:
            assert "git clone" in result
            assert "pip install" in result
    finally:
        # Restore original key
        if original_key is None:
            os.environ.pop("ALPHAGENOME_API_KEY", None)
        else:
            os.environ["ALPHAGENOME_API_KEY"] = original_key


@pytest.mark.asyncio
async def test_predict_variant_effects_basic_parameters():
    """Test that function accepts the expected parameters."""
    # This tests the function interface without requiring AlphaGenome
    with patch.dict("os.environ", {}, clear=True):
        # Test with all parameters
        result = await predict_variant_effects(
            chromosome="chrX",
            position=12345,
            reference="G",
            alternate="C",
            interval_size=500_000,
            tissue_types=["UBERON:0002367", "UBERON:0001157"],
        )

        # Should get API key error (not import error), proving parameters were accepted
        assert "AlphaGenome API key required" in result

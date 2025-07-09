"""Pytest configuration and fixtures."""

import os
from unittest.mock import AsyncMock, patch

import pytest

# Check if we should skip integration tests
SKIP_INTEGRATION = os.environ.get("SKIP_INTEGRATION_TESTS", "").lower() in (
    "true",
    "1",
    "yes",
)


def pytest_configure(config):
    """Configure pytest with custom markers."""
    config.addinivalue_line(
        "markers",
        "integration: marks tests as integration tests (deselect with '-m \"not integration\"')",
    )


def pytest_collection_modifyitems(config, items):
    """Modify test collection to handle integration tests."""
    if SKIP_INTEGRATION:
        skip_integration = pytest.mark.skip(
            reason="Integration tests disabled via SKIP_INTEGRATION_TESTS env var"
        )
        for item in items:
            if "integration" in item.keywords:
                item.add_marker(skip_integration)


@pytest.fixture
def mock_cbioportal_api():
    """Mock cBioPortal API responses for testing."""
    with patch(
        "biomcp.variants.cbioportal_search.CBioPortalSearchClient.get_gene_search_summary"
    ) as mock:
        # Return a mock summary
        mock.return_value = AsyncMock(
            gene="BRAF",
            total_mutations=1000,
            total_samples_tested=2000,
            mutation_frequency=50.0,
            hotspots=[
                AsyncMock(amino_acid_change="V600E", count=800),
                AsyncMock(amino_acid_change="V600K", count=100),
            ],
            cancer_distribution=["Melanoma", "Colorectal Cancer"],
            study_count=10,
        )
        yield mock

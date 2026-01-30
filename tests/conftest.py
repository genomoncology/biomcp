"""Pytest configuration and fixtures."""

import json
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


def _pubtator3_json(data) -> str:
    return json.dumps(data, separators=(",", ":"), ensure_ascii=False)


_PUBTATOR3_AUTOCOMPLETE_RESPONSES = {
    ("gene", "her2"): [
        {
            "_id": "@GENE_ERBB2",
            "biotype": "gene",
            "name": "ERBB2",
            "match": "Matched on name <m>HER2</m>",
        }
    ],
    ("variant", "braf v600e"): [
        {
            "_id": "@VARIANT_p.V600E_BRAF_human",
            "biotype": "variant",
            "name": "p.V600E",
        }
    ],
    ("disease", "lung adenocarcinoma"): [
        {
            "_id": "@DISEASE_Adenocarcinoma_of_Lung",
            "biotype": "disease",
            "name": "Adenocarcinoma of Lung",
            "match": "Multiple matches",
        }
    ],
    ("chemical", "caffeine"): [
        {
            "_id": "@CHEMICAL_Caffeine",
            "biotype": "chemical",
            "name": "Caffeine",
        }
    ],
}


def _pubtator3_autocomplete_response(params: dict) -> tuple[int, str]:
    query = str(params.get("query", "")).strip().lower()
    concept = str(params.get("concept", "")).strip().lower()

    if query == "iphone":
        return 200, "[]"

    return 200, _pubtator3_json(
        _PUBTATOR3_AUTOCOMPLETE_RESPONSES.get((concept, query), [])
    )


def _pubtator3_search_response(params: dict) -> tuple[int, str]:
    size = int(params.get("size", 10))
    return (
        200,
        _pubtator3_json({
            "results": [
                {
                    "pmid": 21717063,
                    "title": "Stub PubTator3 search result",
                }
            ],
            "page_size": size,
            "current": 1,
            "count": 1,
            "total_pages": 1,
        }),
    )


def _pubtator3_biocjson_response(params: dict) -> tuple[int, str]:
    pmids = {p.strip() for p in str(params.get("pmids", "")).split(",")}

    if "99999999" in pmids:
        return 400, _pubtator3_json({
            "detail": "Could not retrieve publications"
        })

    if "21717063" in pmids:
        return (
            200,
            _pubtator3_json({
                "PubTator3": [
                    {
                        "pmid": 21717063,
                        "passages": [
                            {
                                "infons": {"section_type": "TITLE"},
                                "text": "Stub PubTator3 title",
                            },
                            {
                                "infons": {"section_type": "ABSTRACT"},
                                "text": (
                                    "melanomas presenting with the "
                                    "BRAF(V600E) mutation"
                                ),
                            },
                        ],
                    }
                ]
            }),
        )

    return 200, _pubtator3_json({"PubTator3": []})


def _pubtator3_response(url: str, params: dict) -> tuple[int, str]:
    if url.endswith("/entity/autocomplete/"):
        return _pubtator3_autocomplete_response(params)

    if url.endswith("/search/"):
        return _pubtator3_search_response(params)

    if url.endswith("/publications/export/biocjson"):
        return _pubtator3_biocjson_response(params)

    return 404, _pubtator3_json({"detail": f"Unhandled PubTator3 URL: {url}"})


@pytest.fixture
def mock_pubtator3_http(monkeypatch):
    """Mock PubTator3 API endpoints to keep unit/BDD tests deterministic.

    Use in tests that otherwise make real requests to PubTator3 and can be flaky
    under parallel pytest execution.
    """
    from biomcp import http_client
    from biomcp.constants import PUBTATOR3_BASE_URL

    real_call_http = http_client.call_http

    async def fake_call_http(
        method: str,
        url: str,
        params: dict,
        verify=True,
        retry_config=None,
        headers=None,
    ) -> tuple[int, str]:
        if not url.startswith(PUBTATOR3_BASE_URL):
            return await real_call_http(
                method,
                url,
                params,
                verify=verify,
                retry_config=retry_config,
                headers=headers,
            )

        return _pubtator3_response(url, params)

    monkeypatch.setattr(http_client, "call_http", fake_call_http)
    yield

import json
from unittest.mock import patch

import pytest

from biomcp.articles.search import (
    PubmedRequest,
    ResultItem,
    SearchResponse,
    convert_request,
    search_articles,
)


async def test_convert_search_query(anyio_backend):
    pubmed_request = PubmedRequest(
        chemicals=["Caffeine"],
        diseases=["non-small cell lung cancer"],
        genes=["BRAF"],
        variants=["BRAF V600E"],
        keywords=["therapy"],
    )
    pubtator_request = await convert_request(request=pubmed_request)

    # The API may or may not return prefixed entity IDs, so we check for both possibilities
    query_text = pubtator_request.text

    # Keywords should always be first
    assert query_text.startswith("therapy AND ")

    # Check that all terms are present (with or without prefixes)
    assert "Caffeine" in query_text or "@CHEMICAL_Caffeine" in query_text
    assert (
        "non-small cell lung cancer" in query_text.lower()
        or "carcinoma" in query_text.lower()
        or "@DISEASE_" in query_text
    )
    assert "BRAF" in query_text or "@GENE_BRAF" in query_text
    assert (
        "V600E" in query_text
        or "p.V600E" in query_text
        or "@VARIANT_" in query_text
    )

    # All terms should be joined with AND
    assert (
        query_text.count(" AND ") >= 4
    )  # At least 4 AND operators for 5 terms

    # default page request
    assert pubtator_request.size == 40


async def test_convert_search_query_with_or_logic(anyio_backend):
    """Test that keywords with pipe separators are converted to OR queries."""
    pubmed_request = PubmedRequest(
        genes=["PTEN"],
        keywords=["R173|Arg173|p.R173", "mutation"],
    )
    pubtator_request = await convert_request(request=pubmed_request)

    query_text = pubtator_request.text

    # Check that OR logic is properly formatted
    assert "(R173 OR Arg173 OR p.R173)" in query_text
    assert "mutation" in query_text
    assert "PTEN" in query_text or "@GENE_PTEN" in query_text

    # Check overall structure
    assert (
        query_text.count(" AND ") >= 2
    )  # At least 2 AND operators for 3 terms


async def test_search(anyio_backend):
    """Test search with real API call - may be flaky due to network dependency.

    This test makes real API calls to PubTator3 and can fail due to:
    - Network connectivity issues (Error 599)
    - API rate limiting
    - Changes in search results over time

    Consider using test_search_mocked for more reliable testing.
    """
    query = {
        "genes": ["BRAF"],
        "diseases": ["NSCLC", "Non - Small Cell Lung Cancer"],
        "keywords": ["BRAF mutations NSCLC"],
        "variants": ["mutation", "mutations"],
    }

    query = PubmedRequest(**query)
    output = await search_articles(query, output_json=True)
    data = json.loads(output)
    assert isinstance(data, list)

    # Handle potential errors - if the first item has an 'error' key, it's an error response
    if data and isinstance(data[0], dict) and "error" in data[0]:
        import pytest

        pytest.skip(f"API returned error: {data[0]['error']}")

    assert len(data) == 40
    result = ResultItem.model_validate(data[0])
    # todo: this might be flaky.
    assert (
        result.title
        == "[Expert consensus on the diagnosis and treatment in advanced "
        "non-small cell lung cancer with BRAF mutation in China]."
    )


@pytest.mark.asyncio
async def test_search_mocked(anyio_backend):
    """Test search with mocked API response to avoid network dependency."""
    query = {
        "genes": ["BRAF"],
        "diseases": ["NSCLC", "Non - Small Cell Lung Cancer"],
        "keywords": ["BRAF mutations NSCLC"],
        "variants": ["mutation", "mutations"],
    }

    # Create mock response - don't include abstract here as it will be added by add_abstracts
    mock_response = SearchResponse(
        results=[
            ResultItem(
                pmid=37495419,
                title="[Expert consensus on the diagnosis and treatment in advanced "
                "non-small cell lung cancer with BRAF mutation in China].",
                journal="Zhonghua Zhong Liu Za Zhi",
                authors=["Zhang", "Li", "Wang"],
                date="2023-07-23",
                doi="10.3760/cma.j.cn112152-20230314-00115",
            )
            for _ in range(40)  # Create 40 results
        ],
        page_size=40,
        current=1,
        count=40,
        total_pages=1,
    )

    with patch("biomcp.http_client.request_api") as mock_request:
        mock_request.return_value = (mock_response, None)

        # Mock the autocomplete calls
        with patch("biomcp.articles.search.autocomplete") as mock_autocomplete:
            mock_autocomplete.return_value = (
                None  # Simplified - no entity mapping
            )

            # Mock the call_pubtator_api function
            with patch(
                "biomcp.articles.search.call_pubtator_api"
            ) as mock_pubtator:
                from biomcp.articles.fetch import (
                    Article,
                    FetchArticlesResponse,
                    Passage,
                    PassageInfo,
                )

                # Create a mock response with abstracts
                mock_fetch_response = FetchArticlesResponse(
                    PubTator3=[
                        Article(
                            pmid=37495419,
                            passages=[
                                Passage(
                                    text="This is a test abstract about BRAF mutations in NSCLC.",
                                    infons=PassageInfo(
                                        section_type="ABSTRACT"
                                    ),
                                )
                            ],
                        )
                    ]
                )
                mock_pubtator.return_value = (mock_fetch_response, None)

                query_obj = PubmedRequest(**query)
                output = await search_articles(query_obj, output_json=True)
                data = json.loads(output)

                assert isinstance(data, list)
                assert len(data) == 40
                result = ResultItem.model_validate(data[0])
                assert (
                    result.title
                    == "[Expert consensus on the diagnosis and treatment in advanced "
                    "non-small cell lung cancer with BRAF mutation in China]."
                )
                assert (
                    result.abstract
                    == "This is a test abstract about BRAF mutations in NSCLC."
                )


@pytest.mark.asyncio
async def test_search_network_error(anyio_backend):
    """Test search handles network errors gracefully."""
    query = PubmedRequest(genes=["BRAF"])

    with patch("biomcp.http_client.request_api") as mock_request:
        from biomcp.http_client import RequestError

        mock_request.return_value = (
            None,
            RequestError(code=599, message="Network connectivity error"),
        )

        output = await search_articles(query, output_json=True)
        data = json.loads(output)

        assert isinstance(data, list)
        assert len(data) == 1
        assert "error" in data[0]
        assert "Error 599: Network connectivity error" in data[0]["error"]

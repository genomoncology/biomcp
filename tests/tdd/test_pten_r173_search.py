"""Test case demonstrating PTEN R173 search limitations."""

import asyncio
import json

import pytest

from biomcp.articles.search import PubmedRequest, search_articles


@pytest.mark.asyncio
async def test_pten_r173_search_limitations():
    """Demonstrate that current AND logic is too restrictive for finding PTEN R173 papers."""

    # Test 1: Current approach with multiple keywords
    request_restrictive = PubmedRequest(
        genes=["PTEN"], keywords=["R173", "Arg173"]
    )
    result_restrictive = await search_articles(
        request_restrictive, output_json=True
    )
    data_restrictive = json.loads(result_restrictive)

    # Test 2: Less restrictive approach
    request_less_restrictive = PubmedRequest(genes=["PTEN"], keywords=["R173"])
    result_less_restrictive = await search_articles(
        request_less_restrictive, output_json=True
    )
    data_less_restrictive = json.loads(result_less_restrictive)

    # Test 3: Alternative variant notations
    request_notation = PubmedRequest(genes=["PTEN"], keywords=["p.R173C"])
    result_notation = await search_articles(request_notation, output_json=True)
    data_notation = json.loads(result_notation)

    print("\nPTEN R173 Search Results:")
    print(
        f"1. PTEN + R173 + Arg173 (AND logic): {len(data_restrictive)} articles"
    )
    print(f"2. PTEN + R173 only: {len(data_less_restrictive)} articles")
    print(f"3. PTEN + p.R173C: {len(data_notation)} articles")

    # The restrictive search should find fewer results
    assert len(data_restrictive) <= len(data_less_restrictive)

    # Show some example articles found
    if data_less_restrictive:
        print("\nExample articles found with 'PTEN + R173':")
        for i, article in enumerate(data_less_restrictive[:5]):
            title = article.get("title", "No title")
            pmid = article.get("pmid", "N/A")
            year = article.get("pub_year", article.get("date", "N/A"))
            print(f"{i + 1}. {title[:80]}... (PMID: {pmid}, Year: {year[:4]})")


@pytest.mark.asyncio
async def test_specific_pten_papers_not_found():
    """Test that specific PTEN R173 papers mentioned by user are not found."""

    # Papers mentioned by user that should be found
    expected_papers = [
        "Mester et al 2018 Human Mutation",
        "Mighell et al 2020 AJHG",
        "Smith et al 2016 Proteins",
        "Smith et al 2019 AJHG",
        "Smith et al 2023 JPCB",
    ]

    # Search for Smith IN papers on PTEN
    request = PubmedRequest(keywords=["Smith IN", "PTEN"])
    result = await search_articles(request, output_json=True)
    data = json.loads(result)

    print(f"\nSmith IN + PTEN search found {len(data)} articles")

    # Check if any contain R173 in title/abstract
    r173_papers = []
    for article in data:
        title = article.get("title", "")
        abstract = article.get("abstract", "")
        if (
            "R173" in title
            or "R173" in abstract
            or "Arg173" in title
            or "Arg173" in abstract
        ):
            r173_papers.append(article)

    print(f"Papers mentioning R173/Arg173: {len(r173_papers)}")

    # The issue: R173 might only be in full text, not abstract
    assert len(r173_papers) < len(
        expected_papers
    ), "Not all expected R173 papers are found"


def test_and_logic_explanation():
    """Document why AND logic causes issues for variant searches."""

    explanation = """
    Current search behavior:
    - Query: genes=['PTEN'], keywords=['R173', 'Arg173']
    - Translates to: "@GENE_PTEN AND R173 AND Arg173"
    - This requires ALL terms to be present

    Issues:
    1. Papers may use either "R173" OR "Arg173", not both
    2. Variant notations vary: "R173C", "p.R173C", "c.517C>T", etc.
    3. Specific mutation details may only be in full text, not abstract
    4. AND logic is too restrictive for synonym/variant searches

    Potential solutions:
    1. Implement OR logic within variant/keyword groups
    2. Add variant notation normalization
    3. Support multiple search strategies (AND vs OR)
    4. Consider full-text search capabilities
    """

    print(explanation)
    assert True  # This test is for documentation


if __name__ == "__main__":
    # Run the tests to demonstrate the issue
    asyncio.run(test_pten_r173_search_limitations())
    asyncio.run(test_specific_pten_papers_not_found())
    test_and_logic_explanation()

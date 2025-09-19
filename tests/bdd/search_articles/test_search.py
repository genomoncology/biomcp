"""Test steps for search_pubmed feature."""

from __future__ import annotations

import asyncio
import json

from pytest_bdd import given, parsers, scenarios, then, when

from biomcp.articles.search import (
    PubmedRequest,
    search_articles,
)

scenarios("search.feature")


@given(
    parsers.parse('I build a query for "{gene}" "{disease}" "{variant}"'),
    target_fixture="query",
)
def query(gene, disease, variant) -> PubmedRequest:
    return PubmedRequest(
        genes=[gene],
        diseases=[disease],
        variants=[variant],
    )


@when("I perform a search with that query", target_fixture="result")
def result(query) -> list[dict]:
    text = asyncio.run(search_articles(query, output_json=True))
    return json.loads(text)


@then(parsers.parse('the response should contain the article "{pmid:d}"'))
def step_impl(result: list[dict], pmid: int):
    pm_ids = [article["pmid"] for article in result]
    assert pmid in pm_ids, "pmid not found in {pm_ids}"


@then(
    parsers.parse('the article "{pmid:d}" abstract should contain "{phrase}"'),
)
def step_check_abstract(result: list[dict], pmid: int, phrase: str):
    for r in result:
        if r["pmid"] == pmid and r.get("abstract"):
            assert (
                phrase in r["abstract"]
            ), f"Phrase '{phrase}' not found in article {pmid}'s abstract"
            return
    raise AssertionError(f"Article {pmid} not found or has no abstract")

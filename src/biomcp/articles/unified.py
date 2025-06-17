"""Unified article search combining PubMed and preprint sources."""

import asyncio
import json

from .. import render
from .preprints import search_preprints
from .search import PubmedRequest, search_articles


def _deduplicate_articles(articles: list[dict]) -> list[dict]:
    """Remove duplicate articles based on DOI."""
    seen_dois = set()
    unique_articles = []
    for article in articles:
        doi = article.get("doi")
        if doi and doi in seen_dois:
            continue
        if doi:
            seen_dois.add(doi)
        unique_articles.append(article)
    return unique_articles


def _parse_search_results(results: list) -> list[dict]:
    """Parse search results from JSON strings."""
    all_articles = []
    for result in results:
        if isinstance(result, str):
            try:
                articles = json.loads(result)
                if isinstance(articles, list):
                    all_articles.extend(articles)
            except json.JSONDecodeError:
                continue
    return all_articles


async def search_articles_unified(
    request: PubmedRequest,
    include_pubmed: bool = True,
    include_preprints: bool = False,
    output_json: bool = False,
) -> str:
    """Search for articles across PubMed and preprint sources."""
    tasks = []

    if include_pubmed:
        tasks.append(search_articles(request, output_json=True))

    if include_preprints:
        tasks.append(search_preprints(request, output_json=True))

    if not tasks:
        return json.dumps([]) if output_json else render.to_markdown([])

    # Run searches in parallel
    results = await asyncio.gather(*tasks, return_exceptions=True)

    # Parse and deduplicate results
    all_articles = _parse_search_results(results)
    unique_articles = _deduplicate_articles(all_articles)

    # Sort by publication state (peer-reviewed first) and then by date
    unique_articles.sort(
        key=lambda x: (
            0
            if x.get("publication_state", "peer_reviewed") == "peer_reviewed"
            else 1,
            x.get("date", "0000-00-00"),
        ),
        reverse=True,
    )

    if unique_articles and not output_json:
        return render.to_markdown(unique_articles)
    else:
        return json.dumps(unique_articles, indent=2)

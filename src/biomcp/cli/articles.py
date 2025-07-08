import asyncio
import json
from typing import Annotated

import typer

from ..articles import fetch
from ..articles.search import PubmedRequest, search_articles
from ..articles.unified import search_articles_unified

article_app = typer.Typer(help="Search and retrieve biomedical articles.")


async def get_article_details(
    identifier: str, output_json: bool = False
) -> str:
    """Get article details handling both PMIDs and DOIs with proper output format."""
    # Use the fetch module functions directly to control output format
    if fetch.is_doi(identifier):
        from ..articles.preprints import fetch_europe_pmc_article

        return await fetch_europe_pmc_article(
            identifier, output_json=output_json
        )
    elif fetch.is_pmid(identifier):
        return await fetch.fetch_articles(
            [int(identifier)], full=True, output_json=output_json
        )
    else:
        # Unknown identifier format
        error_data = [
            {
                "error": f"Invalid identifier format: {identifier}. Expected either a PMID (numeric) or DOI (10.xxxx/xxxx format)."
            }
        ]
        if output_json:
            return json.dumps(error_data, indent=2)
        else:
            from .. import render

            return render.to_markdown(error_data)


@article_app.command("search")
def search_article(
    genes: Annotated[
        list[str] | None,
        typer.Option(
            "--gene",
            "-g",
            help="Gene name to search for (can be specified multiple times)",
        ),
    ] = None,
    variants: Annotated[
        list[str] | None,
        typer.Option(
            "--variant",
            "-v",
            help="Genetic variant to search for (can be specified multiple times)",
        ),
    ] = None,
    diseases: Annotated[
        list[str] | None,
        typer.Option(
            "--disease",
            "-d",
            help="Disease name to search for (can be specified multiple times)",
        ),
    ] = None,
    chemicals: Annotated[
        list[str] | None,
        typer.Option(
            "--chemical",
            "-c",
            help="Chemical name to search for (can be specified multiple times)",
        ),
    ] = None,
    keywords: Annotated[
        list[str] | None,
        typer.Option(
            "--keyword",
            "-k",
            help="Keyword to search for (can be specified multiple times)",
        ),
    ] = None,
    page: Annotated[
        int,
        typer.Option(
            "--page",
            "-p",
            help="Page number for pagination (starts at 1)",
        ),
    ] = 1,
    output_json: Annotated[
        bool,
        typer.Option(
            "--json",
            "-j",
            help="Render in JSON format",
            case_sensitive=False,
        ),
    ] = False,
    include_preprints: Annotated[
        bool,
        typer.Option(
            "--include-preprints/--no-preprints",
            help="Include preprint articles from bioRxiv/medRxiv and Europe PMC",
        ),
    ] = True,
):
    """Search biomedical research articles"""
    request = PubmedRequest(
        genes=genes or [],
        variants=variants or [],
        diseases=diseases or [],
        chemicals=chemicals or [],
        keywords=keywords or [],
    )

    if include_preprints:
        result = asyncio.run(
            search_articles_unified(
                request,
                include_pubmed=True,
                include_preprints=True,
                output_json=output_json,
            )
        )
    else:
        result = asyncio.run(search_articles(request, output_json))
    typer.echo(result)


@article_app.command("get")
def get_article(
    identifiers: Annotated[
        list[str],
        typer.Argument(
            help="Article identifiers - PubMed IDs (e.g., 38768446) or DOIs (e.g., 10.1101/2024.01.20.23288905)",
        ),
    ],
    full: Annotated[
        bool,
        typer.Option(
            "--full",
            "-f",
            help="Whether to fetch full article text (PubMed only)",
        ),
    ] = False,
    output_json: Annotated[
        bool,
        typer.Option(
            "--json",
            "-j",
            help="Render in JSON format",
            case_sensitive=False,
        ),
    ] = False,
):
    """
    Retrieve articles by PubMed ID or DOI.

    Supports:
    - PubMed IDs for published articles (e.g., 38768446)
    - DOIs for Europe PMC preprints (e.g., 10.1101/2024.01.20.23288905)

    For multiple articles, results are returned as a list.
    """
    # Handle single identifier
    if len(identifiers) == 1:
        result = asyncio.run(
            get_article_details(identifiers[0], output_json=output_json)
        )
    else:
        # For multiple identifiers, we need to handle them individually
        # since they might be a mix of PMIDs and DOIs
        results = []
        for identifier in identifiers:
            article_result = asyncio.run(
                get_article_details(identifier, output_json=True)
            )
            # Parse the result and add to list
            try:
                article_data = json.loads(article_result)
                if isinstance(article_data, list):
                    results.extend(article_data)
                else:
                    results.append(article_data)
            except json.JSONDecodeError:
                # This shouldn't happen with our new function
                results.append({
                    "error": f"Failed to parse result for {identifier}"
                })

        if output_json:
            result = json.dumps(results, indent=2)
        else:
            from .. import render

            result = render.to_markdown(results)

    typer.echo(result)

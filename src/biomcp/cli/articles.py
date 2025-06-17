import asyncio
from typing import Annotated

import typer

from ..articles import fetch
from ..articles.search import PubmedRequest, search_articles
from ..articles.unified import search_articles_unified

article_app = typer.Typer(help="Search and retrieve biomedical articles.")


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
    pmids: Annotated[
        list[int],
        typer.Argument(
            help="PubMed IDs of articles to retrieve",
        ),
    ],
    full: Annotated[
        bool,
        typer.Option(
            "--full",
            "-f",
            help="Whether to fetch full article text",
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
    Retrieve batch of articles with a list of PubMed IDs.
    """
    result = asyncio.run(fetch.fetch_articles(pmids, full, output_json))
    typer.echo(result)

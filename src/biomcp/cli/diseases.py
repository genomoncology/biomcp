"""CLI commands for disease information and search."""

import asyncio
from typing import Annotated

import typer

from ..diseases import get_disease
from ..diseases.search import format_disease_results, search_diseases
from ..integrations.cts_api import CTSAPIError, get_api_key_instructions

disease_app = typer.Typer(
    no_args_is_help=True,
    help="Search and retrieve disease information",
)


@disease_app.command("get")
def get_disease_cli(
    disease_name: Annotated[
        str,
        typer.Argument(help="Disease name or identifier"),
    ],
) -> None:
    """
    Get disease information from MyDisease.info.

    This returns detailed information including synonyms, definitions,
    and database cross-references.

    Examples:
        biomcp disease get melanoma
        biomcp disease get "lung cancer"
        biomcp disease get GIST
    """
    result = asyncio.run(get_disease(disease_name))
    typer.echo(result)


@disease_app.command("search")
def search_diseases_cli(
    name: Annotated[
        str | None,
        typer.Argument(
            help="Disease name to search for (partial match supported)"
        ),
    ] = None,
    include_synonyms: Annotated[
        bool,
        typer.Option(
            "--synonyms/--no-synonyms",
            help="[Deprecated] This option is ignored - API always searches synonyms",
        ),
    ] = True,
    category: Annotated[
        str | None,
        typer.Option(
            "--category",
            help="Disease category/type filter",
        ),
    ] = None,
    page_size: Annotated[
        int,
        typer.Option(
            "--page-size",
            help="Number of results per page",
            min=1,
            max=100,
        ),
    ] = 20,
    page: Annotated[
        int,
        typer.Option(
            "--page",
            help="Page number",
            min=1,
        ),
    ] = 1,
    api_key: Annotated[
        str | None,
        typer.Option(
            "--api-key",
            help="NCI API key (overrides NCI_API_KEY env var)",
            envvar="NCI_API_KEY",
        ),
    ] = None,
    source: Annotated[
        str,
        typer.Option(
            "--source",
            help="Data source: 'mydisease' (default) or 'nci'",
            show_choices=True,
        ),
    ] = "mydisease",
) -> None:
    """
    Search for diseases in MyDisease.info or NCI CTS database.

    The NCI source provides controlled vocabulary of cancer conditions
    used in clinical trials, with official terms and synonyms.

    Examples:
        # Search MyDisease.info (default)
        biomcp disease search melanoma

        # Search NCI cancer terms
        biomcp disease search melanoma --source nci

        # Search without synonyms
        biomcp disease search "breast cancer" --no-synonyms --source nci

        # Filter by category
        biomcp disease search --category neoplasm --source nci
    """
    if source == "nci":
        # Use NCI CTS API
        try:
            results = asyncio.run(
                search_diseases(
                    name=name,
                    include_synonyms=include_synonyms,
                    category=category,
                    page_size=page_size,
                    page=page,
                    api_key=api_key,
                )
            )

            output = format_disease_results(results)
            typer.echo(output)

        except CTSAPIError as e:
            if "API key required" in str(e):
                typer.echo(get_api_key_instructions())
            else:
                typer.echo(f"Error: {e}", err=True)
            raise typer.Exit(1) from e
        except Exception as e:
            typer.echo(f"Unexpected error: {e}", err=True)
            raise typer.Exit(1) from e
    else:
        # Default to MyDisease.info
        # For now, just search by name
        if name:
            result = asyncio.run(get_disease(name))
            typer.echo(result)
        else:
            typer.echo("Please provide a disease name to search for.")
            raise typer.Exit(1)

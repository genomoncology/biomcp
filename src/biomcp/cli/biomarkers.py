"""CLI commands for biomarker search."""

import asyncio
from typing import Annotated

import typer

from ..biomarkers import search_biomarkers
from ..biomarkers.search import format_biomarker_results
from ..integrations.cts_api import CTSAPIError, get_api_key_instructions

biomarker_app = typer.Typer(
    no_args_is_help=True,
    help="Search biomarkers used in clinical trial eligibility criteria",
)


@biomarker_app.command("search")
def search_biomarkers_cli(
    name: Annotated[
        str | None,
        typer.Argument(
            help="Biomarker name to search for (e.g., 'PD-L1', 'EGFR mutation')"
        ),
    ] = None,
    biomarker_type: Annotated[
        str | None,
        typer.Option(
            "--type",
            help="Type of biomarker ('reference_gene' or 'branch')",
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
) -> None:
    """
    Search for biomarkers used in clinical trial eligibility criteria.

    Note: Biomarker data availability may be limited in CTRP. Results focus on
    biomarkers referenced in trial eligibility criteria. For detailed variant
    annotations, use 'biomcp variant search' with MyVariant.info.

    Examples:
        # Search by biomarker name
        biomcp biomarker search "PD-L1"

        # Search by type
        biomcp biomarker search --type reference_gene

        # Search for specific biomarker
        biomcp biomarker search "EGFR mutation"
    """
    try:
        results = asyncio.run(
            search_biomarkers(
                name=name,
                biomarker_type=biomarker_type,
                page_size=page_size,
                page=page,
                api_key=api_key,
            )
        )

        output = format_biomarker_results(results)
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

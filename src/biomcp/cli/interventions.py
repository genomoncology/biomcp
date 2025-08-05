"""CLI commands for intervention search and lookup."""

import asyncio
from typing import Annotated

import typer

from ..integrations.cts_api import CTSAPIError, get_api_key_instructions
from ..interventions import get_intervention, search_interventions
from ..interventions.getter import format_intervention_details
from ..interventions.search import (
    INTERVENTION_TYPES,
    format_intervention_results,
)

intervention_app = typer.Typer(
    no_args_is_help=True,
    help="Search and retrieve intervention information from NCI CTS API",
)


@intervention_app.command("search")
def search_interventions_cli(
    name: Annotated[
        str | None,
        typer.Argument(
            help="Intervention name to search for (partial match supported)"
        ),
    ] = None,
    intervention_type: Annotated[
        str | None,
        typer.Option(
            "--type",
            help=f"Type of intervention. Options: {', '.join(INTERVENTION_TYPES)}",
            show_choices=True,
        ),
    ] = None,
    synonyms: Annotated[
        bool,
        typer.Option(
            "--synonyms/--no-synonyms",
            help="Include synonym matches in search",
        ),
    ] = True,
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
    Search for interventions (drugs, devices, procedures) in the NCI database.

    Examples:
        # Search by drug name
        biomcp intervention search pembrolizumab

        # Search by type
        biomcp intervention search --type Drug

        # Search for devices
        biomcp intervention search "CAR T" --type Biological

        # Search without synonyms
        biomcp intervention search imatinib --no-synonyms
    """
    try:
        results = asyncio.run(
            search_interventions(
                name=name,
                intervention_type=intervention_type,
                synonyms=synonyms,
                page_size=page_size,
                page=page,
                api_key=api_key,
            )
        )

        output = format_intervention_results(results)
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


@intervention_app.command("get")
def get_intervention_cli(
    intervention_id: Annotated[
        str,
        typer.Argument(help="Intervention ID"),
    ],
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
    Get detailed information about a specific intervention.

    Example:
        biomcp intervention get INT123456
    """
    try:
        intervention_data = asyncio.run(
            get_intervention(
                intervention_id=intervention_id,
                api_key=api_key,
            )
        )

        output = format_intervention_details(intervention_data)
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


@intervention_app.command("types")
def list_intervention_types() -> None:
    """
    List all available intervention types.
    """
    typer.echo("## Available Intervention Types\n")
    for int_type in INTERVENTION_TYPES:
        typer.echo(f"- {int_type}")
    typer.echo("\nUse these values with the --type option when searching.")

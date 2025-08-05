"""CLI commands for organization search and lookup."""

import asyncio
from typing import Annotated

import typer

from ..integrations.cts_api import CTSAPIError, get_api_key_instructions
from ..organizations import get_organization, search_organizations
from ..organizations.getter import format_organization_details
from ..organizations.search import format_organization_results

organization_app = typer.Typer(
    no_args_is_help=True,
    help="Search and retrieve organization information from NCI CTS API",
)


@organization_app.command("search")
def search_organizations_cli(
    name: Annotated[
        str | None,
        typer.Argument(
            help="Organization name to search for (partial match supported)"
        ),
    ] = None,
    org_type: Annotated[
        str | None,
        typer.Option(
            "--type",
            help="Type of organization (e.g., industry, academic)",
        ),
    ] = None,
    city: Annotated[
        str | None,
        typer.Option(
            "--city",
            help="City location",
        ),
    ] = None,
    state: Annotated[
        str | None,
        typer.Option(
            "--state",
            help="State location (2-letter code)",
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
    Search for organizations in the NCI Clinical Trials database.

    Examples:
        # Search by name
        biomcp organization search "MD Anderson"

        # Search by type
        biomcp organization search --type academic

        # Search by location
        biomcp organization search --city Boston --state MA

        # Combine filters
        biomcp organization search Cancer --type industry --state CA
    """
    try:
        results = asyncio.run(
            search_organizations(
                name=name,
                org_type=org_type,
                city=city,
                state=state,
                page_size=page_size,
                page=page,
                api_key=api_key,
            )
        )

        output = format_organization_results(results)
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


@organization_app.command("get")
def get_organization_cli(
    org_id: Annotated[
        str,
        typer.Argument(help="Organization ID"),
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
    Get detailed information about a specific organization.

    Example:
        biomcp organization get ORG123456
    """
    try:
        org_data = asyncio.run(
            get_organization(
                org_id=org_id,
                api_key=api_key,
            )
        )

        output = format_organization_details(org_data)
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

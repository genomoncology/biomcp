"""CLI commands for drug information and search."""

import asyncio
from typing import Annotated

import typer

from ..drugs.getter import get_drug

drug_app = typer.Typer(
    no_args_is_help=True,
    help="Search and retrieve drug information",
)


@drug_app.command("get")
def get_drug_cli(
    drug_id_or_name: Annotated[
        str,
        typer.Argument(
            help="Drug ID, name, or trade name (e.g., imatinib, idecabtagene vicleucel, DB00945)"
        ),
    ],
    output_json: Annotated[
        bool,
        typer.Option(
            "--json",
            "-j",
            help="Render in JSON format",
            case_sensitive=False,
        ),
    ] = False,
) -> None:
    """
    Get drug information from MyChem.info.

    This returns detailed information including drug identifiers,
    chemical properties, trade names, clinical indications, and
    mechanism of action.

    Examples:
        # Search by drug name
        biomcp drug get imatinib
        biomcp drug get aspirin

        # Multi-word drug names
        biomcp drug get "idecabtagene vicleucel"

        # Search by drug ID
        biomcp drug get DB00945
        biomcp drug get CHEMBL25

        # Output as JSON
        biomcp drug get imatinib --json
    """
    result = asyncio.run(get_drug(drug_id_or_name, output_json=output_json))
    typer.echo(result)


@drug_app.command("search")
def search_drugs_cli(
    query: Annotated[
        str,
        typer.Argument(help="Drug search query (name, trade name, or ID)"),
    ],
    page: Annotated[
        int,
        typer.Option(
            "--page",
            "-p",
            help="Page number (starts at 1)",
            min=1,
        ),
    ] = 1,
    page_size: Annotated[
        int,
        typer.Option(
            "--page-size",
            help="Number of results per page",
            min=1,
            max=100,
        ),
    ] = 10,
    output_json: Annotated[
        bool,
        typer.Option(
            "--json",
            "-j",
            help="Render in JSON format",
            case_sensitive=False,
        ),
    ] = False,
) -> None:
    """
    Search for drugs in MyChem.info database.

    This searches across drug names, trade names, and identifiers
    to find matching drugs.

    Examples:
        # Search by drug name
        biomcp drug search imatinib

        # Search by partial name
        biomcp drug search "kinase inhibitor"

        # Search with pagination
        biomcp drug search aspirin --page 2 --page-size 20

        # Output as JSON
        biomcp drug search imatinib --json
    """
    # For now, use get_drug to search by the query
    # A full search implementation would require a separate search function
    result = asyncio.run(get_drug(query, output_json=output_json))
    typer.echo(result)

    # Note about pagination
    if page > 1 or page_size != 10:
        typer.echo(
            "\n---\n"
            "Note: Full search with pagination is currently in development.\n"
            "Currently showing basic drug information for the query.\n",
            err=True,
        )

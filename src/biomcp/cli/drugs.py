"""CLI commands for drug information retrieval."""

import asyncio
from typing import Annotated

import typer

from ..drugs import get_drug

drug_app = typer.Typer(
    no_args_is_help=True,
    help="Search and retrieve drug information from MyChem.info",
)


@drug_app.command("get")
def get_drug_cli(
    drug_id_or_name: Annotated[
        str,
        typer.Argument(
            help="Drug name (e.g., imatinib) or ID (e.g., DB00619, CHEMBL25)"
        ),
    ],
    output_json: Annotated[
        bool,
        typer.Option(
            "--json",
            "-j",
            help="Output in JSON format",
        ),
    ] = False,
) -> None:
    """
    Get drug information from MyChem.info.

    Retrieves comprehensive drug information including:
    - Drug identifiers (DrugBank, ChEMBL, PubChem, etc.)
    - Chemical properties (formula, InChIKey)
    - Trade names and synonyms
    - Clinical indications
    - Mechanism of action
    - Links to external databases

    Examples:
        biomcp drug get imatinib
        biomcp drug get pembrolizumab
        biomcp drug get DB00945
        biomcp drug get imatinib --json
    """
    result = asyncio.run(get_drug(drug_id_or_name, output_json=output_json))
    typer.echo(result)

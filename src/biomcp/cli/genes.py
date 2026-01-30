"""CLI commands for gene information retrieval."""

import asyncio
from typing import Annotated

import typer

from ..genes import get_gene

gene_app = typer.Typer(
    no_args_is_help=True,
    help="Search and retrieve gene information from MyGene.info",
)


@gene_app.command("get")
def get_gene_cli(
    gene_id_or_symbol: Annotated[
        str,
        typer.Argument(
            help="Gene symbol (e.g., TP53, BRAF) or ID (e.g., 7157)"
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
    Get gene information from MyGene.info.

    Retrieves detailed gene annotations including:
    - Official gene name and symbol
    - Gene summary/description
    - Aliases and alternative names
    - Gene type (protein-coding, etc.)
    - Links to external databases

    Examples:
        biomcp gene get TP53
        biomcp gene get BRCA1
        biomcp gene get 7157
        biomcp gene get TP53 --json
    """
    result = asyncio.run(get_gene(gene_id_or_symbol, output_json=output_json))
    typer.echo(result)

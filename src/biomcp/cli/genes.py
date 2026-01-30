"""
CLI commands for gene information retrieval.

Enrichment functionality inspired by gget enrichr (https://github.com/pachterlab/gget).
Citation: Luebbert & Pachter (2023). Bioinformatics, 39(1), btac836.
BioMCP directly integrates with Enrichr API rather than using gget as a dependency.
"""

import asyncio
import sys
from typing import Annotated

import typer

from ..enrichr import ENRICHR_DATABASES
from ..genes import get_gene

gene_app = typer.Typer(
    no_args_is_help=True,
    help="Search and retrieve gene information from MyGene.info",
)


def validate_enrich_type(enrich: str | None) -> str | None:
    """Validate enrichment type and return the database name."""
    if enrich is None:
        return None

    # Check if it's a valid short name
    if enrich.lower() in ENRICHR_DATABASES:
        return ENRICHR_DATABASES[enrich.lower()]

    # Check if it's already a valid database name (contains underscore and year)
    if "_" in enrich and any(
        year in enrich for year in ["2021", "2022", "2023"]
    ):
        return enrich

    # Invalid enrichment type
    raise typer.BadParameter(
        f"Invalid enrichment type: '{enrich}'. "
        f"Available options: {', '.join(ENRICHR_DATABASES.keys())}"
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
    enrich: Annotated[
        str | None,
        typer.Option(
            "--enrich",
            "-e",
            help=f"Add functional enrichment analysis. Options: "
            f"{', '.join(ENRICHR_DATABASES.keys())} or full database name",
        ),
    ] = None,
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
        biomcp gene get TP53 --enrich pathway
        biomcp gene get BRCA1 --enrich ontology --json
    """
    # Validate enrichment type before running async code
    try:
        enrichment_database = validate_enrich_type(enrich)
    except typer.BadParameter:
        typer.echo(f"Invalid enrichment type: '{enrich}'")
        raise typer.Exit(1) from None

    include_enrichment = enrich is not None

    async def run():
        result = await get_gene(
            gene_id_or_symbol=gene_id_or_symbol,
            output_json=output_json,
            include_enrichment=include_enrichment,
            enrichment_database=enrichment_database
            or "GO_Biological_Process_2021",
        )
        typer.echo(result)

    # Add enrichment analysis section if requested
    if include_enrichment:
        typer.echo("\n## Enrichment Analysis\n")
        typer.echo(
            f"Using database: {enrichment_database or enrich} ({enrich})\n"
        )

    try:
        asyncio.run(run())
    except KeyboardInterrupt:
        typer.echo("\nOperation cancelled.", err=True)
        sys.exit(130)
    except Exception as e:
        typer.echo(f"Error: {e}", err=True)
        sys.exit(1)


@gene_app.command("search")
def search_genes_cli(
    query: Annotated[
        str,
        typer.Argument(
            help="Search query (gene name, symbol, or description)"
        ),
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
            help="Output in JSON format",
        ),
    ] = False,
) -> None:
    """
    Search for genes in MyGene.info database.

    This searches across gene names, symbols, and descriptions
    to find matching genes.

    Examples:
        biomcp gene search TP53
        biomcp gene search "tumor protein"
        biomcp gene search kinase --page 2 --page-size 20
        biomcp gene search BRCA --json
    """

    async def run():
        # For now, use get_gene to search by the query
        # A full search implementation would require a separate search function
        result = await get_gene(query, output_json=output_json)
        typer.echo(result)

    try:
        asyncio.run(run())
    except KeyboardInterrupt:
        typer.echo("\nOperation cancelled.", err=True)
        sys.exit(130)
    except Exception as e:
        typer.echo(f"Error: {e}", err=True)
        sys.exit(1)

    # Note about pagination
    if page > 1 or page_size != 10:
        typer.echo(
            "\n---\n"
            "Note: Full search with pagination is currently in development.\n"
            "Currently showing basic gene information for the query.\n",
            err=True,
        )

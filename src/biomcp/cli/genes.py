"""
Gene CLI commands.

Enrichment functionality inspired by gget enrichr (https://github.com/pachterlab/gget).
Citation: Luebbert & Pachter (2023). Bioinformatics, 39(1), btac836.
BioMCP directly integrates with Enrichr API rather than using gget as a dependency.
"""

import asyncio
import sys

import typer

from ..enrichr import ENRICHR_DATABASES
from ..genes.getter import get_gene

gene_app = typer.Typer(
    help="Gene information retrieval and enrichment analysis",
    no_args_is_help=True,
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
def get_gene_command(
    gene_id_or_symbol: str = typer.Argument(
        ...,
        help="Gene symbol (e.g., TP53) or ID (e.g., 7157)",
    ),
    json_output: bool = typer.Option(
        False,
        "--json",
        "-j",
        help="Output as JSON instead of markdown",
    ),
    enrich: str = typer.Option(
        None,
        "--enrich",
        "-e",
        help=f"Add functional enrichment analysis. Options: {', '.join(ENRICHR_DATABASES.keys())} or full database name",
    ),
):
    """
    Get detailed gene information from MyGene.info.

    Examples:
      biomcp gene get TP53
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
            output_json=json_output,
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
def search_genes_command(
    query: str = typer.Argument(
        ...,
        help="Search query (gene name, symbol, or description)",
    ),
    page: int = typer.Option(
        1,
        "--page",
        "-p",
        help="Page number (starts at 1)",
        min=1,
    ),
    page_size: int = typer.Option(
        10,
        "--page-size",
        help="Number of results per page",
        min=1,
        max=100,
    ),
    json_output: bool = typer.Option(
        False,
        "--json",
        "-j",
        help="Output as JSON instead of markdown",
    ),
):
    """
    Search for genes in MyGene.info database.

    This searches across gene names, symbols, and descriptions
    to find matching genes.

    Examples:
        # Search by gene symbol
        biomcp gene search TP53

        # Search by partial name
        biomcp gene search "tumor protein"

        # Search with pagination
        biomcp gene search kinase --page 2 --page-size 20

        # Output as JSON
        biomcp gene search BRCA --json
    """

    async def run():
        # For now, use get_gene to search by the query
        # A full search implementation would require a separate search function
        result = await get_gene(query, output_json=json_output)
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

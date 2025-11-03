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

    async def run():
        include_enrichment = enrich is not None
        enrichment_database = (
            enrich if enrich else "GO_Biological_Process_2021"
        )

        result = await get_gene(
            gene_id_or_symbol=gene_id_or_symbol,
            output_json=json_output,
            include_enrichment=include_enrichment,
            enrichment_database=enrichment_database,
        )
        typer.echo(result)

    try:
        asyncio.run(run())
    except KeyboardInterrupt:
        typer.echo("\nOperation cancelled.", err=True)
        sys.exit(130)
    except Exception as e:
        typer.echo(f"Error: {e}", err=True)
        sys.exit(1)

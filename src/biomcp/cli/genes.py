"""CLI commands for gene information and search."""

import asyncio
from typing import Annotated

import typer

from ..genes import get_gene

gene_app = typer.Typer(
    no_args_is_help=True,
    help="Search and retrieve gene information",
)


@gene_app.command("get")
def get_gene_cli(
    gene_id_or_symbol: Annotated[
        str,
        typer.Argument(help="Gene ID or symbol (e.g., TP53, BRAF, 7157)"),
    ],
    enrich: Annotated[
        str | None,
        typer.Option(
            "--enrich",
            help="Enrichment analysis type: pathway, kegg, reactome, "
            "wikipathways, ontology, go_process, go_molecular, go_cellular, "
            "celltypes, tissues, diseases, gwas, transcription_factors, tf",
        ),
    ] = None,
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
    Get gene information from MyGene.info.

    This returns detailed information including official gene name,
    symbol, aliases, gene type, and links to external databases.

    Optionally perform functional enrichment analysis using Enrichr API.

    Examples:
        # Basic gene information
        biomcp gene get TP53
        biomcp gene get BRCA1
        biomcp gene get 7157

        # Gene information with pathway enrichment
        biomcp gene get TP53 --enrich pathway
        biomcp gene get BRCA1 --enrich ontology
        biomcp gene get EGFR --enrich celltypes

        # Output as JSON
        biomcp gene get TP53 --json
    """
    # Validate enrichment option if provided
    valid_enrichment_types = {
        "pathway",
        "kegg",
        "reactome",
        "wikipathways",
        "ontology",
        "go_process",
        "go_molecular",
        "go_cellular",
        "celltypes",
        "tissues",
        "diseases",
        "gwas",
        "transcription_factors",
        "tf",
    }

    if enrich and enrich not in valid_enrichment_types:
        typer.echo(
            f"Error: Invalid enrichment type '{enrich}'. "
            f"Valid options: {', '.join(sorted(valid_enrichment_types))}",
            err=True,
        )
        raise typer.Exit(1)

    # Get basic gene information
    result = asyncio.run(get_gene(gene_id_or_symbol, output_json=output_json))
    typer.echo(result)

    # Perform enrichment analysis if requested
    if enrich:
        typer.echo("\n---\n")
        typer.echo(
            f"# Enrichment Analysis ({enrich})\n\n"
            "Note: Enrichment functionality is currently in development.\n"
            "This feature will query the Enrichr API to provide functional\n"
            "enrichment analysis including pathways, ontologies, cell types,\n"
            "diseases, and transcription factor targets.\n"
        )


@gene_app.command("search")
def search_genes_cli(
    query: Annotated[
        str,
        typer.Argument(
            help="Gene search query (symbol, name, or description)"
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
            help="Render in JSON format",
            case_sensitive=False,
        ),
    ] = False,
) -> None:
    """
    Search for genes in MyGene.info database.

    This searches across gene symbols, names, and descriptions
    to find matching genes.

    Examples:
        # Search by gene symbol
        biomcp gene search TP53

        # Search by gene name
        biomcp gene search "tumor protein"

        # Search with pagination
        biomcp gene search kinase --page 2 --page-size 20

        # Output as JSON
        biomcp gene search BRCA --json
    """
    # For now, use get_gene to search by the query
    # A full search implementation would require a separate search function
    result = asyncio.run(get_gene(query, output_json=output_json))
    typer.echo(result)

    # Note about pagination
    if page > 1 or page_size != 10:
        typer.echo(
            "\n---\n"
            "Note: Full search with pagination is currently in development.\n"
            "Currently showing basic gene information for the query.\n",
            err=True,
        )

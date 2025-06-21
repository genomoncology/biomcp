"""BioMCP Command Line Interface for genetic variants."""

import asyncio
from typing import Annotated

import typer

from ..constants import SYSTEM_PAGE_SIZE
from ..variants import getter, search

variant_app = typer.Typer(help="Search and get variants from MyVariant.info.")


@variant_app.command("get")
def get_variant(
    variant_id: Annotated[
        str,
        typer.Argument(
            help="rsID (rs456) or MyVariant ID (chr1:g.1234A>G)",
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
    include_external: Annotated[
        bool,
        typer.Option(
            "--include-external/--no-external",
            help="Include annotations from external sources (TCGA, 1000 Genomes, cBioPortal)",
        ),
    ] = True,
):
    """
    Get detailed information about a specific genetic variant.

    Supports HGVS identifiers (e.g., 'chr7:g.140453136A>T') or dbSNP rsIDs.

    Examples:
        Get by HGVS: biomcp variant get "chr7:g.140453136A>T"
        Get by rsID: biomcp variant get rs113488022
        Get as JSON: biomcp variant get rs113488022 --json
        Get without external annotations: biomcp variant get rs113488022 --no-external
    """
    if not variant_id:
        typer.echo("Error: A variant identifier must be provided.", err=True)
        raise typer.Exit(code=1)

    result = asyncio.run(
        getter.get_variant(
            variant_id,
            output_json=output_json,
            include_external=include_external,
        )
    )
    typer.echo(result)


@variant_app.command("search")
def search_variant_cmd(
    gene: Annotated[
        str | None,
        typer.Option(
            "--gene",
            help="Gene symbol (e.g., BRCA1)",
        ),
    ] = None,
    hgvsp: Annotated[
        str | None,
        typer.Option(
            "--hgvsp",
            help="Protein notation (e.g., p.Val600Glu).",
        ),
    ] = None,
    hgvsc: Annotated[
        str | None,
        typer.Option(
            "--hgvsc",
            help="cDNA notation (e.g., c.1799T>A).",
        ),
    ] = None,
    rsid: Annotated[
        str | None,
        typer.Option(
            "--rsid",
            help="dbSNP rsID (e.g., rs113488022)",
        ),
    ] = None,
    region: Annotated[
        str | None,
        typer.Option(
            "--region",
            help="Genomic region (e.g., chr1:69000-70000)",
        ),
    ] = None,
    significance: Annotated[
        search.ClinicalSignificance | None,
        typer.Option(
            "--significance",
            help="Clinical significance (e.g., pathogenic, likely benign)",
            case_sensitive=False,
        ),
    ] = None,
    min_frequency: Annotated[
        float | None,
        typer.Option(
            "--min-frequency",
            help="Minimum gnomAD exome allele frequency (0.0 to 1.0)",
            min=0.0,
            max=1.0,
        ),
    ] = None,
    max_frequency: Annotated[
        float | None,
        typer.Option(
            "--max-frequency",
            help="Maximum gnomAD exome allele frequency (0.0 to 1.0)",
            min=0.0,
            max=1.0,
        ),
    ] = None,
    cadd: Annotated[
        float | None,
        typer.Option(
            "--cadd",
            help="Minimum CADD phred score",
            min=0.0,
        ),
    ] = None,
    polyphen: Annotated[
        search.PolyPhenPrediction | None,
        typer.Option(
            "--polyphen",
            help="PolyPhen-2 prediction: Probably damaging = D,"
            "Possibly damaging = P, Benign = B",
            case_sensitive=False,
        ),
    ] = None,
    sift: Annotated[
        search.SiftPrediction | None,
        typer.Option(
            "--sift",
            help="SIFT prediction: D = Deleterious, T = Tolerated",
            case_sensitive=False,
        ),
    ] = None,
    size: Annotated[
        int,
        typer.Option(
            "--size",
            help="Maximum number of results to return",
            min=1,
            max=100,
        ),
    ] = SYSTEM_PAGE_SIZE,
    sources: Annotated[
        str | None,
        typer.Option(
            "--sources",
            help="Specific sources to include in results (comma-separated)",
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
):
    query = search.VariantQuery(
        gene=gene,
        hgvsp=hgvsp,
        hgvsc=hgvsc,
        rsid=rsid,
        region=region,
        significance=significance,
        min_frequency=min_frequency,
        max_frequency=max_frequency,
        cadd=cadd,
        polyphen=polyphen,
        sift=sift,
        size=size,
        sources=sources.split(",") if sources else [],
    )

    result = asyncio.run(search.search_variants(query, output_json))
    typer.echo(result)

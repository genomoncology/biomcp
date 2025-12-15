"""Consolidated variant tool for accessing genetic variant databases and predictions."""

import logging
from typing import Annotated, Literal

from pydantic import Field

from biomcp.cbioportal_helper import get_variant_cbioportal_summary
from biomcp.core import mcp_app
from biomcp.metrics import track_performance
from biomcp.oncokb_helper import get_oncokb_summary_for_genes
from biomcp.variants.getter import _variant_details
from biomcp.variants.search import _variant_searcher

logger = logging.getLogger(__name__)


@mcp_app.tool()
@track_performance("biomcp.variant")
async def variant(
    action: Annotated[
        Literal["search", "get"],
        Field(
            description="Action to perform: 'search' to find variant database records, 'get' to retrieve specific variant details"
        ),
    ],
    # Search parameters
    gene: Annotated[
        str | None,
        Field(description="Gene symbol for search (e.g., 'BRAF', 'TP53')"),
    ] = None,
    hgvs: Annotated[
        str | None,
        Field(description="HGVS notation for search (genomic, coding, or protein)"),
    ] = None,
    hgvsp: Annotated[
        str | None,
        Field(description="Protein change in HGVS format for search (e.g., 'p.V600E')"),
    ] = None,
    hgvsc: Annotated[
        str | None,
        Field(description="Coding sequence change for search (e.g., 'c.1799T>A')"),
    ] = None,
    rsid: Annotated[
        str | None,
        Field(description="dbSNP rsID for search (e.g., 'rs113488022')"),
    ] = None,
    region: Annotated[
        str | None,
        Field(description="Genomic region for search (e.g., 'chr7:140753336-140753337')"),
    ] = None,
    significance: Annotated[
        Literal[
            "pathogenic",
            "likely_pathogenic",
            "uncertain_significance",
            "likely_benign",
            "benign",
            "conflicting",
        ]
        | None,
        Field(description="Clinical significance filter for search"),
    ] = None,
    frequency_min: Annotated[
        float | None,
        Field(description="Minimum allele frequency for search", ge=0, le=1),
    ] = None,
    frequency_max: Annotated[
        float | None,
        Field(description="Maximum allele frequency for search", ge=0, le=1),
    ] = None,
    consequence: Annotated[
        str | None,
        Field(description="Variant consequence for search (e.g., 'missense_variant')"),
    ] = None,
    cadd_score_min: Annotated[
        float | None,
        Field(description="Minimum CADD score for pathogenicity in search"),
    ] = None,
    sift_prediction: Annotated[
        Literal["deleterious", "tolerated"] | None,
        Field(description="SIFT functional prediction filter for search"),
    ] = None,
    polyphen_prediction: Annotated[
        Literal["probably_damaging", "possibly_damaging", "benign"] | None,
        Field(description="PolyPhen-2 functional prediction filter for search"),
    ] = None,
    include_cbioportal: Annotated[
        bool,
        Field(
            description="Include cBioPortal cancer genomics summary when searching by gene"
        ),
    ] = True,
    include_oncokb: Annotated[
        bool,
        Field(
            description="Include OncoKB precision oncology summary when searching by gene"
        ),
    ] = True,
    page: Annotated[
        int,
        Field(description="Page number (1-based) for search", ge=1),
    ] = 1,
    page_size: Annotated[
        int,
        Field(description="Results per page for search", ge=1, le=100),
    ] = 10,
    # Get parameters
    variant_id: Annotated[
        str | None,
        Field(
            description="Variant ID for 'get' action (HGVS, rsID, or MyVariant ID like 'chr7:g.140753336A>T')"
        ),
    ] = None,
    include_external: Annotated[
        bool,
        Field(
            description="Include external annotations for 'get' action (TCGA, 1000 Genomes, functional predictions)"
        ),
    ] = True,
) -> str:
    """Access genetic variant databases for comprehensive variant annotations.

    ⚠️ PREREQUISITE: Use the 'think' tool FIRST to plan your research strategy!

    This tool provides access to MyVariant.info for variant database queries.

    ## Actions:

    ### search - Find variant database records
    Search MyVariant.info for variant annotations including:
    - Population frequencies (gnomAD, 1000 Genomes)
    - Clinical significance (ClinVar)
    - Functional predictions (SIFT, PolyPhen, CADD)
    - Gene and protein consequences
    - Optional cBioPortal cancer genomics summary
    - Optional OncoKB precision oncology insights

    Important: This searches variant DATABASE RECORDS, NOT articles.
    For articles about variants, use the article tool.

    ### get - Retrieve specific variant details
    Fetch comprehensive annotations for a variant by ID:
    - Gene location and consequences
    - Population frequencies across databases
    - Clinical significance from ClinVar
    - Functional predictions
    - External annotations (TCGA, conservation scores)

    Accepts: HGVS (NM_004333.4:c.1799T>A), rsID (rs113488022),
    or MyVariant ID (chr7:g.140753336A>T)

    ## Examples:

    Search for BRAF V600E variants:
    ```python
    await variant(action="search", gene="BRAF", hgvsp="p.V600E")
    ```

    Search pathogenic variants in TP53:
    ```python
    await variant(action="search", gene="TP53", significance="pathogenic")
    ```

    Get variant details by rsID:
    ```python
    await variant(action="get", variant_id="rs113488022", include_external=True)
    ```

    Search with clinical filters:
    ```python
    await variant(
        action="search",
        gene="EGFR",
        cadd_score_min=20,
        sift_prediction="deleterious",
        frequency_max=0.01
    )
    ```

    Note: For AI-powered variant regulatory effect predictions, use the 'alphagenome' tool.
    """
    logger.info(f"Variant tool called: action={action}")

    if action == "search":
        result = await _variant_searcher(
            call_benefit="Direct variant database search for genetic analysis",
            gene=gene,
            hgvs=hgvs,
            hgvsp=hgvsp,
            hgvsc=hgvsc,
            rsid=rsid,
            region=region,
            significance=significance,
            min_frequency=frequency_min,
            max_frequency=frequency_max,
            cadd=cadd_score_min,
            sift=sift_prediction,
            polyphen=polyphen_prediction,
            size=page_size,
            offset=(page - 1) * page_size if page > 1 else 0,
        )

        # Add cBioPortal summary if searching by gene
        if include_cbioportal and gene:
            cbioportal_summary = await get_variant_cbioportal_summary(gene)
            if cbioportal_summary:
                result = cbioportal_summary + "\n\n" + result

        # Add OncoKB summary if searching by gene
        if include_oncokb and gene:
            oncokb_summary = await get_oncokb_summary_for_genes([gene])
            if oncokb_summary:
                result = oncokb_summary + "\n\n" + result

        return result

    elif action == "get":
        if not variant_id:
            return "Error: 'variant_id' parameter required for 'get' action"
        return await _variant_details(
            call_benefit="Fetch comprehensive variant annotations for interpretation",
            variant_id=variant_id,
            include_external=include_external,
        )

    return f"Error: Invalid action '{action}'"

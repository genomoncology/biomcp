"""Individual MCP tools for specific biomedical search and fetch operations.

This module provides the original 9 individual tools that offer direct access
to specific search and fetch functionality, complementing the unified tools.
"""

import logging
from typing import Annotated, Literal

from pydantic import Field

from biomcp.articles.fetch import _article_details
from biomcp.articles.search import _article_searcher
from biomcp.cbioportal_helper import (
    get_cbioportal_summary_for_genes,
    get_variant_cbioportal_summary,
)
from biomcp.core import ensure_list, mcp_app
from biomcp.metrics import track_performance
from biomcp.trials.getter import (
    _trial_locations,
    _trial_outcomes,
    _trial_protocol,
    _trial_references,
)
from biomcp.trials.search import _trial_searcher
from biomcp.variants.getter import _variant_details
from biomcp.variants.search import _variant_searcher

logger = logging.getLogger(__name__)


# Article Tools
@mcp_app.tool()
@track_performance("biomcp.article_searcher")
async def article_searcher(
    chemicals: Annotated[
        list[str] | str | None,
        Field(description="Chemical/drug names to search for"),
    ] = None,
    diseases: Annotated[
        list[str] | str | None,
        Field(description="Disease names to search for"),
    ] = None,
    genes: Annotated[
        list[str] | str | None,
        Field(description="Gene symbols to search for"),
    ] = None,
    keywords: Annotated[
        list[str] | str | None,
        Field(description="Free-text keywords to search for"),
    ] = None,
    variants: Annotated[
        list[str] | str | None,
        Field(
            description="Variant strings to search for (e.g., 'V600E', 'p.D277Y')"
        ),
    ] = None,
    include_preprints: Annotated[
        bool,
        Field(description="Include preprints from bioRxiv/medRxiv"),
    ] = True,
    include_cbioportal: Annotated[
        bool,
        Field(
            description="Include cBioPortal cancer genomics summary when searching by gene"
        ),
    ] = True,
    page: Annotated[
        int,
        Field(description="Page number (1-based)", ge=1),
    ] = 1,
    page_size: Annotated[
        int,
        Field(description="Results per page", ge=1, le=100),
    ] = 10,
) -> str:
    """Search PubMed/PubTator3 for research articles and preprints.

    ⚠️ PREREQUISITE: Use the 'think' tool FIRST to plan your research strategy!

    Use this tool to find scientific literature ABOUT genes, variants, diseases, or chemicals.
    Results include articles from PubMed and optionally preprints from bioRxiv/medRxiv.

    Important: This searches for ARTICLES ABOUT these topics, not database records.
    For genetic variant database records, use variant_searcher instead.

    Example usage:
    - Find articles about BRAF mutations in melanoma
    - Search for papers on a specific drug's effects
    - Locate research on gene-disease associations
    """
    # Convert single values to lists
    chemicals = ensure_list(chemicals) if chemicals else None
    diseases = ensure_list(diseases) if diseases else None
    genes = ensure_list(genes) if genes else None
    keywords = ensure_list(keywords) if keywords else None
    variants = ensure_list(variants) if variants else None

    result = await _article_searcher(
        call_benefit="Direct article search for specific biomedical topics",
        chemicals=chemicals,
        diseases=diseases,
        genes=genes,
        keywords=keywords,
        variants=variants,
        include_preprints=include_preprints,
        include_cbioportal=include_cbioportal,
    )

    # Add cBioPortal summary if searching by gene
    if include_cbioportal and genes:
        request_params = {
            "keywords": keywords,
            "diseases": diseases,
            "chemicals": chemicals,
            "variants": variants,
        }
        cbioportal_summary = await get_cbioportal_summary_for_genes(
            genes, request_params
        )
        if cbioportal_summary:
            result = cbioportal_summary + "\n\n---\n\n" + result

    return result


@mcp_app.tool()
@track_performance("biomcp.article_getter")
async def article_getter(
    pmid: Annotated[
        str,
        Field(
            description="Article identifier - either a PubMed ID (e.g., '38768446' or 'PMC11193658') or DOI (e.g., '10.1101/2024.01.20.23288905')"
        ),
    ],
) -> str:
    """Fetch detailed information for a specific article.

    Retrieves the full abstract and available text for an article by its identifier.
    Supports:
    - PubMed IDs (PMID) for published articles
    - PMC IDs for articles in PubMed Central
    - DOIs for preprints from Europe PMC

    Returns formatted text including:
    - Title
    - Abstract
    - Full text (when available from PMC for published articles)
    - Source information (PubMed or Europe PMC)
    """
    return await _article_details(
        call_benefit="Fetch detailed article information for analysis",
        pmid=pmid,
    )


# Trial Tools
@mcp_app.tool()
@track_performance("biomcp.trial_searcher")
async def trial_searcher(
    conditions: Annotated[
        list[str] | str | None,
        Field(description="Medical conditions to search for"),
    ] = None,
    interventions: Annotated[
        list[str] | str | None,
        Field(description="Treatment interventions to search for"),
    ] = None,
    other_terms: Annotated[
        list[str] | str | None,
        Field(description="Additional search terms"),
    ] = None,
    recruiting_status: Annotated[
        Literal["OPEN", "CLOSED", "ANY"] | None,
        Field(description="Filter by recruiting status"),
    ] = None,
    phase: Annotated[
        Literal[
            "EARLY_PHASE1",
            "PHASE1",
            "PHASE2",
            "PHASE3",
            "PHASE4",
            "NOT_APPLICABLE",
        ]
        | None,
        Field(description="Filter by clinical trial phase"),
    ] = None,
    location: Annotated[
        str | None,
        Field(description="Location term for geographic filtering"),
    ] = None,
    lat: Annotated[
        float | None,
        Field(
            description="Latitude for location-based search. AI agents should geocode city names before using.",
            ge=-90,
            le=90,
        ),
    ] = None,
    long: Annotated[
        float | None,
        Field(
            description="Longitude for location-based search. AI agents should geocode city names before using.",
            ge=-180,
            le=180,
        ),
    ] = None,
    distance: Annotated[
        int | None,
        Field(
            description="Distance in miles from lat/long coordinates",
            ge=1,
        ),
    ] = None,
    age_group: Annotated[
        Literal["CHILD", "ADULT", "OLDER_ADULT"] | None,
        Field(description="Filter by age group"),
    ] = None,
    sex: Annotated[
        Literal["FEMALE", "MALE", "ALL"] | None,
        Field(description="Filter by biological sex"),
    ] = None,
    healthy_volunteers: Annotated[
        Literal["YES", "NO"] | None,
        Field(description="Filter by healthy volunteer eligibility"),
    ] = None,
    study_type: Annotated[
        Literal["INTERVENTIONAL", "OBSERVATIONAL", "EXPANDED_ACCESS"] | None,
        Field(description="Filter by study type"),
    ] = None,
    funder_type: Annotated[
        Literal["NIH", "OTHER_GOV", "INDUSTRY", "OTHER"] | None,
        Field(description="Filter by funding source"),
    ] = None,
    page: Annotated[
        int,
        Field(description="Page number (1-based)", ge=1),
    ] = 1,
    page_size: Annotated[
        int,
        Field(description="Results per page", ge=1, le=100),
    ] = 10,
) -> str:
    """Search ClinicalTrials.gov for clinical studies.

    ⚠️ PREREQUISITE: Use the 'think' tool FIRST to plan your research strategy!

    Comprehensive search tool for finding clinical trials based on multiple criteria.
    Supports filtering by conditions, interventions, location, phase, and eligibility.

    Location search notes:
    - Use either location term OR lat/long coordinates, not both
    - For city-based searches, AI agents should geocode to lat/long first
    - Distance parameter only works with lat/long coordinates

    Returns a formatted list of matching trials with key details.
    """
    # Validate location parameters
    if location and (lat is not None or long is not None):
        raise ValueError(
            "Use either location term OR lat/long coordinates, not both"
        )

    if (lat is not None and long is None) or (
        lat is None and long is not None
    ):
        raise ValueError(
            "Both latitude and longitude must be provided together"
        )

    if distance is not None and (lat is None or long is None):
        raise ValueError(
            "Distance parameter requires both latitude and longitude"
        )

    # Convert single values to lists
    conditions = ensure_list(conditions) if conditions else None
    interventions = ensure_list(interventions) if interventions else None
    other_terms = ensure_list(other_terms) if other_terms else None

    return await _trial_searcher(
        call_benefit="Direct clinical trial search for specific criteria",
        conditions=conditions,
        interventions=interventions,
        terms=other_terms,
        recruiting_status=recruiting_status,
        phase=phase,
        lat=lat,
        long=long,
        distance=distance,
        age_group=age_group,
        study_type=study_type,
        page_size=page_size,
    )


@mcp_app.tool()
@track_performance("biomcp.trial_getter")
async def trial_getter(
    nct_id: Annotated[
        str,
        Field(description="NCT ID (e.g., 'NCT06524388')"),
    ],
) -> str:
    """Fetch comprehensive details for a specific clinical trial.

    Retrieves all available information for a clinical trial by its NCT ID.
    This includes protocol details, locations, outcomes, and references.

    For specific sections only, use the specialized getter tools:
    - trial_protocol_getter: Core protocol information
    - trial_locations_getter: Site locations and contacts
    - trial_outcomes_getter: Primary/secondary outcomes and results
    - trial_references_getter: Publications and references
    """
    results = []

    # Get all sections
    protocol = await _trial_protocol(
        call_benefit="Fetch comprehensive trial details for analysis",
        nct_id=nct_id,
    )
    if protocol:
        results.append(protocol)

    locations = await _trial_locations(
        call_benefit="Fetch comprehensive trial details for analysis",
        nct_id=nct_id,
    )
    if locations:
        results.append(locations)

    outcomes = await _trial_outcomes(
        call_benefit="Fetch comprehensive trial details for analysis",
        nct_id=nct_id,
    )
    if outcomes:
        results.append(outcomes)

    references = await _trial_references(
        call_benefit="Fetch comprehensive trial details for analysis",
        nct_id=nct_id,
    )
    if references:
        results.append(references)

    return (
        "\n\n".join(results)
        if results
        else f"No data found for trial {nct_id}"
    )


@mcp_app.tool()
@track_performance("biomcp.trial_protocol_getter")
async def trial_protocol_getter(
    nct_id: Annotated[
        str,
        Field(description="NCT ID (e.g., 'NCT06524388')"),
    ],
) -> str:
    """Fetch core protocol information for a clinical trial.

    Retrieves essential protocol details including:
    - Official title and brief summary
    - Study status and sponsor information
    - Study design (type, phase, allocation, masking)
    - Eligibility criteria
    - Primary completion date
    """
    return await _trial_protocol(
        call_benefit="Fetch trial protocol information for eligibility assessment",
        nct_id=nct_id,
    )


@mcp_app.tool()
@track_performance("biomcp.trial_references_getter")
async def trial_references_getter(
    nct_id: Annotated[
        str,
        Field(description="NCT ID (e.g., 'NCT06524388')"),
    ],
) -> str:
    """Fetch publications and references for a clinical trial.

    Retrieves all linked publications including:
    - Published results papers
    - Background literature
    - Protocol publications
    - Related analyses

    Includes PubMed IDs when available for easy cross-referencing.
    """
    return await _trial_references(
        call_benefit="Fetch trial publications and references for evidence review",
        nct_id=nct_id,
    )


@mcp_app.tool()
@track_performance("biomcp.trial_outcomes_getter")
async def trial_outcomes_getter(
    nct_id: Annotated[
        str,
        Field(description="NCT ID (e.g., 'NCT06524388')"),
    ],
) -> str:
    """Fetch outcome measures and results for a clinical trial.

    Retrieves detailed outcome information including:
    - Primary outcome measures
    - Secondary outcome measures
    - Results data (if available)
    - Adverse events (if reported)

    Note: Results are only available for completed trials that have posted data.
    """
    return await _trial_outcomes(
        call_benefit="Fetch trial outcome measures and results for efficacy assessment",
        nct_id=nct_id,
    )


@mcp_app.tool()
@track_performance("biomcp.trial_locations_getter")
async def trial_locations_getter(
    nct_id: Annotated[
        str,
        Field(description="NCT ID (e.g., 'NCT06524388')"),
    ],
) -> str:
    """Fetch contact and location details for a clinical trial.

    Retrieves all study locations including:
    - Facility names and addresses
    - Principal investigator information
    - Contact details (when recruiting)
    - Recruitment status by site

    Useful for finding trials near specific locations or contacting study teams.
    """
    return await _trial_locations(
        call_benefit="Fetch trial locations and contacts for enrollment information",
        nct_id=nct_id,
    )


# Variant Tools
@mcp_app.tool()
@track_performance("biomcp.variant_searcher")
async def variant_searcher(
    gene: Annotated[
        str | None,
        Field(description="Gene symbol (e.g., 'BRAF', 'TP53')"),
    ] = None,
    hgvs: Annotated[
        str | None,
        Field(description="HGVS notation (genomic, coding, or protein)"),
    ] = None,
    hgvsp: Annotated[
        str | None,
        Field(description="Protein change in HGVS format (e.g., 'p.V600E')"),
    ] = None,
    hgvsc: Annotated[
        str | None,
        Field(description="Coding sequence change (e.g., 'c.1799T>A')"),
    ] = None,
    rsid: Annotated[
        str | None,
        Field(description="dbSNP rsID (e.g., 'rs113488022')"),
    ] = None,
    region: Annotated[
        str | None,
        Field(description="Genomic region (e.g., 'chr7:140753336-140753337')"),
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
        Field(description="Clinical significance filter"),
    ] = None,
    frequency_min: Annotated[
        float | None,
        Field(description="Minimum allele frequency", ge=0, le=1),
    ] = None,
    frequency_max: Annotated[
        float | None,
        Field(description="Maximum allele frequency", ge=0, le=1),
    ] = None,
    consequence: Annotated[
        str | None,
        Field(description="Variant consequence (e.g., 'missense_variant')"),
    ] = None,
    cadd_score_min: Annotated[
        float | None,
        Field(description="Minimum CADD score for pathogenicity"),
    ] = None,
    sift_prediction: Annotated[
        Literal["deleterious", "tolerated"] | None,
        Field(description="SIFT functional prediction"),
    ] = None,
    polyphen_prediction: Annotated[
        Literal["probably_damaging", "possibly_damaging", "benign"] | None,
        Field(description="PolyPhen-2 functional prediction"),
    ] = None,
    include_cbioportal: Annotated[
        bool,
        Field(
            description="Include cBioPortal cancer genomics summary when searching by gene"
        ),
    ] = True,
    page: Annotated[
        int,
        Field(description="Page number (1-based)", ge=1),
    ] = 1,
    page_size: Annotated[
        int,
        Field(description="Results per page", ge=1, le=100),
    ] = 10,
) -> str:
    """Search MyVariant.info for genetic variant DATABASE RECORDS.

    ⚠️ PREREQUISITE: Use the 'think' tool FIRST to plan your research strategy!

    Important: This searches for variant DATABASE RECORDS (frequency, significance, etc.),
    NOT articles about variants. For articles about variants, use article_searcher.

    Searches the comprehensive variant database including:
    - Population frequencies (gnomAD, 1000 Genomes, etc.)
    - Clinical significance (ClinVar)
    - Functional predictions (SIFT, PolyPhen, CADD)
    - Gene and protein consequences

    Search by various identifiers or filter by clinical/functional criteria.
    """
    result = await _variant_searcher(
        call_benefit="Direct variant database search for genetic analysis",
        gene=gene,
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

    return result


@mcp_app.tool()
@track_performance("biomcp.variant_getter")
async def variant_getter(
    variant_id: Annotated[
        str,
        Field(
            description="Variant ID (HGVS, rsID, or MyVariant ID like 'chr7:g.140753336A>T')"
        ),
    ],
    include_external: Annotated[
        bool,
        Field(
            description="Include external annotations (TCGA, 1000 Genomes, functional predictions)"
        ),
    ] = True,
) -> str:
    """Fetch comprehensive details for a specific genetic variant.

    Retrieves all available information for a variant including:
    - Gene location and consequences
    - Population frequencies across databases
    - Clinical significance from ClinVar
    - Functional predictions
    - External annotations (TCGA cancer data, conservation scores)

    Accepts various ID formats:
    - HGVS: NM_004333.4:c.1799T>A
    - rsID: rs113488022
    - MyVariant ID: chr7:g.140753336A>T
    """
    return await _variant_details(
        call_benefit="Fetch comprehensive variant annotations for interpretation",
        variant_id=variant_id,
        include_external=include_external,
    )


@mcp_app.tool()
@track_performance("biomcp.alphagenome_predictor")
async def alphagenome_predictor(
    chromosome: Annotated[
        str,
        Field(description="Chromosome (e.g., 'chr7', 'chrX')"),
    ],
    position: Annotated[
        int,
        Field(description="1-based genomic position of the variant"),
    ],
    reference: Annotated[
        str,
        Field(description="Reference allele(s) (e.g., 'A', 'ATG')"),
    ],
    alternate: Annotated[
        str,
        Field(description="Alternate allele(s) (e.g., 'T', 'A')"),
    ],
    interval_size: Annotated[
        int,
        Field(
            description="Size of genomic interval to analyze in bp (max 1,000,000)",
            ge=2000,
            le=1000000,
        ),
    ] = 131072,
    tissue_types: Annotated[
        list[str] | str | None,
        Field(
            description="UBERON ontology terms for tissue-specific predictions (e.g., 'UBERON:0002367' for external ear)"
        ),
    ] = None,
    significance_threshold: Annotated[
        float,
        Field(
            description="Threshold for significant log2 fold changes (default: 0.5)",
            ge=0.0,
            le=5.0,
        ),
    ] = 0.5,
    api_key: Annotated[
        str | None,
        Field(
            description="AlphaGenome API key. Check if user mentioned 'my AlphaGenome API key is...' in their message. If not provided here and no env var is set, user will be prompted to provide one."
        ),
    ] = None,
) -> str:
    """Predict variant effects on gene regulation using Google DeepMind's AlphaGenome.

    ⚠️ PREREQUISITE: Use the 'think' tool FIRST to plan your analysis strategy!

    AlphaGenome provides state-of-the-art predictions for how genetic variants
    affect gene regulation, including:
    - Gene expression changes (RNA-seq)
    - Chromatin accessibility impacts (ATAC-seq, DNase-seq)
    - Splicing alterations
    - Promoter activity changes (CAGE)

    This tool requires:
    1. AlphaGenome to be installed (see error message for instructions)
    2. An API key from https://deepmind.google.com/science/alphagenome

    API Key Options:
    - Provide directly via the api_key parameter
    - Or set ALPHAGENOME_API_KEY environment variable

    Example usage:
    - Predict regulatory effects of BRAF V600E mutation: chr7:140753336 A>T
    - Assess non-coding variant impact on gene expression
    - Evaluate promoter variants in specific tissues

    Note: This is an optional tool that enhances variant interpretation
    with AI predictions. Standard annotations remain available via variant_getter.
    """
    from biomcp.variants.alphagenome import predict_variant_effects

    # Convert tissue_types to list if needed
    tissue_types_list = ensure_list(tissue_types) if tissue_types else None

    # Call the prediction function
    return await predict_variant_effects(
        chromosome=chromosome,
        position=position,
        reference=reference,
        alternate=alternate,
        interval_size=interval_size,
        tissue_types=tissue_types_list,
        significance_threshold=significance_threshold,
        api_key=api_key,
    )

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
from biomcp.diseases.getter import _disease_details
from biomcp.drugs.getter import _drug_details
from biomcp.genes.getter import _gene_details
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


# Gene Tools
@mcp_app.tool()
@track_performance("biomcp.gene_getter")
async def gene_getter(
    gene_id_or_symbol: Annotated[
        str,
        Field(
            description="Gene symbol (e.g., 'TP53', 'BRAF') or Entrez ID (e.g., '7157')"
        ),
    ],
) -> str:
    """Get detailed gene information from MyGene.info.

    ⚠️ PREREQUISITE: Use the 'think' tool FIRST to understand your research goal!

    Provides real-time gene annotations including:
    - Official gene name and symbol
    - Gene summary/description
    - Aliases and alternative names
    - Gene type (protein-coding, etc.)
    - Links to external databases

    This tool fetches CURRENT gene information from MyGene.info, ensuring
    you always have the latest annotations and nomenclature.

    Example usage:
    - Get information about TP53 tumor suppressor
    - Look up BRAF kinase gene details
    - Find the official name for a gene by its alias

    Note: For genetic variants, use variant_searcher. For articles about genes, use article_searcher.
    """
    return await _gene_details(
        call_benefit="Get up-to-date gene annotations and information",
        gene_id_or_symbol=gene_id_or_symbol,
    )


# Disease Tools
@mcp_app.tool()
@track_performance("biomcp.disease_getter")
async def disease_getter(
    disease_id_or_name: Annotated[
        str,
        Field(
            description="Disease name (e.g., 'melanoma', 'lung cancer') or ontology ID (e.g., 'MONDO:0016575', 'DOID:1909')"
        ),
    ],
) -> str:
    """Get detailed disease information from MyDisease.info.

    ⚠️ PREREQUISITE: Use the 'think' tool FIRST to understand your research goal!

    Provides real-time disease annotations including:
    - Official disease name and definition
    - Disease synonyms and alternative names
    - Ontology mappings (MONDO, DOID, OMIM, etc.)
    - Associated phenotypes
    - Links to disease databases

    This tool fetches CURRENT disease information from MyDisease.info, ensuring
    you always have the latest ontology mappings and definitions.

    Example usage:
    - Get the definition of GIST (Gastrointestinal Stromal Tumor)
    - Look up synonyms for melanoma
    - Find the MONDO ID for a disease by name

    Note: For clinical trials about diseases, use trial_searcher. For articles about diseases, use article_searcher.
    """
    return await _disease_details(
        call_benefit="Get up-to-date disease definitions and ontology information",
        disease_id_or_name=disease_id_or_name,
    )


@mcp_app.tool()
@track_performance("biomcp.drug_getter")
async def drug_getter(
    drug_id_or_name: Annotated[
        str,
        Field(
            description="Drug name (e.g., 'aspirin', 'imatinib') or ID (e.g., 'DB00945', 'CHEMBL941')"
        ),
    ],
) -> str:
    """Get detailed drug/chemical information from MyChem.info.

    ⚠️ PREREQUISITE: Use the 'think' tool FIRST to understand your research goal!

    This tool provides comprehensive drug information including:
    - Chemical properties (formula, InChIKey)
    - Drug identifiers (DrugBank, ChEMBL, PubChem)
    - Trade names and brand names
    - Clinical indications
    - Mechanism of action
    - Pharmacology details
    - Links to drug databases

    This tool fetches CURRENT drug information from MyChem.info, part of the
    BioThings suite, ensuring you always have the latest drug data.

    Example usage:
    - Get information about imatinib (Gleevec)
    - Look up details for DrugBank ID DB00619
    - Find the mechanism of action for pembrolizumab

    Note: For clinical trials about drugs, use trial_searcher. For articles about drugs, use article_searcher.
    """
    return await _drug_details(drug_id_or_name)


# NCI-Specific Tools
@mcp_app.tool()
@track_performance("biomcp.nci_organization_searcher")
async def nci_organization_searcher(
    name: Annotated[
        str | None,
        Field(
            description="Organization name to search for (partial match supported)"
        ),
    ] = None,
    organization_type: Annotated[
        str | None,
        Field(
            description="Type of organization (e.g., 'Academic', 'Industry', 'Government')"
        ),
    ] = None,
    city: Annotated[
        str | None,
        Field(
            description="City where organization is located. IMPORTANT: Always use with state to avoid API errors"
        ),
    ] = None,
    state: Annotated[
        str | None,
        Field(
            description="State/province code (e.g., 'CA', 'NY'). IMPORTANT: Always use with city to avoid API errors"
        ),
    ] = None,
    api_key: Annotated[
        str | None,
        Field(
            description="NCI API key. Check if user mentioned 'my NCI API key is...' in their message. If not provided here and no env var is set, user will be prompted to provide one."
        ),
    ] = None,
    page: Annotated[
        int,
        Field(description="Page number (1-based)", ge=1),
    ] = 1,
    page_size: Annotated[
        int,
        Field(description="Results per page", ge=1, le=100),
    ] = 20,
) -> str:
    """Search for organizations in the NCI Clinical Trials database.

    Searches the National Cancer Institute's curated database of organizations
    involved in cancer clinical trials. This includes:
    - Academic medical centers
    - Community hospitals
    - Industry sponsors
    - Government facilities
    - Research networks

    Requires NCI API key from: https://clinicaltrialsapi.cancer.gov/

    IMPORTANT: To avoid API errors, always use city AND state together when searching by location.
    The NCI API has limitations on broad searches.

    Example usage:
    - Find cancer centers in Boston, MA (city AND state)
    - Search for "MD Anderson" in Houston, TX
    - List academic organizations in Cleveland, OH
    - Search by organization name alone (without location)
    """
    from biomcp.integrations.cts_api import CTSAPIError
    from biomcp.organizations import search_organizations
    from biomcp.organizations.search import format_organization_results

    try:
        results = await search_organizations(
            name=name,
            org_type=organization_type,
            city=city,
            state=state,
            page_size=page_size,
            page=page,
            api_key=api_key,
        )
        return format_organization_results(results)
    except CTSAPIError as e:
        # Check for Elasticsearch bucket limit error
        error_msg = str(e)
        if "too_many_buckets_exception" in error_msg or "75000" in error_msg:
            return (
                "⚠️ **Search Too Broad**\n\n"
                "The NCI API cannot process this search because it returns too many results.\n\n"
                "**To fix this, try:**\n"
                "1. **Always use city AND state together** for location searches\n"
                "2. Add an organization name (even partial) to narrow results\n"
                "3. Use multiple filters together (name + location, or name + type)\n\n"
                "**Examples that work:**\n"
                "- `nci_organization_searcher(city='Cleveland', state='OH')`\n"
                "- `nci_organization_searcher(name='Cleveland Clinic')`\n"
                "- `nci_organization_searcher(name='cancer', city='Boston', state='MA')`\n"
                "- `nci_organization_searcher(organization_type='Academic', city='Houston', state='TX')`"
            )
        raise


@mcp_app.tool()
@track_performance("biomcp.nci_organization_getter")
async def nci_organization_getter(
    organization_id: Annotated[
        str,
        Field(description="NCI organization ID (e.g., 'NCI-2011-03337')"),
    ],
    api_key: Annotated[
        str | None,
        Field(
            description="NCI API key. Check if user mentioned 'my NCI API key is...' in their message. If not provided here and no env var is set, user will be prompted to provide one."
        ),
    ] = None,
) -> str:
    """Get detailed information about a specific organization from NCI.

    Retrieves comprehensive details about an organization including:
    - Full name and aliases
    - Address and contact information
    - Organization type and role
    - Associated clinical trials
    - Research focus areas

    Requires NCI API key from: https://clinicaltrialsapi.cancer.gov/

    Example usage:
    - Get details about a specific cancer center
    - Find contact information for trial sponsors
    - View organization's trial portfolio
    """
    from biomcp.organizations import get_organization
    from biomcp.organizations.getter import format_organization_details

    org_data = await get_organization(
        org_id=organization_id,
        api_key=api_key,
    )

    return format_organization_details(org_data)


@mcp_app.tool()
@track_performance("biomcp.nci_intervention_searcher")
async def nci_intervention_searcher(
    name: Annotated[
        str | None,
        Field(
            description="Intervention name to search for (e.g., 'pembrolizumab')"
        ),
    ] = None,
    intervention_type: Annotated[
        str | None,
        Field(
            description="Type of intervention: 'Drug', 'Device', 'Biological', 'Procedure', 'Radiation', 'Behavioral', 'Genetic', 'Dietary', 'Other'"
        ),
    ] = None,
    synonyms: Annotated[
        bool,
        Field(description="Include synonym matches in search"),
    ] = True,
    api_key: Annotated[
        str | None,
        Field(
            description="NCI API key. Check if user mentioned 'my NCI API key is...' in their message. If not provided here and no env var is set, user will be prompted to provide one."
        ),
    ] = None,
    page: Annotated[
        int,
        Field(description="Page number (1-based)", ge=1),
    ] = 1,
    page_size: Annotated[
        int | None,
        Field(
            description="Results per page. If not specified, returns all matching results.",
            ge=1,
            le=100,
        ),
    ] = None,
) -> str:
    """Search for interventions in the NCI Clinical Trials database.

    Searches the National Cancer Institute's curated database of interventions
    used in cancer clinical trials. This includes:
    - FDA-approved drugs
    - Investigational agents
    - Medical devices
    - Surgical procedures
    - Radiation therapies
    - Behavioral interventions

    Requires NCI API key from: https://clinicaltrialsapi.cancer.gov/

    Example usage:
    - Find all trials using pembrolizumab
    - Search for CAR-T cell therapies
    - List radiation therapy protocols
    - Find dietary interventions
    """
    from biomcp.integrations.cts_api import CTSAPIError
    from biomcp.interventions import search_interventions
    from biomcp.interventions.search import format_intervention_results

    try:
        results = await search_interventions(
            name=name,
            intervention_type=intervention_type,
            synonyms=synonyms,
            page_size=page_size,
            page=page,
            api_key=api_key,
        )
        return format_intervention_results(results)
    except CTSAPIError as e:
        # Check for Elasticsearch bucket limit error
        error_msg = str(e)
        if "too_many_buckets_exception" in error_msg or "75000" in error_msg:
            return (
                "⚠️ **Search Too Broad**\n\n"
                "The NCI API cannot process this search because it returns too many results.\n\n"
                "**Try adding more specific filters:**\n"
                "- Add an intervention name (even partial)\n"
                "- Specify an intervention type (e.g., 'Drug', 'Device')\n"
                "- Search for a specific drug or therapy name\n\n"
                "**Example searches that work better:**\n"
                "- Search for 'pembrolizumab' instead of all drugs\n"
                "- Search for 'CAR-T' to find CAR-T cell therapies\n"
                "- Filter by type: Drug, Device, Procedure, etc."
            )
        raise


@mcp_app.tool()
@track_performance("biomcp.nci_intervention_getter")
async def nci_intervention_getter(
    intervention_id: Annotated[
        str,
        Field(description="NCI intervention ID (e.g., 'INT123456')"),
    ],
    api_key: Annotated[
        str | None,
        Field(
            description="NCI API key. Check if user mentioned 'my NCI API key is...' in their message. If not provided here and no env var is set, user will be prompted to provide one."
        ),
    ] = None,
) -> str:
    """Get detailed information about a specific intervention from NCI.

    Retrieves comprehensive details about an intervention including:
    - Full name and synonyms
    - Intervention type and category
    - Mechanism of action (for drugs)
    - FDA approval status
    - Associated clinical trials
    - Combination therapies

    Requires NCI API key from: https://clinicaltrialsapi.cancer.gov/

    Example usage:
    - Get details about a specific drug
    - Find all trials using a device
    - View combination therapy protocols
    """
    from biomcp.interventions import get_intervention
    from biomcp.interventions.getter import format_intervention_details

    intervention_data = await get_intervention(
        intervention_id=intervention_id,
        api_key=api_key,
    )

    return format_intervention_details(intervention_data)


# Biomarker Tools
@mcp_app.tool()
@track_performance("biomcp.nci_biomarker_searcher")
async def nci_biomarker_searcher(
    name: Annotated[
        str | None,
        Field(
            description="Biomarker name to search for (e.g., 'PD-L1', 'EGFR mutation')"
        ),
    ] = None,
    biomarker_type: Annotated[
        str | None,
        Field(description="Type of biomarker ('reference_gene' or 'branch')"),
    ] = None,
    api_key: Annotated[
        str | None,
        Field(
            description="NCI API key. Check if user mentioned 'my NCI API key is...' in their message. If not provided here and no env var is set, user will be prompted to provide one."
        ),
    ] = None,
    page: Annotated[
        int,
        Field(description="Page number (1-based)", ge=1),
    ] = 1,
    page_size: Annotated[
        int,
        Field(description="Results per page", ge=1, le=100),
    ] = 20,
) -> str:
    """Search for biomarkers in the NCI Clinical Trials database.

    Searches for biomarkers used in clinical trial eligibility criteria.
    This is essential for precision medicine trials that select patients
    based on specific biomarker characteristics.

    Biomarker examples:
    - Gene mutations (e.g., BRAF V600E, EGFR T790M)
    - Protein expression (e.g., PD-L1 ≥ 50%, HER2 positive)
    - Gene fusions (e.g., ALK fusion, ROS1 fusion)
    - Other molecular markers (e.g., MSI-H, TMB-high)

    Requires NCI API key from: https://clinicaltrialsapi.cancer.gov/

    Note: Biomarker data availability may be limited in CTRP.
    Results focus on biomarkers used in trial eligibility criteria.

    Example usage:
    - Search for PD-L1 expression biomarkers
    - Find trials requiring EGFR mutations
    - Look up biomarkers tested by NGS
    - Search for HER2 expression markers
    """
    from biomcp.biomarkers import search_biomarkers
    from biomcp.biomarkers.search import format_biomarker_results
    from biomcp.integrations.cts_api import CTSAPIError

    try:
        results = await search_biomarkers(
            name=name,
            biomarker_type=biomarker_type,
            page_size=page_size,
            page=page,
            api_key=api_key,
        )
        return format_biomarker_results(results)
    except CTSAPIError as e:
        # Check for Elasticsearch bucket limit error
        error_msg = str(e)
        if "too_many_buckets_exception" in error_msg or "75000" in error_msg:
            return (
                "⚠️ **Search Too Broad**\n\n"
                "The NCI API cannot process this search because it returns too many results.\n\n"
                "**Try adding more specific filters:**\n"
                "- Add a biomarker name (even partial)\n"
                "- Specify a gene symbol\n"
                "- Add an assay type (e.g., 'IHC', 'NGS')\n\n"
                "**Example searches that work:**\n"
                "- `nci_biomarker_searcher(name='PD-L1')`\n"
                "- `nci_biomarker_searcher(gene='EGFR', biomarker_type='mutation')`\n"
                "- `nci_biomarker_searcher(assay_type='IHC')`"
            )
        raise


# NCI Disease Tools
@mcp_app.tool()
@track_performance("biomcp.nci_disease_searcher")
async def nci_disease_searcher(
    name: Annotated[
        str | None,
        Field(description="Disease name to search for (partial match)"),
    ] = None,
    include_synonyms: Annotated[
        bool,
        Field(description="Include synonym matches in search"),
    ] = True,
    category: Annotated[
        str | None,
        Field(description="Disease category/type filter"),
    ] = None,
    api_key: Annotated[
        str | None,
        Field(
            description="NCI API key. Check if user mentioned 'my NCI API key is...' in their message. If not provided here and no env var is set, user will be prompted to provide one."
        ),
    ] = None,
    page: Annotated[
        int,
        Field(description="Page number (1-based)", ge=1),
    ] = 1,
    page_size: Annotated[
        int,
        Field(description="Results per page", ge=1, le=100),
    ] = 20,
) -> str:
    """Search NCI's controlled vocabulary of cancer conditions.

    Searches the National Cancer Institute's curated database of cancer
    conditions and diseases used in clinical trials. This is different from
    the general disease_getter tool which uses MyDisease.info.

    NCI's disease vocabulary provides:
    - Official cancer terminology used in trials
    - Disease synonyms and alternative names
    - Hierarchical disease classifications
    - Standardized disease codes for trial matching

    Requires NCI API key from: https://clinicaltrialsapi.cancer.gov/

    Example usage:
    - Search for specific cancer types (e.g., "melanoma")
    - Find all lung cancer subtypes
    - Look up official names for disease synonyms
    - Get standardized disease terms for trial searches

    Note: This is specifically for NCI's cancer disease vocabulary.
    For general disease information, use the disease_getter tool.
    """
    from biomcp.diseases import search_diseases
    from biomcp.diseases.search import format_disease_results
    from biomcp.integrations.cts_api import CTSAPIError

    try:
        results = await search_diseases(
            name=name,
            include_synonyms=include_synonyms,
            category=category,
            page_size=page_size,
            page=page,
            api_key=api_key,
        )
        return format_disease_results(results)
    except CTSAPIError as e:
        # Check for Elasticsearch bucket limit error
        error_msg = str(e)
        if "too_many_buckets_exception" in error_msg or "75000" in error_msg:
            return (
                "⚠️ **Search Too Broad**\n\n"
                "The NCI API cannot process this search because it returns too many results.\n\n"
                "**Try adding more specific filters:**\n"
                "- Add a disease name (even partial)\n"
                "- Specify a disease category\n"
                "- Use more specific search terms\n\n"
                "**Example searches that work:**\n"
                "- `nci_disease_searcher(name='melanoma')`\n"
                "- `nci_disease_searcher(name='lung', category='maintype')`\n"
                "- `nci_disease_searcher(name='NSCLC')`"
            )
        raise


# OpenFDA Tools
@mcp_app.tool()
@track_performance("biomcp.openfda_adverse_searcher")
async def openfda_adverse_searcher(
    drug: Annotated[
        str | None,
        Field(description="Drug name to search for adverse events"),
    ] = None,
    reaction: Annotated[
        str | None,
        Field(description="Adverse reaction term to search for"),
    ] = None,
    serious: Annotated[
        bool | None,
        Field(description="Filter for serious events only"),
    ] = None,
    limit: Annotated[
        int,
        Field(description="Maximum number of results", ge=1, le=100),
    ] = 25,
    page: Annotated[
        int,
        Field(description="Page number (1-based)", ge=1),
    ] = 1,
    api_key: Annotated[
        str | None,
        Field(
            description="Optional OpenFDA API key (overrides OPENFDA_API_KEY env var)"
        ),
    ] = None,
) -> str:
    """Search FDA adverse event reports (FAERS) for drug safety information.

    ⚠️ PREREQUISITE: Use the 'think' tool FIRST to plan your research strategy!

    Searches FDA's Adverse Event Reporting System for:
    - Drug side effects and adverse reactions
    - Serious event reports (death, hospitalization, disability)
    - Safety signal patterns across patient populations

    Note: These reports do not establish causation - they are voluntary reports
    that may contain incomplete or unverified information.
    """
    from biomcp.openfda import search_adverse_events

    skip = (page - 1) * limit
    return await search_adverse_events(
        drug=drug,
        reaction=reaction,
        serious=serious,
        limit=limit,
        skip=skip,
        api_key=api_key,
    )


@mcp_app.tool()
@track_performance("biomcp.openfda_adverse_getter")
async def openfda_adverse_getter(
    report_id: Annotated[
        str,
        Field(description="Safety report ID"),
    ],
    api_key: Annotated[
        str | None,
        Field(
            description="Optional OpenFDA API key (overrides OPENFDA_API_KEY env var)"
        ),
    ] = None,
) -> str:
    """Get detailed information for a specific FDA adverse event report.

    Retrieves complete details including:
    - Patient demographics and medical history
    - All drugs involved and dosages
    - Complete list of adverse reactions
    - Event narrative and outcomes
    - Reporter information
    """
    from biomcp.openfda import get_adverse_event

    return await get_adverse_event(report_id, api_key=api_key)


@mcp_app.tool()
@track_performance("biomcp.openfda_label_searcher")
async def openfda_label_searcher(
    name: Annotated[
        str | None,
        Field(description="Drug name to search for"),
    ] = None,
    indication: Annotated[
        str | None,
        Field(description="Search for drugs indicated for this condition"),
    ] = None,
    boxed_warning: Annotated[
        bool,
        Field(description="Filter for drugs with boxed warnings"),
    ] = False,
    section: Annotated[
        str | None,
        Field(
            description="Specific label section (e.g., 'contraindications', 'warnings')"
        ),
    ] = None,
    limit: Annotated[
        int,
        Field(description="Maximum number of results", ge=1, le=100),
    ] = 25,
    page: Annotated[
        int,
        Field(description="Page number (1-based)", ge=1),
    ] = 1,
    api_key: Annotated[
        str | None,
        Field(
            description="Optional OpenFDA API key (overrides OPENFDA_API_KEY env var)"
        ),
    ] = None,
) -> str:
    """Search FDA drug product labels (SPL) for prescribing information.

    ⚠️ PREREQUISITE: Use the 'think' tool FIRST to plan your research strategy!

    Searches official FDA drug labels for:
    - Approved indications and usage
    - Dosage and administration guidelines
    - Contraindications and warnings
    - Drug interactions and adverse reactions
    - Special population considerations

    Label sections include: indications, dosage, contraindications, warnings,
    adverse, interactions, pregnancy, pediatric, geriatric, overdose
    """
    from biomcp.openfda import search_drug_labels

    skip = (page - 1) * limit
    return await search_drug_labels(
        name=name,
        indication=indication,
        boxed_warning=boxed_warning,
        section=section,
        limit=limit,
        skip=skip,
        api_key=api_key,
    )


@mcp_app.tool()
@track_performance("biomcp.openfda_label_getter")
async def openfda_label_getter(
    set_id: Annotated[
        str,
        Field(description="Label set ID"),
    ],
    sections: Annotated[
        list[str] | None,
        Field(
            description="Specific sections to retrieve (default: key sections)"
        ),
    ] = None,
    api_key: Annotated[
        str | None,
        Field(
            description="Optional OpenFDA API key (overrides OPENFDA_API_KEY env var)"
        ),
    ] = None,
) -> str:
    """Get complete FDA drug label information by set ID.

    Retrieves the full prescribing information including:
    - Complete indications and usage text
    - Detailed dosing instructions
    - All warnings and precautions
    - Clinical pharmacology and studies
    - Manufacturing and storage information

    Specify sections to retrieve specific parts, or leave empty for default key sections.
    """
    from biomcp.openfda import get_drug_label

    return await get_drug_label(set_id, sections, api_key=api_key)


@mcp_app.tool()
@track_performance("biomcp.openfda_device_searcher")
async def openfda_device_searcher(
    device: Annotated[
        str | None,
        Field(description="Device name to search for"),
    ] = None,
    manufacturer: Annotated[
        str | None,
        Field(description="Manufacturer name"),
    ] = None,
    problem: Annotated[
        str | None,
        Field(description="Device problem description"),
    ] = None,
    product_code: Annotated[
        str | None,
        Field(description="FDA product code"),
    ] = None,
    genomics_only: Annotated[
        bool,
        Field(description="Filter to genomic/diagnostic devices only"),
    ] = True,
    limit: Annotated[
        int,
        Field(description="Maximum number of results", ge=1, le=100),
    ] = 25,
    page: Annotated[
        int,
        Field(description="Page number (1-based)", ge=1),
    ] = 1,
    api_key: Annotated[
        str | None,
        Field(
            description="Optional OpenFDA API key (overrides OPENFDA_API_KEY env var)"
        ),
    ] = None,
) -> str:
    """Search FDA device adverse event reports (MAUDE) for medical device issues.

    ⚠️ PREREQUISITE: Use the 'think' tool FIRST to plan your research strategy!

    Searches FDA's device adverse event database for:
    - Device malfunctions and failures
    - Patient injuries related to devices
    - Genomic test and diagnostic device issues

    By default, filters to genomic/diagnostic devices relevant to precision medicine.
    Set genomics_only=False to search all medical devices.
    """
    from biomcp.openfda import search_device_events

    skip = (page - 1) * limit
    return await search_device_events(
        device=device,
        manufacturer=manufacturer,
        problem=problem,
        product_code=product_code,
        genomics_only=genomics_only,
        limit=limit,
        skip=skip,
        api_key=api_key,
    )


@mcp_app.tool()
@track_performance("biomcp.openfda_device_getter")
async def openfda_device_getter(
    mdr_report_key: Annotated[
        str,
        Field(description="MDR report key"),
    ],
    api_key: Annotated[
        str | None,
        Field(
            description="Optional OpenFDA API key (overrides OPENFDA_API_KEY env var)"
        ),
    ] = None,
) -> str:
    """Get detailed information for a specific FDA device event report.

    Retrieves complete device event details including:
    - Device identification and specifications
    - Complete event narrative
    - Patient outcomes and impacts
    - Manufacturer analysis and actions
    - Remedial actions taken
    """
    from biomcp.openfda import get_device_event

    return await get_device_event(mdr_report_key, api_key=api_key)


@mcp_app.tool()
@track_performance("biomcp.openfda_approval_searcher")
async def openfda_approval_searcher(
    drug: Annotated[
        str | None,
        Field(description="Drug name (brand or generic) to search for"),
    ] = None,
    application_number: Annotated[
        str | None,
        Field(description="NDA or BLA application number"),
    ] = None,
    approval_year: Annotated[
        str | None,
        Field(description="Year of approval (YYYY format)"),
    ] = None,
    limit: Annotated[
        int,
        Field(description="Maximum number of results", ge=1, le=100),
    ] = 25,
    page: Annotated[
        int,
        Field(description="Page number (1-based)", ge=1),
    ] = 1,
    api_key: Annotated[
        str | None,
        Field(
            description="Optional OpenFDA API key (overrides OPENFDA_API_KEY env var)"
        ),
    ] = None,
) -> str:
    """Search FDA drug approval records from Drugs@FDA database.

    ⚠️ PREREQUISITE: Use the 'think' tool FIRST to plan your research strategy!

    Returns information about:
    - Application numbers and sponsors
    - Brand and generic names
    - Product formulations and strengths
    - Marketing status and approval dates
    - Submission history

    Useful for verifying if a drug is FDA-approved and when.
    """
    from biomcp.openfda import search_drug_approvals

    skip = (page - 1) * limit
    return await search_drug_approvals(
        drug=drug,
        application_number=application_number,
        approval_year=approval_year,
        limit=limit,
        skip=skip,
        api_key=api_key,
    )


@mcp_app.tool()
@track_performance("biomcp.openfda_approval_getter")
async def openfda_approval_getter(
    application_number: Annotated[
        str,
        Field(description="NDA or BLA application number"),
    ],
    api_key: Annotated[
        str | None,
        Field(
            description="Optional OpenFDA API key (overrides OPENFDA_API_KEY env var)"
        ),
    ] = None,
) -> str:
    """Get detailed FDA drug approval information for a specific application.

    Returns comprehensive approval details including:
    - Full product list with dosage forms and strengths
    - Complete submission history
    - Marketing status timeline
    - Therapeutic equivalence codes
    - Pharmacologic class information
    """
    from biomcp.openfda import get_drug_approval

    return await get_drug_approval(application_number, api_key=api_key)


@mcp_app.tool()
@track_performance("biomcp.openfda_recall_searcher")
async def openfda_recall_searcher(
    drug: Annotated[
        str | None,
        Field(description="Drug name to search for recalls"),
    ] = None,
    recall_class: Annotated[
        str | None,
        Field(
            description="Recall classification (1=most serious, 2=moderate, 3=least serious)"
        ),
    ] = None,
    status: Annotated[
        str | None,
        Field(description="Recall status (ongoing, completed, terminated)"),
    ] = None,
    reason: Annotated[
        str | None,
        Field(description="Search text in recall reason"),
    ] = None,
    since_date: Annotated[
        str | None,
        Field(description="Show recalls after this date (YYYYMMDD format)"),
    ] = None,
    limit: Annotated[
        int,
        Field(description="Maximum number of results", ge=1, le=100),
    ] = 25,
    page: Annotated[
        int,
        Field(description="Page number (1-based)", ge=1),
    ] = 1,
    api_key: Annotated[
        str | None,
        Field(
            description="Optional OpenFDA API key (overrides OPENFDA_API_KEY env var)"
        ),
    ] = None,
) -> str:
    """Search FDA drug recall records from the Enforcement database.

    ⚠️ PREREQUISITE: Use the 'think' tool FIRST to plan your research strategy!

    Returns recall information including:
    - Classification (Class I, II, or III)
    - Recall reason and description
    - Product identification
    - Distribution information
    - Recalling firm details
    - Current status

    Class I = most serious (death/serious harm)
    Class II = moderate (temporary/reversible harm)
    Class III = least serious (unlikely to cause harm)
    """
    from biomcp.openfda import search_drug_recalls

    skip = (page - 1) * limit
    return await search_drug_recalls(
        drug=drug,
        recall_class=recall_class,
        status=status,
        reason=reason,
        since_date=since_date,
        limit=limit,
        skip=skip,
        api_key=api_key,
    )


@mcp_app.tool()
@track_performance("biomcp.openfda_recall_getter")
async def openfda_recall_getter(
    recall_number: Annotated[
        str,
        Field(description="FDA recall number"),
    ],
    api_key: Annotated[
        str | None,
        Field(
            description="Optional OpenFDA API key (overrides OPENFDA_API_KEY env var)"
        ),
    ] = None,
) -> str:
    """Get detailed FDA drug recall information for a specific recall.

    Returns complete recall details including:
    - Full product description and code information
    - Complete reason for recall
    - Distribution pattern and locations
    - Quantity of product recalled
    - Firm information and actions taken
    - Timeline of recall events
    """
    from biomcp.openfda import get_drug_recall

    return await get_drug_recall(recall_number, api_key=api_key)


@mcp_app.tool()
@track_performance("biomcp.openfda_shortage_searcher")
async def openfda_shortage_searcher(
    drug: Annotated[
        str | None,
        Field(description="Drug name (generic or brand) to search"),
    ] = None,
    status: Annotated[
        str | None,
        Field(description="Shortage status (current or resolved)"),
    ] = None,
    therapeutic_category: Annotated[
        str | None,
        Field(
            description="Therapeutic category (e.g., Oncology, Anti-infective)"
        ),
    ] = None,
    limit: Annotated[
        int,
        Field(description="Maximum number of results", ge=1, le=100),
    ] = 25,
    page: Annotated[
        int,
        Field(description="Page number (1-based)", ge=1),
    ] = 1,
    api_key: Annotated[
        str | None,
        Field(
            description="Optional OpenFDA API key (overrides OPENFDA_API_KEY env var)"
        ),
    ] = None,
) -> str:
    """Search FDA drug shortage records.

    ⚠️ PREREQUISITE: Use the 'think' tool FIRST to plan your research strategy!

    Returns shortage information including:
    - Current shortage status
    - Shortage start and resolution dates
    - Reason for shortage
    - Therapeutic category
    - Manufacturer information
    - Estimated resolution timeline

    Note: Shortage data is cached and updated periodically.
    Check FDA.gov for most current information.
    """
    from biomcp.openfda import search_drug_shortages

    skip = (page - 1) * limit
    return await search_drug_shortages(
        drug=drug,
        status=status,
        therapeutic_category=therapeutic_category,
        limit=limit,
        skip=skip,
        api_key=api_key,
    )


@mcp_app.tool()
@track_performance("biomcp.openfda_shortage_getter")
async def openfda_shortage_getter(
    drug: Annotated[
        str,
        Field(description="Drug name (generic or brand)"),
    ],
    api_key: Annotated[
        str | None,
        Field(
            description="Optional OpenFDA API key (overrides OPENFDA_API_KEY env var)"
        ),
    ] = None,
) -> str:
    """Get detailed FDA drug shortage information for a specific drug.

    Returns comprehensive shortage details including:
    - Complete timeline of shortage
    - Detailed reason for shortage
    - All affected manufacturers
    - Alternative products if available
    - Resolution status and estimates
    - Additional notes and recommendations

    Data is updated periodically from FDA shortage database.
    """
    from biomcp.openfda import get_drug_shortage

    return await get_drug_shortage(drug, api_key=api_key)

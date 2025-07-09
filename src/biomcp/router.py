"""Unified search and fetch tools for BioMCP.

This module provides the main MCP tools for searching and fetching biomedical data
across different domains (articles, trials, variants) with integrated sequential
thinking capabilities.
"""

import json
import logging
from typing import Annotated, Any, Literal

from pydantic import Field

from biomcp.constants import (
    DEFAULT_PAGE_NUMBER,
    DEFAULT_PAGE_SIZE,
    DEFAULT_TITLE,
    ERROR_DOMAIN_REQUIRED,
    ESTIMATED_ADDITIONAL_RESULTS,
    MAX_RESULTS_PER_DOMAIN_DEFAULT,
    TRIAL_DETAIL_SECTIONS,
    VALID_DOMAINS,
)
from biomcp.core import mcp_app
from biomcp.domain_handlers import get_domain_handler
from biomcp.exceptions import (
    InvalidDomainError,
    InvalidParameterError,
    QueryParsingError,
    ResultParsingError,
    SearchExecutionError,
)
from biomcp.metrics import track_performance
from biomcp.parameter_parser import ParameterParser
from biomcp.query_parser import QueryParser
from biomcp.query_router import QueryRouter, execute_routing_plan
from biomcp.thinking_tracker import get_thinking_reminder
from biomcp.trials import getter as trial_getter

logger = logging.getLogger(__name__)


def format_results(
    results: list[dict], domain: str, page: int, page_size: int, total: int
) -> dict:
    """Format search results according to OpenAI MCP search semantics.

    Converts domain-specific result formats into a standardized structure with:
    - id: Unique identifier for the result (required)
    - title: Human-readable title (required)
    - text: Brief preview or summary of the content (required)
    - url: Link to the full resource (optional but recommended for citations)

    Note: The OpenAI MCP specification does NOT require metadata in search results.
    Metadata should only be included in fetch results.

    Args:
        results: Raw results from domain-specific search
        domain: Type of results ('article', 'trial', or 'variant')
        page: Current page number (for internal tracking only)
        page_size: Number of results per page (for internal tracking only)
        total: Total number of results available (for internal tracking only)

    Returns:
        Dictionary with results array following OpenAI MCP format:
        {"results": [{"id", "title", "text", "url"}, ...]}

    Raises:
        InvalidDomainError: If domain is not recognized
    """
    logger.debug(f"Formatting {len(results)} results for domain: {domain}")

    formatted_data = []

    # Get the appropriate handler
    try:
        handler_class = get_domain_handler(domain)
    except ValueError:
        raise InvalidDomainError(domain, VALID_DOMAINS) from None

    # Format each result
    for result in results:
        try:
            formatted_result = handler_class.format_result(result)
            # Ensure the result has the required OpenAI MCP fields
            openai_result = {
                "id": formatted_result.get("id", ""),
                "title": formatted_result.get("title", DEFAULT_TITLE),
                "text": formatted_result.get(
                    "snippet", formatted_result.get("text", "")
                ),
                "url": formatted_result.get("url", ""),
            }
            # Note: OpenAI MCP spec doesn't require metadata in search results
            # Only include it if explicitly needed for enhanced functionality
            formatted_data.append(openai_result)
        except Exception as e:
            logger.warning(f"Failed to format result in domain {domain}: {e}")
            # Skip malformed results
            continue

    # Add thinking reminder if needed (as first result)
    reminder = get_thinking_reminder()
    if reminder and formatted_data:
        reminder_result = {
            "id": "thinking-reminder",
            "title": "⚠️ Research Best Practice Reminder",
            "text": reminder,
            "url": "",
        }
        formatted_data.insert(0, reminder_result)

    # Return OpenAI MCP compliant format
    return {"results": formatted_data}


# ────────────────────────────
# Unified SEARCH tool
# ────────────────────────────
@mcp_app.tool()
@track_performance("biomcp.search")
async def search(  # noqa: C901
    query: Annotated[
        str,
        "Unified search query (e.g., 'gene:BRAF AND trials.condition:melanoma'). If provided, other parameters are ignored.",
    ],
    call_benefit: Annotated[
        str | None,
        Field(
            description="Brief explanation of why this search is being performed and expected benefit. Helps improve search accuracy and provides context for analytics. Highly recommended for better results."
        ),
    ] = None,
    domain: Annotated[
        Literal["article", "trial", "variant"] | None,
        Field(
            description="Domain to search: 'article' for papers/literature ABOUT genes/variants/diseases, 'trial' for clinical studies, 'variant' for genetic variant DATABASE RECORDS (NOT articles about variants)"
        ),
    ] = None,
    genes: Annotated[list[str] | str | None, "Gene symbols"] = None,
    diseases: Annotated[list[str] | str | None, "Disease terms"] = None,
    variants: Annotated[list[str] | str | None, "Variant strings"] = None,
    chemicals: Annotated[list[str] | str | None, "Drug/chemical terms"] = None,
    keywords: Annotated[list[str] | str | None, "Free-text keywords"] = None,
    conditions: Annotated[list[str] | str | None, "Trial conditions"] = None,
    interventions: Annotated[
        list[str] | str | None, "Trial interventions"
    ] = None,
    recruiting_status: Annotated[
        str | None, "Trial status filter (OPEN, CLOSED, or ANY)"
    ] = None,
    phase: Annotated[str | None, "Trial phase filter"] = None,
    significance: Annotated[
        str | None, "Variant clinical significance"
    ] = None,
    lat: Annotated[
        float | None,
        "Latitude for trial location search. AI agents should geocode city names (e.g., 'Cleveland' → 41.4993) before using.",
    ] = None,
    long: Annotated[
        float | None,
        "Longitude for trial location search. AI agents should geocode city names (e.g., 'Cleveland' → -81.6944) before using.",
    ] = None,
    distance: Annotated[
        int | None,
        "Distance in miles from lat/long for trial search (default: 50 miles if lat/long provided)",
    ] = None,
    page: Annotated[int, "Page number (minimum: 1)"] = DEFAULT_PAGE_NUMBER,
    page_size: Annotated[int, "Results per page (1-100)"] = DEFAULT_PAGE_SIZE,
    max_results_per_domain: Annotated[
        int | None, "Max results per domain (unified search only)"
    ] = None,
    explain_query: Annotated[
        bool, "Return query explanation (unified search only)"
    ] = False,
    get_schema: Annotated[
        bool, "Return searchable fields schema instead of results"
    ] = False,
) -> dict:
    """Search biomedical literature, clinical trials, and genetic variants.

    ⚠️ IMPORTANT: Have you used the 'think' tool first? If not, STOP and use it NOW!
    The 'think' tool is REQUIRED for proper research planning and should be your FIRST step.

    This tool provides access to biomedical data from PubMed/PubTator3, ClinicalTrials.gov,
    and MyVariant.info. It supports two search modes:

    ## 1. UNIFIED QUERY LANGUAGE
    Use the 'query' parameter with field-based syntax for precise cross-domain searches.

    Syntax:
    - Basic: "gene:BRAF"
    - AND logic: "gene:BRAF AND disease:melanoma"
    - OR logic: "gene:PTEN AND (R173 OR Arg173 OR 'position 173')"
    - Domain-specific: "trials.condition:melanoma AND trials.phase:3"

    Common fields:
    - Cross-domain: gene, disease, variant, chemical/drug
    - Articles: pmid, title, abstract, journal, author
    - Trials: trials.condition, trials.intervention, trials.phase, trials.status
    - Variants: variants.hgvs, variants.rsid, variants.significance

    Example:
    ```
    await search(
        query="gene:BRAF AND disease:melanoma AND trials.phase:3",
        max_results_per_domain=20
    )
    ```

    ## 2. DOMAIN-SPECIFIC SEARCH
    Use the 'domain' parameter with specific filters for targeted searches.

    Domains:
    - "article": Search PubMed/PubTator3 for research articles and preprints ABOUT genes, variants, diseases, or chemicals
    - "trial": Search ClinicalTrials.gov for clinical studies
    - "variant": Search MyVariant.info for genetic variant DATABASE RECORDS (population frequency, clinical significance, etc.) - NOT for articles about variants!

    Example:
    ```
    await search(
        domain="article",
        genes=["BRAF", "NRAS"],
        diseases=["melanoma"],
        page_size=50
    )
    ```

    ## DOMAIN SELECTION EXAMPLES:
    - To find ARTICLES about BRAF V600E mutation: domain="article", genes=["BRAF"], variants=["V600E"]
    - To find VARIANT DATA for BRAF mutations: domain="variant", gene="BRAF"
    - To find articles about ERBB2 p.D277Y: domain="article", genes=["ERBB2"], variants=["p.D277Y"]
    - Common mistake: Using domain="variant" when you want articles about a variant

    ## IMPORTANT NOTES:
    - For complex research questions, use the separate 'think' tool for systematic analysis
    - The tool returns results in OpenAI MCP format: {"results": [{"id", "title", "text", "url"}, ...]}
    - Search results do NOT include metadata (per OpenAI MCP specification)
    - Use the fetch tool to get detailed metadata for specific records
    - Use get_schema=True to explore available search fields
    - Use explain_query=True to understand query parsing (unified mode)
    - Domain-specific searches use AND logic for multiple values
    - For OR logic, use the unified query language
    - NEW: Article search keywords support OR with pipe separator: "R173|Arg173|p.R173"
    - Remember: domain="article" finds LITERATURE, domain="variant" finds DATABASE RECORDS

    ## RETURN FORMAT:
    All search modes return results in this format:
    ```json
    {
        "results": [
            {
                "id": "unique_identifier",
                "title": "Human-readable title",
                "text": "Summary or snippet of content",
                "url": "Link to full resource"
            }
        ]
    }
    ```
    """
    logger.info(f"Search called with domain={domain}, query={query}")

    # Return schema if requested
    if get_schema:
        parser = QueryParser()
        return parser.get_schema()

    # Determine search mode
    if query and query.strip():
        # Unified query language mode
        logger.info(f"Using unified query mode: {query}")
        return await _unified_search(
            query=query,
            max_results_per_domain=max_results_per_domain
            or MAX_RESULTS_PER_DOMAIN_DEFAULT,
            domains=None,
            explain_query=explain_query,
        )

    # Legacy domain-based search
    if not domain:
        raise InvalidParameterError(
            "query or domain", None, ERROR_DOMAIN_REQUIRED
        )

    # Validate pagination parameters
    try:
        page, page_size = ParameterParser.validate_page_params(page, page_size)
    except InvalidParameterError as e:
        logger.error(f"Invalid pagination parameters: {e}")
        raise

    # Parse parameters using ParameterParser
    genes = ParameterParser.parse_list_param(genes, "genes")
    diseases = ParameterParser.parse_list_param(diseases, "diseases")
    variants = ParameterParser.parse_list_param(variants, "variants")
    chemicals = ParameterParser.parse_list_param(chemicals, "chemicals")
    keywords = ParameterParser.parse_list_param(keywords, "keywords")
    conditions = ParameterParser.parse_list_param(conditions, "conditions")
    interventions = ParameterParser.parse_list_param(
        interventions, "interventions"
    )

    logger.debug(
        f"Parsed parameters for domain {domain}: "
        f"genes={genes}, diseases={diseases}, variants={variants}"
    )

    if domain == "article":
        from .router_handlers import handle_article_search

        items, total = await handle_article_search(
            genes=genes,
            diseases=diseases,
            variants=variants,
            chemicals=chemicals,
            keywords=keywords,
            page=page,
            page_size=page_size,
        )

        return format_results(
            items,
            domain="article",
            page=page,
            page_size=page_size,
            total=total,
        )

    elif domain == "trial":
        logger.info("Executing trial search")
        # Build the trial search parameters
        search_params: dict[str, Any] = {}
        if conditions:
            search_params["conditions"] = conditions
        if interventions:
            search_params["interventions"] = interventions
        if recruiting_status:
            search_params["recruiting_status"] = recruiting_status
        if phase:
            try:
                search_params["phase"] = ParameterParser.normalize_phase(phase)
            except InvalidParameterError:
                raise
        if keywords:
            search_params["keywords"] = keywords
        if lat is not None:
            search_params["lat"] = lat
        if long is not None:
            search_params["long"] = long
        if distance is not None:
            search_params["distance"] = distance

        try:
            from biomcp.trials.search import TrialQuery, search_trials

            # Convert search_params to TrialQuery
            trial_query = TrialQuery(**search_params, page_size=page_size)
            result_str = await search_trials(trial_query, output_json=True)
        except Exception as e:
            logger.error(f"Trial search failed: {e}")
            raise SearchExecutionError("trial", e) from e

        # Parse the JSON results
        try:
            results = json.loads(result_str)
        except (json.JSONDecodeError, TypeError) as e:
            logger.error(f"Failed to parse trial results: {e}")
            raise ResultParsingError("trial", e) from e

        # Handle different response formats from the trials API
        # The API can return either a dict with 'studies' key or a direct list
        if isinstance(results, dict):
            # ClinicalTrials.gov API v2 format with studies array
            if "studies" in results:
                items = results["studies"]
                total = len(items)  # API doesn't provide total count
            # Legacy format or error
            elif "error" in results:
                logger.warning(
                    f"Trial API returned error: {results.get('error')}"
                )
                return format_results(
                    [], domain="trial", page=page, page_size=page_size, total=0
                )
            else:
                # Assume the dict itself is a single result
                items = [results]
                total = 1
        elif isinstance(results, list):
            # Direct list of results
            items = results
            total = len(items)
        else:
            items = []
            total = 0

        logger.info(f"Trial search returned {total} total results")

        return format_results(
            items, domain="trial", page=page, page_size=page_size, total=total
        )

    elif domain == "variant":
        logger.info("Executing variant search")
        # Build the variant search parameters
        # Note: variant searcher expects single gene, not list
        gene = genes[0] if genes else None

        # Use keywords to search for significance if provided
        keyword_list = keywords or []
        if significance:
            keyword_list.append(significance)

        try:
            from biomcp.variants.search import VariantQuery, search_variants

            variant_query = VariantQuery(
                gene=gene,
                significance=significance,
                size=page_size,
                offset=(page - 1) * page_size,
            )
            result_str = await search_variants(variant_query, output_json=True)
        except Exception as e:
            logger.error(f"Variant search failed: {e}")
            raise SearchExecutionError("variant", e) from e

        # Parse the JSON results
        try:
            all_results = json.loads(result_str)
        except (json.JSONDecodeError, TypeError) as e:
            logger.error(f"Failed to parse variant results: {e}")
            raise ResultParsingError("variant", e) from e

        # For variants, the results are already paginated by the API
        # We need to estimate total based on whether we got a full page
        items = all_results if isinstance(all_results, list) else []
        # Rough estimate: if we got a full page, there might be more
        total = len(items) + (
            ESTIMATED_ADDITIONAL_RESULTS if len(items) == page_size else 0
        )

        logger.info(f"Variant search returned {len(items)} results")

        return format_results(
            items,
            domain="variant",
            page=page,
            page_size=page_size,
            total=total,
        )

    else:
        raise InvalidDomainError(domain, VALID_DOMAINS)


# ────────────────────────────
# Unified FETCH tool
# ────────────────────────────
@mcp_app.tool()
@track_performance("biomcp.fetch")
async def fetch(  # noqa: C901
    id: Annotated[str, "PMID / NCT ID / Variant ID / DOI"],  # noqa: A002
    domain: Annotated[
        Literal["article", "trial", "variant"] | None,
        Field(
            description="Domain of the record (auto-detected if not provided)"
        ),
    ] = None,
    call_benefit: Annotated[
        str | None,
        Field(
            description="Brief explanation of why this fetch is being performed and expected benefit. Helps provide context for analytics and improves result relevance."
        ),
    ] = None,
    detail: Annotated[
        Literal[
            "protocol", "locations", "outcomes", "references", "all", "full"
        ]
        | None,
        "Specific section to retrieve (trials) or 'full' (articles)",
    ] = None,
) -> dict:
    """Fetch comprehensive details for a specific biomedical record.

    This tool retrieves full information for articles, clinical trials, or genetic variants
    using their unique identifiers. It returns data in a standardized format suitable for
    detailed analysis and research.

    ## IDENTIFIER FORMATS:
    - Articles: PMID (PubMed ID) - e.g., "35271234" OR DOI - e.g., "10.1101/2024.01.20.23288905"
    - Trials: NCT ID (ClinicalTrials.gov ID) - e.g., "NCT04280705"
    - Variants: HGVS notation or dbSNP ID - e.g., "chr7:g.140453136A>T" or "rs121913254"

    The domain is automatically detected from the ID format if not provided:
    - NCT* → trial
    - Contains "/" with numeric prefix (DOI) → article
    - Pure numeric → article (PMID)
    - rs* or contains ':' or 'g.' → variant

    ## DOMAIN-SPECIFIC OPTIONS:

    ### Articles (domain="article"):
    - Returns full article metadata, abstract, and full text when available
    - Supports both PubMed articles (via PMID) and Europe PMC preprints (via DOI)
    - Includes annotations for genes, diseases, chemicals, and variants (PubMed only)
    - detail="full" attempts to retrieve full text content (PubMed only)

    ### Clinical Trials (domain="trial"):
    - detail=None or "protocol": Core study information
    - detail="locations": Study sites and contact information
    - detail="outcomes": Primary/secondary outcomes and results
    - detail="references": Related publications and citations
    - detail="all": Complete trial record with all sections

    ### Variants (domain="variant"):
    - Returns comprehensive variant information including:
      - Clinical significance and interpretations
      - Population frequencies
      - Gene/protein effects
      - External database links
    - detail parameter is ignored (always returns full data)

    ## RETURN FORMAT:
    All fetch operations return a standardized format:
    ```json
    {
        "id": "unique_identifier",
        "title": "Record title or name",
        "text": "Full content or comprehensive description",
        "url": "Link to original source",
        "metadata": {
            // Domain-specific additional fields
        }
    }
    ```

    ## EXAMPLES:

    Fetch article by PMID (domain auto-detected):
    ```
    await fetch(id="35271234")
    ```

    Fetch article by DOI (domain auto-detected):
    ```
    await fetch(id="10.1101/2024.01.20.23288905")
    ```

    Fetch complete trial information (domain auto-detected):
    ```
    await fetch(
        id="NCT04280705",
        detail="all"
    )
    ```

    Fetch variant with clinical interpretations:
    ```
    await fetch(id="rs121913254")
    ```

    Explicitly specify domain (optional):
    ```
    await fetch(
        domain="variant",
        id="chr7:g.140453136A>T"
    )
    ```
    """
    # Auto-detect domain if not provided
    if domain is None:
        # Try to infer domain from ID format
        if id.upper().startswith("NCT"):
            domain = "trial"
            logger.info(f"Auto-detected domain 'trial' from NCT ID: {id}")
        elif "/" in id and id.split("/")[0].replace(".", "").isdigit():
            # DOI format (e.g., 10.1038/nature12373) - treat as article
            domain = "article"
            logger.info(f"Auto-detected domain 'article' from DOI: {id}")
        elif id.isdigit():
            # Numeric ID - likely PMID
            domain = "article"
            logger.info(
                f"Auto-detected domain 'article' from numeric ID: {id}"
            )
        elif id.startswith("rs") or ":" in id or "g." in id:
            # rsID or HGVS notation
            domain = "variant"
            logger.info(f"Auto-detected domain 'variant' from ID format: {id}")
        else:
            # Default to article if we can't determine
            domain = "article"
            logger.warning(
                f"Could not auto-detect domain for ID '{id}', defaulting to 'article'"
            )

    logger.info(f"Fetch called for {domain} with id={id}, detail={detail}")

    if domain == "article":
        logger.debug("Fetching article details")
        try:
            from biomcp.articles.fetch import _article_details

            # The _article_details function handles both PMIDs and DOIs
            result_str = await _article_details(
                call_benefit=call_benefit
                or "Fetching article details via MCP tool",
                pmid=id,
            )
        except Exception as e:
            logger.error(f"Article fetch failed: {e}")
            raise SearchExecutionError("article", e) from e

        # Parse and return the first article
        try:
            articles = (
                json.loads(result_str)
                if isinstance(result_str, str)
                else result_str
            )
        except (json.JSONDecodeError, TypeError) as e:
            logger.error(f"Failed to parse article fetch results: {e}")
            raise ResultParsingError("article", e) from e

        if not articles:
            return {"error": "Article not found"}

        article = articles[0]

        # Check if the article is actually an error response
        if "error" in article:
            return {"error": article["error"]}

        # Format according to OpenAI MCP standard
        full_text = article.get("full_text", "")
        abstract = article.get("abstract", "")
        text_content = full_text if full_text else abstract

        return {
            "id": str(article.get("pmid", id)),
            "title": article.get("title", DEFAULT_TITLE),
            "text": text_content,
            "url": article.get(
                "url", f"https://pubmed.ncbi.nlm.nih.gov/{id}/"
            ),
            "metadata": {
                "pmid": article.get("pmid"),
                "journal": article.get("journal"),
                "authors": article.get("authors"),
                "year": article.get("year"),
                "doi": article.get("doi"),
                "annotations": article.get("annotations", {}),
                "is_preprint": article.get("is_preprint", False),
                "preprint_source": article.get("preprint_source"),
            },
        }

    elif domain == "trial":
        logger.debug(f"Fetching trial details for section: {detail}")

        # Validate detail parameter
        if detail is not None and detail not in TRIAL_DETAIL_SECTIONS:
            raise InvalidParameterError(
                "detail",
                detail,
                f"one of: {', '.join(TRIAL_DETAIL_SECTIONS)} or None",
            )

        try:
            # Always fetch protocol for basic info - get JSON format
            protocol_json = await trial_getter.get_trial(
                nct_id=id,
                module=trial_getter.Module.PROTOCOL,
                output_json=True,
            )

            # Parse the JSON response
            try:
                protocol_data = json.loads(protocol_json)
            except json.JSONDecodeError as e:
                logger.error(f"Failed to parse protocol JSON for {id}: {e}")
                return {
                    "id": id,
                    "title": f"Clinical Trial {id}",
                    "text": f"Error parsing trial data: {e}",
                    "url": f"https://clinicaltrials.gov/study/{id}",
                    "metadata": {
                        "nct_id": id,
                        "error": f"JSON parse error: {e}",
                    },
                }

            # Check for errors in the response
            if "error" in protocol_data:
                return {
                    "id": id,
                    "title": f"Clinical Trial {id}",
                    "text": protocol_data.get(
                        "details",
                        protocol_data.get("error", "Trial not found"),
                    ),
                    "url": f"https://clinicaltrials.gov/study/{id}",
                    "metadata": {
                        "nct_id": id,
                        "error": protocol_data.get("error"),
                    },
                }

            # Build comprehensive text description
            text_parts = []

            # Extract protocol section data from the API response
            protocol_section = protocol_data.get("protocolSection", {})

            # Extract basic info from the protocol section
            id_module = protocol_section.get("identificationModule", {})
            status_module = protocol_section.get("statusModule", {})
            desc_module = protocol_section.get("descriptionModule", {})
            conditions_module = protocol_section.get("conditionsModule", {})
            design_module = protocol_section.get("designModule", {})
            arms_module = protocol_section.get("armsInterventionsModule", {})

            # Add basic protocol info to text
            title = id_module.get("briefTitle", f"Clinical Trial {id}")
            text_parts.append(f"Study Title: {title}")

            # Conditions
            conditions = conditions_module.get("conditions", [])
            if conditions:
                text_parts.append(f"\nConditions: {', '.join(conditions)}")

            # Interventions
            interventions = []
            for intervention in arms_module.get("interventions", []):
                interventions.append(intervention.get("name", ""))
            if interventions:
                text_parts.append(f"Interventions: {', '.join(interventions)}")

            # Phase
            phases = design_module.get("phases", [])
            if phases:
                text_parts.append(f"Phase: {', '.join(phases)}")

            # Status
            overall_status = status_module.get("overallStatus", "N/A")
            text_parts.append(f"Status: {overall_status}")

            # Summary
            brief_summary = desc_module.get(
                "briefSummary", "No summary available"
            )
            text_parts.append(f"\nSummary: {brief_summary}")

            # Prepare metadata
            metadata = {"nct_id": id, "protocol": protocol_data}

            if detail in ("all", "locations", "outcomes", "references"):
                # Fetch additional sections as needed
                if detail == "all" or detail == "locations":
                    try:
                        locations_json = await trial_getter.get_trial(
                            nct_id=id,
                            module=trial_getter.Module.LOCATIONS,
                            output_json=True,
                        )
                        locations_data = json.loads(locations_json)
                        if "error" not in locations_data:
                            # Extract locations from the protocol section
                            locations_module = locations_data.get(
                                "protocolSection", {}
                            ).get("contactsLocationsModule", {})
                            locations_list = locations_module.get(
                                "locations", []
                            )
                            metadata["locations"] = locations_list
                            if locations_list:
                                text_parts.append(
                                    f"\n\nLocations: {len(locations_list)} study sites"
                                )
                    except Exception as e:
                        logger.warning(
                            f"Failed to fetch locations for {id}: {e}"
                        )
                        metadata["locations"] = []

                if detail == "all" or detail == "outcomes":
                    try:
                        outcomes_json = await trial_getter.get_trial(
                            nct_id=id,
                            module=trial_getter.Module.OUTCOMES,
                            output_json=True,
                        )
                        outcomes_data = json.loads(outcomes_json)
                        if "error" not in outcomes_data:
                            # Extract outcomes from the protocol section
                            outcomes_module = outcomes_data.get(
                                "protocolSection", {}
                            ).get("outcomesModule", {})
                            primary_outcomes = outcomes_module.get(
                                "primaryOutcomes", []
                            )
                            secondary_outcomes = outcomes_module.get(
                                "secondaryOutcomes", []
                            )
                            metadata["outcomes"] = {
                                "primary_outcomes": primary_outcomes,
                                "secondary_outcomes": secondary_outcomes,
                            }
                            if primary_outcomes:
                                text_parts.append(
                                    f"\n\nPrimary Outcomes: {len(primary_outcomes)} measures"
                                )
                    except Exception as e:
                        logger.warning(
                            f"Failed to fetch outcomes for {id}: {e}"
                        )
                        metadata["outcomes"] = {}

                if detail == "all" or detail == "references":
                    try:
                        references_json = await trial_getter.get_trial(
                            nct_id=id,
                            module=trial_getter.Module.REFERENCES,
                            output_json=True,
                        )
                        references_data = json.loads(references_json)
                        if "error" not in references_data:
                            # Extract references from the protocol section
                            references_module = references_data.get(
                                "protocolSection", {}
                            ).get("referencesModule", {})
                            references_list = references_module.get(
                                "references", []
                            )
                            metadata["references"] = references_list
                            if references_list:
                                text_parts.append(
                                    f"\n\nReferences: {len(references_list)} publications"
                                )
                    except Exception as e:
                        logger.warning(
                            f"Failed to fetch references for {id}: {e}"
                        )
                        metadata["references"] = []

            # Return OpenAI MCP compliant format
            return {
                "id": id,
                "title": title,
                "text": "\n".join(text_parts),
                "url": f"https://clinicaltrials.gov/study/{id}",
                "metadata": metadata,
            }

        except Exception as e:
            logger.error(f"Trial fetch failed: {e}")
            raise SearchExecutionError("trial", e) from e

    elif domain == "variant":
        logger.debug("Fetching variant details")
        try:
            from biomcp.variants.getter import get_variant

            result_str = await get_variant(
                variant_id=id,
                output_json=True,
                include_external=True,
            )
        except Exception as e:
            logger.error(f"Variant fetch failed: {e}")
            raise SearchExecutionError("variant", e) from e

        try:
            variant_response = (
                json.loads(result_str)
                if isinstance(result_str, str)
                else result_str
            )
        except (json.JSONDecodeError, TypeError) as e:
            logger.error(f"Failed to parse variant fetch results: {e}")
            raise ResultParsingError("variant", e) from e

        # get_variant returns a list, extract the first variant
        if isinstance(variant_response, list) and variant_response:
            variant_data = variant_response[0]
        elif isinstance(variant_response, dict):
            variant_data = variant_response
        else:
            return {"error": "Variant not found"}

        # Build comprehensive text description
        text_parts = []

        # Basic variant info
        text_parts.append(f"Variant: {variant_data.get('_id', id)}")

        # Gene information
        if variant_data.get("gene"):
            gene_info = variant_data["gene"]
            text_parts.append(
                f"\nGene: {gene_info.get('symbol', 'Unknown')} ({gene_info.get('name', '')})"
            )

        # Clinical significance
        if variant_data.get("clinvar"):
            clinvar = variant_data["clinvar"]
            if clinvar.get("clinical_significance"):
                text_parts.append(
                    f"\nClinical Significance: {clinvar['clinical_significance']}"
                )
            if clinvar.get("review_status"):
                text_parts.append(f"Review Status: {clinvar['review_status']}")

        # dbSNP info
        if variant_data.get("dbsnp"):
            dbsnp = variant_data["dbsnp"]
            if dbsnp.get("rsid"):
                text_parts.append(f"\ndbSNP: {dbsnp['rsid']}")

        # CADD scores
        if variant_data.get("cadd"):
            cadd = variant_data["cadd"]
            if cadd.get("phred"):
                text_parts.append(f"\nCADD Score: {cadd['phred']}")

        # Allele frequencies
        if variant_data.get("gnomad_exome"):
            gnomad = variant_data["gnomad_exome"]
            if gnomad.get("af", {}).get("af"):
                text_parts.append(
                    f"\nGnomAD Allele Frequency: {gnomad['af']['af']:.6f}"
                )

        # External links
        if variant_data.get("external_links"):
            links = variant_data["external_links"]
            text_parts.append(
                f"\n\nExternal Resources: {len(links)} database links available"
            )

        # Check for external data indicators
        if variant_data.get("tcga"):
            text_parts.append("\n\nTCGA Data: Available")
        if variant_data.get("1000genomes"):
            text_parts.append("\n1000 Genomes Data: Available")

        # Determine best URL
        url = variant_data.get("url", "")
        if not url and variant_data.get("dbsnp", {}).get("rsid"):
            url = f"https://www.ncbi.nlm.nih.gov/snp/{variant_data['dbsnp']['rsid']}"
        elif not url:
            url = f"https://myvariant.info/v1/variant/{id}"

        # Return OpenAI MCP compliant format
        return {
            "id": variant_data.get("_id", id),
            "title": f"Variant {variant_data.get('_id', id)}",
            "text": "\n".join(text_parts),
            "url": url,
            "metadata": variant_data,
        }

    # Invalid domain
    raise InvalidDomainError(domain, VALID_DOMAINS)


# Internal function for unified search
async def _unified_search(  # noqa: C901
    query: str,
    max_results_per_domain: int = MAX_RESULTS_PER_DOMAIN_DEFAULT,
    domains: list[str] | None = None,
    explain_query: bool = False,
) -> dict:
    """Internal unified search implementation.

    Parses the unified query language and routes to appropriate domain tools.
    Supports field-based syntax like 'gene:BRAF AND trials.phase:3'.

    Args:
        query: Unified query string with field syntax
        max_results_per_domain: Limit results per domain
        domains: Optional list to filter which domains to search
        explain_query: If True, return query parsing explanation

    Returns:
        Dictionary with results organized by domain

    Raises:
        QueryParsingError: If query cannot be parsed
        SearchExecutionError: If search execution fails
    """
    logger.info(f"Unified search with query: {query}")
    # Parse the query
    try:
        parser = QueryParser()
        parsed = parser.parse(query)
    except Exception as e:
        logger.error(f"Failed to parse query: {e}")
        raise QueryParsingError(query, e) from e

    # Route to appropriate tools
    router = QueryRouter()
    plan = router.route(parsed)

    # Filter domains if specified
    if domains:
        filtered_tools = []
        for tool in plan.tools_to_call:
            if (
                ("article" in tool and "articles" in domains)
                or ("trial" in tool and "trials" in domains)
                or ("variant" in tool and "variants" in domains)
            ):
                filtered_tools.append(tool)
        plan.tools_to_call = filtered_tools

    # Return explanation if requested
    if explain_query:
        return {
            "original_query": query,
            "parsed_structure": {
                "cross_domain_fields": parsed.cross_domain_fields,
                "domain_specific_fields": parsed.domain_specific_fields,
                "terms": [
                    {
                        "field": term.field,
                        "operator": term.operator.value,
                        "value": term.value,
                        "domain": term.domain,
                    }
                    for term in parsed.terms
                ],
            },
            "routing_plan": {
                "tools_to_call": plan.tools_to_call,
                "field_mappings": plan.field_mappings,
            },
            "schema": parser.get_schema(),
        }

    # Execute the search plan
    try:
        results = await execute_routing_plan(plan, output_json=True)
    except Exception as e:
        logger.error(f"Failed to execute search plan: {e}")
        raise SearchExecutionError("unified", e) from e

    # Format unified results - collect all results into a single array
    all_results = []

    for domain, result_str in results.items():
        if isinstance(result_str, dict) and "error" in result_str:
            logger.warning(f"Error in domain {domain}: {result_str['error']}")
            continue

        try:
            data = (
                json.loads(result_str)
                if isinstance(result_str, str)
                else result_str
            )

            # Get the appropriate handler for formatting
            handler_class = get_domain_handler(
                domain.rstrip("s")
            )  # Remove trailing 's'

            # Process and format each result
            # Handle both list format and dict format (for articles with cBioPortal data)
            items_to_process = []
            cbioportal_summary = None

            if isinstance(data, list):
                items_to_process = data[:max_results_per_domain]
            elif isinstance(data, dict):
                # Handle unified search format with cBioPortal data
                if "articles" in data:
                    items_to_process = data["articles"][
                        :max_results_per_domain
                    ]
                    cbioportal_summary = data.get("cbioportal_summary")
                else:
                    # Single item dict
                    items_to_process = [data]

            # Add cBioPortal summary as first result if available
            if cbioportal_summary and domain == "articles":
                try:
                    # Extract gene name from parsed query or summary
                    gene_name = parsed.cross_domain_fields.get("gene", "")
                    if not gene_name and "Summary for " in cbioportal_summary:
                        # Try to extract from summary title
                        import re

                        match = re.search(
                            r"Summary for (\w+)", cbioportal_summary
                        )
                        if match:
                            gene_name = match.group(1)

                    cbio_result = {
                        "id": f"cbioportal_summary_{gene_name or 'gene'}",
                        "title": f"cBioPortal Summary for {gene_name or 'Gene'}",
                        "text": cbioportal_summary[:5000],  # Limit text length
                        "url": f"https://www.cbioportal.org/results?gene_list={gene_name}"
                        if gene_name
                        else "",
                    }
                    all_results.append(cbio_result)
                except Exception as e:
                    logger.warning(f"Failed to format cBioPortal summary: {e}")

            for item in items_to_process:
                try:
                    formatted_result = handler_class.format_result(item)
                    # Ensure OpenAI MCP format
                    openai_result = {
                        "id": formatted_result.get("id", ""),
                        "title": formatted_result.get("title", DEFAULT_TITLE),
                        "text": formatted_result.get(
                            "snippet", formatted_result.get("text", "")
                        ),
                        "url": formatted_result.get("url", ""),
                    }
                    # Note: For unified search, we can optionally include domain in metadata
                    # This helps distinguish between result types
                    all_results.append(openai_result)
                except Exception as e:
                    logger.warning(
                        f"Failed to format result in domain {domain}: {e}"
                    )
                    continue

        except (json.JSONDecodeError, TypeError, ValueError) as e:
            logger.warning(f"Failed to parse results for domain {domain}: {e}")
            continue

    logger.info(
        f"Unified search completed with {len(all_results)} total results"
    )

    # Return OpenAI MCP compliant format
    return {"results": all_results}

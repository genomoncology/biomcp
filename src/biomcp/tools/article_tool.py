"""Consolidated article tool for accessing PubMed and biomedical literature."""

import logging
from typing import Annotated, Literal

from pydantic import Field

from biomcp.articles.fetch import _article_details
from biomcp.articles.search import _article_searcher
from biomcp.cbioportal_helper import get_cbioportal_summary_for_genes
from biomcp.core import ensure_list, mcp_app
from biomcp.metrics import track_performance

logger = logging.getLogger(__name__)


@mcp_app.tool()
@track_performance("biomcp.article")
async def article(
    action: Annotated[
        Literal["search", "get"],
        Field(
            description="Action to perform: 'search' to find articles, 'get' to retrieve article details"
        ),
    ],
    # Search parameters
    chemicals: Annotated[
        list[str] | str | None,
        Field(description="Chemical/drug names to search for (search only)"),
    ] = None,
    diseases: Annotated[
        list[str] | str | None,
        Field(description="Disease names to search for (search only)"),
    ] = None,
    genes: Annotated[
        list[str] | str | None,
        Field(description="Gene symbols to search for (search only)"),
    ] = None,
    keywords: Annotated[
        list[str] | str | None,
        Field(description="Free-text keywords to search for (search only)"),
    ] = None,
    variants: Annotated[
        list[str] | str | None,
        Field(
            description="Variant strings to search for (e.g., 'V600E', 'p.D277Y') (search only)"
        ),
    ] = None,
    include_preprints: Annotated[
        bool,
        Field(description="Include preprints from bioRxiv/medRxiv (search only)"),
    ] = True,
    include_cbioportal: Annotated[
        bool,
        Field(
            description="Include cBioPortal cancer genomics summary when searching by gene (search only)"
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
    id: Annotated[  # noqa: A002
        str | None,
        Field(
            description="Article identifier for 'get' action - PubMed ID (e.g., '38768446' or 'PMC11193658') or DOI (e.g., '10.1101/2024.01.20.23288905')"
        ),
    ] = None,
) -> str:
    """Access PubMed and biomedical literature databases.

    ⚠️ PREREQUISITE: Use the 'think' tool FIRST to plan your research strategy!

    This tool provides access to biomedical literature from PubMed, PubMed Central,
    and preprint servers.

    ## Actions:

    ### search - Find scientific articles
    Search for scientific literature ABOUT genes, variants, diseases, or chemicals:
    - PubMed/PubTator3 published articles
    - Preprints from bioRxiv/medRxiv (optional)
    - Optional cBioPortal cancer genomics summaries

    Important: This searches for ARTICLES ABOUT these topics, not database records.
    For genetic variant database records, use the 'variant' tool instead.

    Use cases:
    - Find articles about BRAF mutations in melanoma
    - Search for papers on a specific drug's effects
    - Locate research on gene-disease associations
    - Discover recent preprints in your field

    ### get - Retrieve article details
    Fetch the full abstract and available text for a specific article:
    - Title and abstract
    - Full text (when available from PMC)
    - Source information (PubMed or Europe PMC)

    Supports:
    - PubMed IDs (PMID) for published articles
    - PMC IDs for articles in PubMed Central
    - DOIs for preprints from Europe PMC

    ## Examples:

    Search for BRAF articles in melanoma:
    ```python
    await article(
        action="search",
        genes="BRAF",
        diseases="melanoma",
        include_preprints=True
    )
    ```

    Search for immunotherapy cancer papers:
    ```python
    await article(
        action="search",
        keywords=["immunotherapy", "cancer"],
        page_size=20
    )
    ```

    Search by variant:
    ```python
    await article(
        action="search",
        genes="EGFR",
        variants="T790M",
        diseases="lung cancer"
    )
    ```

    Get article details by PMID:
    ```python
    await article(action="get", id="38768446")
    ```

    Get article by DOI:
    ```python
    await article(action="get", id="10.1101/2024.01.20.23288905")
    ```

    Get article by PMC ID:
    ```python
    await article(action="get", id="PMC11193658")
    ```

    ## Notes:
    - For variant DATABASE records, use the 'variant' tool
    - For clinical trial information, use the 'trial' tool
    - For drug safety/labels, use the 'fda' tool
    """
    logger.info(f"Article tool called: action={action}")

    if action == "search":
        # Convert single values to lists
        chemicals_list = ensure_list(chemicals) if chemicals else None
        diseases_list = ensure_list(diseases) if diseases else None
        genes_list = ensure_list(genes) if genes else None
        keywords_list = ensure_list(keywords) if keywords else None
        variants_list = ensure_list(variants) if variants else None

        result = await _article_searcher(
            call_benefit="Direct article search for specific biomedical topics",
            chemicals=chemicals_list,
            diseases=diseases_list,
            genes=genes_list,
            keywords=keywords_list,
            variants=variants_list,
            include_preprints=include_preprints,
            include_cbioportal=include_cbioportal,
        )

        # Add cBioPortal summary if searching by gene
        if include_cbioportal and genes_list:
            request_params = {
                "keywords": keywords_list,
                "diseases": diseases_list,
                "chemicals": chemicals_list,
                "variants": variants_list,
            }
            cbioportal_summary = await get_cbioportal_summary_for_genes(
                genes_list, request_params
            )
            if cbioportal_summary:
                result = cbioportal_summary + "\n\n---\n\n" + result

        return result

    elif action == "get":
        if not id:
            return "Error: 'id' parameter required for 'get' action (PMID, PMC ID, or DOI)"
        return await _article_details(
            call_benefit="Fetch detailed article information for analysis",
            pmid=id,
        )

    return f"Error: Invalid action '{action}'"

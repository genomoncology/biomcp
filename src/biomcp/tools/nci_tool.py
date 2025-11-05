"""Consolidated NCI tool for accessing NCI Clinical Trials Search API databases."""

import logging
from typing import Annotated, Literal

from pydantic import Field

from biomcp.core import mcp_app
from biomcp.metrics import track_performance

logger = logging.getLogger(__name__)


@mcp_app.tool()
@track_performance("biomcp.nci")
async def nci(
    resource: Annotated[
        Literal["organization", "intervention", "biomarker", "disease"],
        Field(
            description="NCI resource type: 'organization' for cancer centers/sponsors, 'intervention' for drugs/devices/procedures, 'biomarker' for trial eligibility biomarkers, 'disease' for NCI cancer vocabulary"
        ),
    ],
    action: Annotated[
        Literal["search", "get"],
        Field(
            description="Action to perform: 'search' to find records, 'get' to retrieve specific record details"
        ),
    ],
    # Common search parameters
    name: Annotated[
        str | None,
        Field(description="Name to search for (partial match supported for all resources)"),
    ] = None,
    page: Annotated[
        int,
        Field(description="Page number (1-based) for search", ge=1),
    ] = 1,
    page_size: Annotated[
        int,
        Field(description="Results per page for search", ge=1, le=100),
    ] = 20,
    # Organization-specific search parameters
    organization_type: Annotated[
        str | None,
        Field(description="Organization type (e.g., 'Academic', 'Industry', 'Government') - organization resource only"),
    ] = None,
    city: Annotated[
        str | None,
        Field(description="City where organization is located. IMPORTANT: Always use with state - organization resource only"),
    ] = None,
    state: Annotated[
        str | None,
        Field(description="State/province code (e.g., 'CA', 'NY'). IMPORTANT: Always use with city - organization resource only"),
    ] = None,
    # Intervention-specific search parameters
    intervention_type: Annotated[
        str | None,
        Field(description="Type: 'Drug', 'Device', 'Biological', 'Procedure', 'Radiation', 'Behavioral', 'Genetic', 'Dietary', 'Other' - intervention resource only"),
    ] = None,
    synonyms: Annotated[
        bool,
        Field(description="Include synonym matches in search - intervention and disease resources"),
    ] = True,
    # Biomarker-specific search parameters
    biomarker_type: Annotated[
        str | None,
        Field(description="Biomarker type ('reference_gene' or 'branch') - biomarker resource only"),
    ] = None,
    # Disease-specific search parameters
    include_synonyms: Annotated[
        bool,
        Field(description="Include synonym matches in search - disease resource only"),
    ] = True,
    category: Annotated[
        str | None,
        Field(description="Disease category/type filter - disease resource only"),
    ] = None,
    # Get-specific parameter
    id: Annotated[  # noqa: A002
        str | None,
        Field(description="Record ID for 'get' action: organization_id, intervention_id, or disease_id"),
    ] = None,
    # Required API key
    api_key: Annotated[
        str | None,
        Field(
            description="NCI API key. Check if user mentioned 'my NCI API key is...' in their message. If not provided here and no env var is set, user will be prompted to provide one."
        ),
    ] = None,
) -> str:
    """Access NCI Clinical Trials Search API databases for cancer research.

    ⚠️ PREREQUISITE: Use the 'think' tool FIRST to plan your research strategy!

    This unified tool provides access to four NCI databases used in cancer clinical trials.
    All resources require an NCI API key from: https://clinicaltrialsapi.cancer.gov/

    ## Resources:

    ### organization
    - Search: Find cancer centers, hospitals, research networks, industry sponsors
    - Get: Retrieve complete organization details by organization_id
    - Includes: name, type, address, contact info, trial portfolio
    - IMPORTANT: Use city AND state together for location searches to avoid API errors

    ### intervention
    - Search: Find drugs, devices, procedures, therapies used in trials
    - Get: Retrieve complete intervention details by intervention_id
    - Includes: FDA-approved drugs, investigational agents, devices, procedures
    - Types: Drug, Device, Biological, Procedure, Radiation, Behavioral, Genetic, Dietary

    ### biomarker
    - Search only (no get): Find biomarkers used in trial eligibility criteria
    - Includes: gene mutations, protein expression, gene fusions, molecular markers
    - Examples: BRAF V600E, PD-L1 ≥ 50%, ALK fusion, MSI-H, TMB-high
    - Note: Data availability may be limited

    ### disease
    - Search only (no get): Find cancer conditions in NCI's controlled vocabulary
    - Includes: official cancer terminology, synonyms, hierarchical classifications
    - Different from general disease_getter (MyDisease.info) - this is NCI-specific

    ## Actions:
    - search: Find records matching criteria
    - get: Retrieve full details for a specific record (organization and intervention only)

    ## Examples:

    Search for cancer centers in Cleveland, OH:
    ```python
    await nci(resource="organization", action="search", city="Cleveland", state="OH", api_key="YOUR_KEY")
    ```

    Get organization details:
    ```python
    await nci(resource="organization", action="get", id="NCI-2011-03337", api_key="YOUR_KEY")
    ```

    Search for pembrolizumab:
    ```python
    await nci(resource="intervention", action="search", name="pembrolizumab", api_key="YOUR_KEY")
    ```

    Find all CAR-T therapies:
    ```python
    await nci(resource="intervention", action="search", name="CAR-T", intervention_type="Biological", api_key="YOUR_KEY")
    ```

    Search for PD-L1 biomarkers:
    ```python
    await nci(resource="biomarker", action="search", name="PD-L1", api_key="YOUR_KEY")
    ```

    Search for melanoma in NCI vocabulary:
    ```python
    await nci(resource="disease", action="search", name="melanoma", include_synonyms=True, api_key="YOUR_KEY")
    ```

    ## Error Handling:
    If you get "too_many_buckets_exception" errors:
    - For organizations: Always use city AND state together
    - For interventions: Add specific name or type filter
    - For biomarkers: Add specific name or gene filter
    - For diseases: Add specific name or category
    """
    logger.info(f"NCI tool called: resource={resource}, action={action}")

    from biomcp.integrations.cts_api import CTSAPIError

    # Route to appropriate handler based on resource and action
    if resource == "organization":
        if action == "search":
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
                        "- `nci(resource='organization', action='search', city='Cleveland', state='OH')`\n"
                        "- `nci(resource='organization', action='search', name='Cleveland Clinic')`\n"
                        "- `nci(resource='organization', action='search', name='cancer', city='Boston', state='MA')`"
                    )
                raise

        else:  # get
            if not id:
                return "Error: 'id' (organization_id) parameter required for organization retrieval"
            from biomcp.organizations import get_organization
            from biomcp.organizations.getter import format_organization_details

            org_data = await get_organization(
                org_id=id,
                api_key=api_key,
            )
            return format_organization_details(org_data)

    elif resource == "intervention":
        if action == "search":
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
                error_msg = str(e)
                if "too_many_buckets_exception" in error_msg or "75000" in error_msg:
                    return (
                        "⚠️ **Search Too Broad**\n\n"
                        "The NCI API cannot process this search because it returns too many results.\n\n"
                        "**Try adding more specific filters:**\n"
                        "- Add an intervention name (even partial)\n"
                        "- Specify an intervention type (e.g., 'Drug', 'Device')\n"
                        "- Search for a specific drug or therapy name\n\n"
                        "**Example searches:**\n"
                        "- `nci(resource='intervention', action='search', name='pembrolizumab')`\n"
                        "- `nci(resource='intervention', action='search', name='CAR-T')`\n"
                        "- `nci(resource='intervention', action='search', intervention_type='Drug')`"
                    )
                raise

        else:  # get
            if not id:
                return "Error: 'id' (intervention_id) parameter required for intervention retrieval"
            from biomcp.interventions import get_intervention
            from biomcp.interventions.getter import format_intervention_details

            intervention_data = await get_intervention(
                intervention_id=id,
                api_key=api_key,
            )
            return format_intervention_details(intervention_data)

    elif resource == "biomarker":
        if action == "get":
            return "Error: 'get' action not supported for biomarker resource (search only)"

        # search
        from biomcp.biomarkers import search_biomarkers
        from biomcp.biomarkers.search import format_biomarker_results

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
            error_msg = str(e)
            if "too_many_buckets_exception" in error_msg or "75000" in error_msg:
                return (
                    "⚠️ **Search Too Broad**\n\n"
                    "The NCI API cannot process this search because it returns too many results.\n\n"
                    "**Try adding more specific filters:**\n"
                    "- Add a biomarker name (even partial)\n"
                    "- Specify a gene symbol\n"
                    "- Add a biomarker type\n\n"
                    "**Example searches:**\n"
                    "- `nci(resource='biomarker', action='search', name='PD-L1')`\n"
                    "- `nci(resource='biomarker', action='search', name='EGFR', biomarker_type='mutation')`"
                )
            raise

    elif resource == "disease":
        if action == "get":
            return "Error: 'get' action not supported for disease resource (search only)"

        # search
        from biomcp.diseases import search_diseases
        from biomcp.diseases.search import format_disease_results

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
            error_msg = str(e)
            if "too_many_buckets_exception" in error_msg or "75000" in error_msg:
                return (
                    "⚠️ **Search Too Broad**\n\n"
                    "The NCI API cannot process this search because it returns too many results.\n\n"
                    "**Try adding more specific filters:**\n"
                    "- Add a disease name (even partial)\n"
                    "- Specify a disease category\n"
                    "- Use more specific search terms\n\n"
                    "**Example searches:**\n"
                    "- `nci(resource='disease', action='search', name='melanoma')`\n"
                    "- `nci(resource='disease', action='search', name='lung', category='maintype')`"
                )
            raise

    return f"Error: Invalid resource '{resource}' or action '{action}'"

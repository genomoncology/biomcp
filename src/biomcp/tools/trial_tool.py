"""Consolidated trial tool for accessing ClinicalTrials.gov."""

import logging
from typing import Annotated, Literal

from pydantic import Field

from biomcp.core import ensure_list, mcp_app
from biomcp.metrics import track_performance
from biomcp.trials.getter import (
    _trial_locations,
    _trial_outcomes,
    _trial_protocol,
    _trial_references,
)
from biomcp.trials.search import _trial_searcher

logger = logging.getLogger(__name__)


@mcp_app.tool()
@track_performance("biomcp.trial")
async def trial(
    action: Annotated[
        Literal["search", "get", "get_protocol", "get_locations", "get_outcomes", "get_references"],
        Field(
            description="Action to perform: 'search' to find trials, 'get' for complete trial details, 'get_protocol' for core protocol info, 'get_locations' for sites/contacts, 'get_outcomes' for results, 'get_references' for publications"
        ),
    ],
    # Search parameters
    conditions: Annotated[
        list[str] | str | None,
        Field(description="Medical conditions to search for (search only)"),
    ] = None,
    interventions: Annotated[
        list[str] | str | None,
        Field(description="Treatment interventions to search for (search only)"),
    ] = None,
    other_terms: Annotated[
        list[str] | str | None,
        Field(description="Additional search terms (search only)"),
    ] = None,
    recruiting_status: Annotated[
        Literal["OPEN", "CLOSED", "ANY"] | None,
        Field(description="Filter by recruiting status (search only)"),
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
        Field(description="Filter by clinical trial phase (search only)"),
    ] = None,
    location: Annotated[
        str | None,
        Field(description="Location term for geographic filtering (search only)"),
    ] = None,
    lat: Annotated[
        float | None,
        Field(
            description="Latitude for location-based search. AI agents should geocode city names (search only)",
            ge=-90,
            le=90,
        ),
    ] = None,
    long: Annotated[
        float | None,
        Field(
            description="Longitude for location-based search. AI agents should geocode city names (search only)",
            ge=-180,
            le=180,
        ),
    ] = None,
    distance: Annotated[
        int | None,
        Field(
            description="Distance in miles from lat/long coordinates (search only)",
            ge=1,
        ),
    ] = None,
    age_group: Annotated[
        Literal["CHILD", "ADULT", "OLDER_ADULT"] | None,
        Field(description="Filter by age group (search only)"),
    ] = None,
    sex: Annotated[
        Literal["FEMALE", "MALE", "ALL"] | None,
        Field(description="Filter by biological sex (search only)"),
    ] = None,
    healthy_volunteers: Annotated[
        Literal["YES", "NO"] | None,
        Field(description="Filter by healthy volunteer eligibility (search only)"),
    ] = None,
    study_type: Annotated[
        Literal["INTERVENTIONAL", "OBSERVATIONAL", "EXPANDED_ACCESS"] | None,
        Field(description="Filter by study type (search only)"),
    ] = None,
    funder_type: Annotated[
        Literal["NIH", "OTHER_GOV", "INDUSTRY", "OTHER"] | None,
        Field(description="Filter by funding source (search only)"),
    ] = None,
    page: Annotated[
        int,
        Field(description="Page number (1-based) for search", ge=1),
    ] = 1,
    page_size: Annotated[
        int,
        Field(description="Results per page for search", ge=1, le=100),
    ] = 10,
    # Get parameters (for all get actions)
    nct_id: Annotated[
        str | None,
        Field(description="NCT ID for 'get' actions (e.g., 'NCT06524388')"),
    ] = None,
) -> str:
    """Access ClinicalTrials.gov for searching and retrieving clinical trial information.

    ⚠️ PREREQUISITE: Use the 'think' tool FIRST to plan your research strategy!

    This tool provides comprehensive access to clinical trial data from ClinicalTrials.gov.

    ## Actions:

    ### search - Find clinical trials
    Search for trials based on multiple criteria including:
    - Conditions/diseases
    - Interventions/treatments
    - Location (by term or coordinates)
    - Phase, status, eligibility
    - Study type and funding

    Location search notes:
    - Use either location term OR lat/long coordinates, not both
    - For city-based searches, AI agents should geocode to lat/long first
    - Distance parameter only works with lat/long coordinates

    ### get - Complete trial details
    Retrieves all available information for a trial including:
    - Protocol (design, eligibility, sponsor)
    - Locations (sites, contacts, investigators)
    - Outcomes (measures, results if available)
    - References (publications, related papers)

    ### get_protocol - Core protocol information
    Retrieves essential protocol details:
    - Official title and summary
    - Study status and sponsor
    - Study design (type, phase, allocation, masking)
    - Eligibility criteria
    - Completion dates

    ### get_locations - Site locations and contacts
    Retrieves all study locations:
    - Facility names and addresses
    - Principal investigator information
    - Contact details (when recruiting)
    - Recruitment status by site

    ### get_outcomes - Outcome measures and results
    Retrieves outcome information:
    - Primary outcome measures
    - Secondary outcome measures
    - Results data (if available for completed trials)
    - Adverse events (if reported)

    ### get_references - Publications and references
    Retrieves linked publications:
    - Published results papers
    - Background literature
    - Protocol publications
    - PubMed IDs for cross-referencing

    ## Examples:

    Search for melanoma trials:
    ```python
    await trial(action="search", conditions="melanoma", recruiting_status="OPEN")
    ```

    Search for phase 3 immunotherapy trials:
    ```python
    await trial(
        action="search",
        conditions="cancer",
        interventions="immunotherapy",
        phase="PHASE3"
    )
    ```

    Search trials near Boston (geocoded):
    ```python
    await trial(
        action="search",
        conditions="diabetes",
        lat=42.3601,
        long=-71.0589,
        distance=50
    )
    ```

    Get complete trial details:
    ```python
    await trial(action="get", nct_id="NCT06524388")
    ```

    Get just protocol information:
    ```python
    await trial(action="get_protocol", nct_id="NCT06524388")
    ```

    Get trial locations:
    ```python
    await trial(action="get_locations", nct_id="NCT06524388")
    ```

    Get trial outcomes:
    ```python
    await trial(action="get_outcomes", nct_id="NCT06524388")
    ```

    Get trial publications:
    ```python
    await trial(action="get_references", nct_id="NCT06524388")
    ```
    """
    logger.info(f"Trial tool called: action={action}")

    if action == "search":
        # Validate location parameters
        if location and (lat is not None or long is not None):
            return "Error: Use either location term OR lat/long coordinates, not both"

        if (lat is not None and long is None) or (lat is None and long is not None):
            return "Error: Both latitude and longitude must be provided together"

        if distance is not None and (lat is None or long is None):
            return "Error: Distance parameter requires both latitude and longitude"

        # Convert single values to lists
        conditions_list = ensure_list(conditions) if conditions else None
        interventions_list = ensure_list(interventions) if interventions else None
        other_terms_list = ensure_list(other_terms) if other_terms else None

        return await _trial_searcher(
            call_benefit="Direct clinical trial search for specific criteria",
            conditions=conditions_list,
            interventions=interventions_list,
            terms=other_terms_list,
            recruiting_status=recruiting_status,
            phase=phase,
            lat=lat,
            long=long,
            distance=distance,
            age_group=age_group,
            study_type=study_type,
            page_size=page_size,
        )

    elif action == "get":
        if not nct_id:
            return "Error: 'nct_id' parameter required for 'get' action"

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

    elif action == "get_protocol":
        if not nct_id:
            return "Error: 'nct_id' parameter required for 'get_protocol' action"
        return await _trial_protocol(
            call_benefit="Fetch trial protocol information for eligibility assessment",
            nct_id=nct_id,
        )

    elif action == "get_locations":
        if not nct_id:
            return "Error: 'nct_id' parameter required for 'get_locations' action"
        return await _trial_locations(
            call_benefit="Fetch trial locations and contacts for enrollment information",
            nct_id=nct_id,
        )

    elif action == "get_outcomes":
        if not nct_id:
            return "Error: 'nct_id' parameter required for 'get_outcomes' action"
        return await _trial_outcomes(
            call_benefit="Fetch trial outcome measures and results for efficacy assessment",
            nct_id=nct_id,
        )

    elif action == "get_references":
        if not nct_id:
            return "Error: 'nct_id' parameter required for 'get_references' action"
        return await _trial_references(
            call_benefit="Fetch trial publications and references for evidence review",
            nct_id=nct_id,
        )

    return f"Error: Invalid action '{action}'"

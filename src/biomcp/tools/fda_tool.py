"""Consolidated FDA tool for accessing OpenFDA databases."""

import logging
from typing import Annotated, Literal

from pydantic import Field

from biomcp.core import mcp_app
from biomcp.metrics import track_performance

logger = logging.getLogger(__name__)


@mcp_app.tool()
@track_performance("biomcp.fda")
async def fda(
    domain: Annotated[
        Literal["adverse", "label", "device", "approval", "recall", "shortage"],
        Field(
            description="FDA database domain: 'adverse' for adverse events (FAERS), 'label' for drug labels (SPL), 'device' for device events (MAUDE), 'approval' for drug approvals, 'recall' for enforcement/recalls, 'shortage' for drug shortages"
        ),
    ],
    action: Annotated[
        Literal["search", "get"],
        Field(
            description="Action to perform: 'search' to find records, 'get' to retrieve specific record details"
        ),
    ],
    # Search parameters (common)
    drug: Annotated[
        str | None,
        Field(description="Drug name (for adverse, label, approval, recall, shortage domains)"),
    ] = None,
    limit: Annotated[
        int,
        Field(description="Maximum number of results for search", ge=1, le=100),
    ] = 25,
    page: Annotated[
        int,
        Field(description="Page number (1-based) for search", ge=1),
    ] = 1,
    # Adverse event search parameters
    reaction: Annotated[
        str | None,
        Field(description="Adverse reaction term (adverse domain search only)"),
    ] = None,
    serious: Annotated[
        bool | None,
        Field(description="Filter for serious events only (adverse domain search only)"),
    ] = None,
    # Label search parameters
    indication: Annotated[
        str | None,
        Field(description="Search for drugs indicated for this condition (label domain search only)"),
    ] = None,
    boxed_warning: Annotated[
        bool,
        Field(description="Filter for drugs with boxed warnings (label domain search only)"),
    ] = False,
    section: Annotated[
        str | None,
        Field(description="Specific label section to search/retrieve (label domain)"),
    ] = None,
    # Device search parameters
    device: Annotated[
        str | None,
        Field(description="Device name (device domain search only)"),
    ] = None,
    manufacturer: Annotated[
        str | None,
        Field(description="Manufacturer name (device domain search only)"),
    ] = None,
    problem: Annotated[
        str | None,
        Field(description="Device problem description (device domain search only)"),
    ] = None,
    product_code: Annotated[
        str | None,
        Field(description="FDA product code (device domain search only)"),
    ] = None,
    genomics_only: Annotated[
        bool,
        Field(description="Filter to genomic/diagnostic devices only (device domain search only)"),
    ] = True,
    # Approval search parameters
    application_number: Annotated[
        str | None,
        Field(description="NDA or BLA application number (approval domain)"),
    ] = None,
    approval_year: Annotated[
        str | None,
        Field(description="Year of approval in YYYY format (approval domain search only)"),
    ] = None,
    # Recall search parameters
    recall_class: Annotated[
        str | None,
        Field(description="Recall classification: 1=most serious, 2=moderate, 3=least serious (recall domain search only)"),
    ] = None,
    status: Annotated[
        str | None,
        Field(description="Status filter: for recalls (ongoing/completed/terminated), for shortages (current/resolved)"),
    ] = None,
    reason: Annotated[
        str | None,
        Field(description="Search text in recall reason (recall domain search only)"),
    ] = None,
    since_date: Annotated[
        str | None,
        Field(description="Show recalls after this date in YYYYMMDD format (recall domain search only)"),
    ] = None,
    # Shortage search parameters
    therapeutic_category: Annotated[
        str | None,
        Field(description="Therapeutic category like 'Oncology', 'Anti-infective' (shortage domain search only)"),
    ] = None,
    # Get-specific parameters
    id: Annotated[  # noqa: A002
        str | None,
        Field(description="Record ID for 'get' action: report_id (adverse), set_id (label), mdr_report_key (device), application_number (approval), recall_number (recall), drug name (shortage)"),
    ] = None,
    sections: Annotated[
        list[str] | None,
        Field(description="Specific sections to retrieve (label domain get only)"),
    ] = None,
    # Common parameter
    api_key: Annotated[
        str | None,
        Field(description="Optional OpenFDA API key (overrides OPENFDA_API_KEY env var)"),
    ] = None,
) -> str:
    """Access FDA databases for drug safety, labeling, and regulatory information.

    ⚠️ PREREQUISITE: Use the 'think' tool FIRST to plan your research strategy!

    This unified tool provides access to six FDA databases:

    ## Domains:

    ### adverse (FAERS - Adverse Event Reporting System)
    - Search: Find adverse event reports by drug, reaction, severity
    - Get: Retrieve complete report details by report_id
    - Note: Reports don't establish causation - voluntary, may be incomplete

    ### label (SPL - Structured Product Labels)
    - Search: Find drug labels by name, indication, warnings
    - Get: Retrieve complete label by set_id
    - Includes: indications, dosage, contraindications, warnings, interactions

    ### device (MAUDE - Medical Device Adverse Events)
    - Search: Find device event reports (defaults to genomic/diagnostic devices)
    - Get: Retrieve complete device report by mdr_report_key
    - Includes: malfunctions, injuries, genomic test issues

    ### approval (Drugs@FDA)
    - Search: Find drug approval records by name, application number, year
    - Get: Retrieve complete approval details by application_number
    - Includes: approval dates, formulations, marketing status

    ### recall (Enforcement Reports)
    - Search: Find recall records by drug, classification, reason
    - Get: Retrieve complete recall details by recall_number
    - Class I=most serious, II=moderate, III=least serious

    ### shortage (Drug Shortages)
    - Search: Find drug shortages by name, status, therapeutic category
    - Get: Retrieve shortage details by drug name
    - Note: Data cached, check FDA.gov for latest

    ## Actions:
    - search: Find records matching criteria
    - get: Retrieve full details for a specific record

    ## Examples:

    Search for aspirin adverse events:
    ```python
    await fda(domain="adverse", action="search", drug="aspirin", serious=True)
    ```

    Get drug label for imatinib:
    ```python
    await fda(domain="label", action="search", drug="imatinib", limit=1)
    # Then use set_id from results
    await fda(domain="label", action="get", id="set_id_here")
    ```

    Search device events for genomic tests:
    ```python
    await fda(domain="device", action="search", device="sequencing", genomics_only=True)
    ```

    Check drug approvals from 2024:
    ```python
    await fda(domain="approval", action="search", approval_year="2024")
    ```

    Find serious drug recalls:
    ```python
    await fda(domain="recall", action="search", recall_class="1")
    ```

    Check current drug shortages:
    ```python
    await fda(domain="shortage", action="search", status="current", therapeutic_category="Oncology")
    ```
    """
    logger.info(f"FDA tool called: domain={domain}, action={action}")

    # Calculate skip for pagination
    skip = (page - 1) * limit

    # Route to appropriate handler based on domain and action
    if domain == "adverse":
        if action == "search":
            from biomcp.openfda import search_adverse_events

            return await search_adverse_events(
                drug=drug,
                reaction=reaction,
                serious=serious,
                limit=limit,
                skip=skip,
                api_key=api_key,
            )
        else:  # get
            if not id:
                return "Error: 'id' (report_id) parameter required for adverse event retrieval"
            from biomcp.openfda import get_adverse_event

            return await get_adverse_event(id, api_key=api_key)

    elif domain == "label":
        if action == "search":
            from biomcp.openfda import search_drug_labels

            return await search_drug_labels(
                name=drug,
                indication=indication,
                boxed_warning=boxed_warning,
                section=section,
                limit=limit,
                skip=skip,
                api_key=api_key,
            )
        else:  # get
            if not id:
                return "Error: 'id' (set_id) parameter required for label retrieval"
            from biomcp.openfda import get_drug_label

            # Clean up sections: filter out None/empty values and convert empty array to None
            if sections:
                sections_cleaned = [s.strip() for s in sections if s and isinstance(s, str) and s.strip()]
                sections_to_use = sections_cleaned if sections_cleaned else None
            else:
                sections_to_use = None

            return await get_drug_label(id, sections_to_use, api_key=api_key)

    elif domain == "device":
        if action == "search":
            from biomcp.openfda import search_device_events

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
        else:  # get
            if not id:
                return "Error: 'id' (mdr_report_key) parameter required for device event retrieval"
            from biomcp.openfda import get_device_event

            return await get_device_event(id, api_key=api_key)

    elif domain == "approval":
        if action == "search":
            from biomcp.openfda import search_drug_approvals

            return await search_drug_approvals(
                drug=drug,
                application_number=application_number,
                approval_year=approval_year,
                limit=limit,
                skip=skip,
                api_key=api_key,
            )
        else:  # get
            if not id:
                return "Error: 'id' (application_number) parameter required for approval retrieval"
            from biomcp.openfda import get_drug_approval

            return await get_drug_approval(id, api_key=api_key)

    elif domain == "recall":
        if action == "search":
            from biomcp.openfda import search_drug_recalls

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
        else:  # get
            if not id:
                return "Error: 'id' (recall_number) parameter required for recall retrieval"
            from biomcp.openfda import get_drug_recall

            return await get_drug_recall(id, api_key=api_key)

    elif domain == "shortage":
        if action == "search":
            from biomcp.openfda import search_drug_shortages

            return await search_drug_shortages(
                drug=drug,
                status=status,
                therapeutic_category=therapeutic_category,
                limit=limit,
                skip=skip,
                api_key=api_key,
            )
        else:  # get
            if not id:
                return "Error: 'id' (drug name) parameter required for shortage retrieval"
            from biomcp.openfda import get_drug_shortage

            return await get_drug_shortage(id, api_key=api_key)

    return f"Error: Invalid domain '{domain}' or action '{action}'"

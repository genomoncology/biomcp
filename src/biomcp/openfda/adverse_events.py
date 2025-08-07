"""
OpenFDA Drug Adverse Events (FAERS) integration.
"""

import logging

from .adverse_events_helpers import (
    format_drug_details,
    format_reaction_details,
    format_report_metadata,
    format_report_summary,
    format_search_summary,
    format_top_reactions,
)
from .constants import (
    OPENFDA_DEFAULT_LIMIT,
    OPENFDA_DISCLAIMER,
    OPENFDA_DRUG_EVENTS_URL,
    OPENFDA_MAX_LIMIT,
)
from .exceptions import (
    OpenFDAConnectionError,
    OpenFDARateLimitError,
    OpenFDATimeoutError,
)
from .input_validation import sanitize_input
from .utils import clean_text, make_openfda_request

logger = logging.getLogger(__name__)


def _build_search_query(
    drug: str | None, reaction: str | None, serious: bool | None
) -> str:
    """Build the search query for adverse events."""
    search_parts = []

    if drug:
        # Sanitize drug input to prevent injection
        drug = sanitize_input(drug, max_length=100)
        if drug:
            drug_query = (
                f'(patient.drug.medicinalproduct:"{drug}" OR '
                f'patient.drug.openfda.brand_name:"{drug}" OR '
                f'patient.drug.openfda.generic_name:"{drug}")'
            )
            search_parts.append(drug_query)

    if reaction:
        # Sanitize reaction input
        reaction = sanitize_input(reaction, max_length=200)
        if reaction:
            search_parts.append(
                f'patient.reaction.reactionmeddrapt:"{reaction}"'
            )

    if serious is not None:
        serious_value = "1" if serious else "2"
        search_parts.append(f"serious:{serious_value}")

    return " AND ".join(search_parts)


async def search_adverse_events(  # noqa: C901
    drug: str | None = None,
    reaction: str | None = None,
    serious: bool | None = None,
    limit: int = OPENFDA_DEFAULT_LIMIT,
    skip: int = 0,
    api_key: str | None = None,
) -> str:
    """
    Search FDA adverse event reports (FAERS).

    Args:
        drug: Drug name to search for
        reaction: Adverse reaction term to search for
        serious: Filter for serious events only
        limit: Maximum number of results
        skip: Number of results to skip
        api_key: Optional OpenFDA API key (overrides OPENFDA_API_KEY env var)

    Returns:
        Formatted string with adverse event information
    """
    if not drug and not reaction:
        return (
            "⚠️ Please specify either a drug name or reaction term to search "
            "adverse events.\n\n"
            "Examples:\n"
            "- Search by drug: --drug 'imatinib'\n"
            "- Search by reaction: --reaction 'nausea'\n"
            "- Both: --drug 'imatinib' --reaction 'nausea'"
        )

    # Build and execute search
    search_query = _build_search_query(drug, reaction, serious)
    params = {
        "search": search_query,
        "limit": min(limit, OPENFDA_MAX_LIMIT),
        "skip": skip,
    }

    try:
        response, error = await make_openfda_request(
            OPENFDA_DRUG_EVENTS_URL, params, "openfda_adverse_events", api_key
        )
    except OpenFDARateLimitError:
        return (
            "⚠️ **FDA API Rate Limit Exceeded**\n\n"
            "You've exceeded the FDA's rate limit. Options:\n"
            "• Wait a moment and try again\n"
            "• Provide an FDA API key for higher limits (240/min vs 40/min)\n"
            "• Get a free key at: https://open.fda.gov/apis/authentication/"
        )
    except OpenFDATimeoutError:
        return (
            "⏱️ **Request Timeout**\n\n"
            "The FDA API is taking too long to respond. This may be due to:\n"
            "• High server load\n"
            "• Complex query\n"
            "• Network issues\n\n"
            "Please try again in a moment."
        )
    except OpenFDAConnectionError as e:
        return (
            "🔌 **Connection Error**\n\n"
            f"Unable to connect to FDA API: {e}\n\n"
            "Please check your internet connection and try again."
        )

    if error:
        return f"⚠️ Error searching adverse events: {error}"

    if not response or not response.get("results"):
        search_desc = []
        if drug:
            search_desc.append(f"drug '{drug}'")
        if reaction:
            search_desc.append(f"reaction '{reaction}'")
        return (
            f"No adverse event reports found for {' and '.join(search_desc)}."
        )

    results = response["results"]
    total = (
        response.get("meta", {}).get("results", {}).get("total", len(results))
    )

    # Build output
    output = ["## FDA Adverse Event Reports\n"]
    output.extend(format_search_summary(drug, reaction, serious, total))

    # Add top reactions if searching by drug
    if drug and not reaction:
        output.extend(format_top_reactions(results))

    # Add sample reports
    output.append(
        f"### Sample Reports (showing {min(len(results), 3)} of {total}):\n"
    )
    for i, result in enumerate(results[:3], 1):
        output.extend(format_report_summary(result, i))

    output.append(f"\n{OPENFDA_DISCLAIMER}")
    return "\n".join(output)


async def get_adverse_event(report_id: str, api_key: str | None = None) -> str:
    """
    Get detailed information for a specific adverse event report.

    Args:
        report_id: Safety report ID
        api_key: Optional OpenFDA API key (overrides OPENFDA_API_KEY env var)

    Returns:
        Formatted string with detailed report information
    """
    params = {
        "search": f'safetyreportid:"{report_id}"',
        "limit": 1,
    }

    response, error = await make_openfda_request(
        OPENFDA_DRUG_EVENTS_URL,
        params,
        "openfda_adverse_event_detail",
        api_key,
    )

    if error:
        return f"⚠️ Error retrieving adverse event report: {error}"

    if not response or not response.get("results"):
        return f"Adverse event report '{report_id}' not found."

    result = response["results"][0]
    patient = result.get("patient", {})

    # Build detailed output
    output = [f"## Adverse Event Report: {report_id}\n"]

    # Patient Information
    output.extend(_format_patient_info(patient))

    # Drug Information
    if drugs := patient.get("drug", []):
        output.extend(format_drug_details(drugs))

    # Reactions
    if reactions := patient.get("reaction", []):
        output.extend(format_reaction_details(reactions))

    # Event Summary
    if summary := patient.get("summary", {}).get("narrativeincludeclinical"):
        output.append("### Event Narrative")
        output.append(clean_text(summary))
        output.append("")

    # Report metadata
    output.extend(format_report_metadata(result))

    output.append(f"\n{OPENFDA_DISCLAIMER}")
    return "\n".join(output)


def _format_patient_info(patient: dict) -> list[str]:
    """Format patient information section."""
    output = ["### Patient Information"]

    if age := patient.get("patientonsetage"):
        output.append(f"- **Age**: {age} years")

    sex_map = {0: "Unknown", 1: "Male", 2: "Female"}
    sex_code = patient.get("patientsex")
    sex = (
        sex_map.get(sex_code, "Unknown") if sex_code is not None else "Unknown"
    )
    output.append(f"- **Sex**: {sex}")

    if weight := patient.get("patientweight"):
        output.append(f"- **Weight**: {weight} kg")

    output.append("")
    return output

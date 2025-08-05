"""NCI Clinical Trials Search API integration for trial searches."""

import logging
from typing import Any

from ..constants import NCI_TRIALS_URL
from ..diseases.search import search_diseases
from ..integrations.cts_api import CTSAPIError, make_cts_request
from ..interventions.search import search_interventions
from .search import TrialQuery

logger = logging.getLogger(__name__)


async def _expand_disease_terms(
    conditions: list[str],
    expand_synonyms: bool,
) -> list[str]:
    """Expand disease terms with synonyms if requested."""
    if not expand_synonyms:
        return conditions

    disease_terms = []
    for condition in conditions:
        try:
            results = await search_diseases(
                name=condition,
                include_synonyms=True,
                page_size=5,
            )
            # Add the original term plus any exact matches
            disease_terms.append(condition)
            for disease in results.get("diseases", [])[:3]:
                if disease.get("name"):
                    disease_terms.append(disease["name"])
                # Add top synonyms
                synonyms = disease.get("synonyms", [])
                if isinstance(synonyms, list):
                    disease_terms.extend(synonyms[:2])
        except Exception as e:
            logger.warning(f"Failed to expand disease term {condition}: {e}")
            disease_terms.append(condition)

    # Remove duplicates while preserving order
    seen = set()
    unique_diseases = []
    for term in disease_terms:
        if term.lower() not in seen:
            seen.add(term.lower())
            unique_diseases.append(term)

    return unique_diseases


async def _normalize_interventions(interventions: list[str]) -> list[str]:
    """Normalize intervention names to IDs where possible."""
    intervention_ids = []
    for intervention in interventions:
        try:
            results = await search_interventions(
                name=intervention,
                page_size=1,
            )
            interventions_data = results.get("interventions", [])
            if interventions_data:
                # Use the ID if available, otherwise the name
                int_id = interventions_data[0].get("id", intervention)
                intervention_ids.append(int_id)
            else:
                intervention_ids.append(intervention)
        except Exception:
            intervention_ids.append(intervention)

    return intervention_ids


def _map_phase_to_nci(phase: Any) -> str | None:
    """Map TrialPhase enum to NCI phase values."""
    if not phase:
        return None

    phase_map = {
        "EARLY_PHASE1": "I",
        "PHASE1": "I",
        "PHASE2": "II",
        "PHASE3": "III",
        "PHASE4": "IV",
        "NOT_APPLICABLE": "NA",
    }
    return phase_map.get(phase.value, phase.value)


def _map_status_to_nci(recruiting_status: Any) -> list[str] | None:
    """Map RecruitingStatus enum to NCI status values."""
    if not recruiting_status:
        return None

    status_map = {
        "OPEN": ["recruiting", "enrolling_by_invitation"],
        "CLOSED": ["active_not_recruiting", "completed", "terminated"],
        "ANY": None,
    }
    return status_map.get(recruiting_status.value)


def _map_sort_to_nci(sort: Any) -> str | None:
    """Map SortOrder enum to NCI sort values."""
    if not sort:
        return None

    sort_map = {
        "RELEVANCE": "relevance",
        "LAST_UPDATE": "last_update_date",
        "START_DATE": "start_date",
        "COMPLETION_DATE": "completion_date",
    }
    return sort_map.get(sort.value)


def _add_location_params(params: dict[str, Any], query: TrialQuery) -> None:
    """Add location parameters if present."""
    if query.lat is not None and query.long is not None:
        params["latitude"] = query.lat
        params["longitude"] = query.long
        params["distance"] = query.distance or 50


def _add_eligibility_params(params: dict[str, Any], query: TrialQuery) -> None:
    """Add advanced eligibility criteria parameters."""
    if query.prior_therapies:
        params["prior_therapy"] = query.prior_therapies

    if query.required_mutations:
        params["biomarkers"] = query.required_mutations

    if query.allow_brain_mets is not None:
        params["accepts_brain_mets"] = query.allow_brain_mets


async def convert_query_to_nci(query: TrialQuery) -> dict[str, Any]:
    """
    Convert a TrialQuery object to NCI CTS API parameters.

    Maps BioMCP's TrialQuery fields to NCI's parameter structure.
    """
    params: dict[str, Any] = {}

    # Basic search terms
    if query.terms:
        params["_fulltext"] = " ".join(query.terms)

    # Conditions/diseases with synonym expansion
    if query.conditions:
        disease_terms = await _expand_disease_terms(
            query.conditions,
            query.expand_synonyms,
        )
        if disease_terms:
            params["diseases"] = disease_terms

    # Interventions
    if query.interventions:
        params["interventions"] = await _normalize_interventions(
            query.interventions
        )

    # NCT IDs
    if query.nct_ids:
        params["nct_ids"] = query.nct_ids

    # Phase and status mappings
    nci_phase = _map_phase_to_nci(query.phase)
    if nci_phase:
        params["phase"] = nci_phase

    statuses = _map_status_to_nci(query.recruiting_status)
    if statuses:
        params["recruitment_status"] = statuses

    # Location and eligibility
    _add_location_params(params, query)
    _add_eligibility_params(params, query)

    # Pagination
    params["size"] = query.page_size if query.page_size else 20

    # Sort order
    sort_value = _map_sort_to_nci(query.sort)
    if sort_value:
        params["sort"] = sort_value

    return params


async def search_trials_nci(
    query: TrialQuery,
    api_key: str | None = None,
) -> dict[str, Any]:
    """
    Search for clinical trials using NCI CTS API.

    Returns:
        Dictionary with:
        - trials: List of trial records
        - total: Total number of results
        - next_page: Token for next page (if available)
        - source: "nci" to indicate data source
    """
    try:
        # Convert query to NCI parameters
        params = await convert_query_to_nci(query)

        # Make API request
        response = await make_cts_request(
            url=NCI_TRIALS_URL,
            params=params,
            api_key=api_key,
        )

        # Process response
        trials = response.get("data", response.get("trials", []))
        total = response.get("total", len(trials))
        next_page = response.get("next_page_token")

        return {
            "trials": trials,
            "total": total,
            "next_page": next_page,
            "source": "nci",
        }

    except CTSAPIError:
        raise
    except Exception as e:
        logger.error(f"NCI trial search failed: {e}")
        raise CTSAPIError(f"Trial search failed: {e!s}") from e


def _format_trial_header(trial: dict[str, Any]) -> list[str]:
    """Format trial header with basic info."""
    nct_id = trial.get("nct_id", trial.get("protocol_id", "Unknown"))
    title = trial.get("title", trial.get("brief_title", "Untitled"))
    phase = trial.get("phase", "Not specified")
    status = trial.get("overall_status", trial.get("status", "Unknown"))

    return [
        f"### [{nct_id}] {title}",
        f"- **Phase**: {phase}",
        f"- **Status**: {status}",
    ]


def _format_trial_summary_text(trial: dict[str, Any]) -> list[str]:
    """Format trial summary text if available."""
    summary = trial.get("brief_summary", trial.get("description", ""))
    if not summary:
        return []

    if len(summary) > 200:
        summary = summary[:197] + "..."
    return [f"- **Summary**: {summary}"]


def _format_trial_conditions(trial: dict[str, Any]) -> list[str]:
    """Format trial conditions/diseases."""
    conditions = trial.get("diseases", trial.get("conditions", []))
    if not conditions:
        return []

    lines = []
    if isinstance(conditions, list):
        lines.append(f"- **Conditions**: {', '.join(conditions[:3])}")
        if len(conditions) > 3:
            lines.append(f"  *(and {len(conditions) - 3} more)*")
    else:
        lines.append(f"- **Conditions**: {conditions}")

    return lines


def _format_trial_interventions(trial: dict[str, Any]) -> list[str]:
    """Format trial interventions."""
    interventions = trial.get("interventions", [])
    if not interventions:
        return []

    int_names = []
    for intervention in interventions[:3]:
        if isinstance(intervention, dict):
            int_names.append(intervention.get("name", "Unknown"))
        else:
            int_names.append(str(intervention))

    if not int_names:
        return []

    lines = [f"- **Interventions**: {', '.join(int_names)}"]
    if len(interventions) > 3:
        lines.append(f"  *(and {len(interventions) - 3} more)*")

    return lines


def _format_trial_metadata(trial: dict[str, Any]) -> list[str]:
    """Format trial metadata (sponsor, eligibility notes)."""
    lines = []

    lead_org = trial.get("lead_org", trial.get("sponsor", ""))
    if lead_org:
        lines.append(f"- **Lead Organization**: {lead_org}")

    if trial.get("accepts_brain_mets"):
        lines.append("- **Note**: Accepts patients with brain metastases")

    return lines


def _format_trial_summary(trial: dict[str, Any]) -> list[str]:
    """Format a single trial summary."""
    lines = []

    # Add header info
    lines.extend(_format_trial_header(trial))

    # Add summary text
    lines.extend(_format_trial_summary_text(trial))

    # Add conditions
    lines.extend(_format_trial_conditions(trial))

    # Add interventions
    lines.extend(_format_trial_interventions(trial))

    # Add metadata
    lines.extend(_format_trial_metadata(trial))

    lines.append("")
    return lines


def format_nci_trial_results(results: dict[str, Any]) -> str:
    """
    Format NCI trial search results as markdown.
    """
    trials = results.get("trials", [])
    total = results.get("total", 0)

    if not trials:
        return "No trials found matching the search criteria in NCI database."

    lines = [
        f"## NCI Clinical Trials Search Results ({total} found)",
        "",
        "*Source: NCI Clinical Trials Search API*",
        "",
    ]

    for trial in trials:
        lines.extend(_format_trial_summary(trial))

    return "\n".join(lines)

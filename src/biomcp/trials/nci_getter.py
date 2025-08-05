"""NCI Clinical Trials Search API integration for getting trial details."""

import logging
from typing import Any

from ..constants import NCI_TRIALS_URL
from ..integrations.cts_api import CTSAPIError, make_cts_request
from ..organizations.getter import get_organization

logger = logging.getLogger(__name__)


async def get_trial_nci(
    nct_id: str,
    api_key: str | None = None,
) -> dict[str, Any]:
    """
    Get detailed trial information from NCI CTS API.

    Args:
        nct_id: NCT identifier (e.g., "NCT04280705")
        api_key: Optional API key

    Returns:
        Dictionary with trial details
    """
    try:
        # Make API request
        url = f"{NCI_TRIALS_URL}/{nct_id}"
        response = await make_cts_request(
            url=url,
            api_key=api_key,
        )

        # Return the trial data
        if "data" in response:
            return response["data"]
        elif "trial" in response:
            return response["trial"]
        else:
            return response

    except CTSAPIError:
        raise
    except Exception as e:
        logger.error(f"Failed to get NCI trial {nct_id}: {e}")
        raise CTSAPIError(f"Failed to retrieve trial: {e!s}") from e


def _format_trial_header(trial: dict[str, Any]) -> list[str]:
    """Format trial header section."""
    nct_id = trial.get("nct_id", trial.get("protocol_id", "Unknown"))
    title = trial.get("official_title", trial.get("title", "Untitled"))
    brief_title = trial.get("brief_title", "")

    lines = [
        f"# Clinical Trial: {nct_id}",
        "",
        f"## {title}",
        "",
    ]

    if brief_title and brief_title != title:
        lines.append(f"**Brief Title**: {brief_title}")
        lines.append("")

    return lines


def _format_protocol_section(trial: dict[str, Any]) -> list[str]:
    """Format protocol information section."""
    lines = [
        "## Protocol Information",
        "",
        f"- **NCT ID**: {trial.get('nct_id', trial.get('protocol_id', 'Unknown'))}",
        f"- **Phase**: {trial.get('phase', 'Not specified')}",
        f"- **Status**: {trial.get('overall_status', 'Unknown')}",
        f"- **Study Type**: {trial.get('study_type', 'Not specified')}",
    ]

    if trial.get("primary_purpose"):
        lines.append(f"- **Primary Purpose**: {trial['primary_purpose']}")

    if trial.get("study_design"):
        design = trial["study_design"]
        if isinstance(design, dict):
            if design.get("allocation"):
                lines.append(f"- **Allocation**: {design['allocation']}")
            if design.get("masking"):
                lines.append(f"- **Masking**: {design['masking']}")
            if design.get("intervention_model"):
                lines.append(
                    f"- **Intervention Model**: {design['intervention_model']}"
                )
        else:
            lines.append(f"- **Study Design**: {design}")

    if trial.get("start_date"):
        lines.append(f"- **Start Date**: {trial['start_date']}")
    if trial.get("completion_date"):
        lines.append(f"- **Completion Date**: {trial['completion_date']}")

    lines.append("")
    return lines


def _format_summary_section(trial: dict[str, Any]) -> list[str]:
    """Format summary section."""
    lines = []
    if trial.get("brief_summary") or trial.get("description"):
        lines.extend([
            "## Summary",
            "",
            trial.get("brief_summary", trial.get("description", "")),
            "",
        ])
    return lines


def _format_conditions_section(trial: dict[str, Any]) -> list[str]:
    """Format conditions/diseases section."""
    conditions = trial.get("diseases", trial.get("conditions", []))
    if not conditions:
        return []

    lines = ["## Conditions", ""]
    if isinstance(conditions, list):
        for condition in conditions:
            lines.append(f"- {condition}")
    else:
        lines.append(f"- {conditions}")
    lines.append("")
    return lines


def _format_interventions_section(trial: dict[str, Any]) -> list[str]:
    """Format interventions section."""
    interventions = trial.get("interventions", [])
    if not interventions:
        return []

    lines = ["## Interventions", ""]
    for intervention in interventions:
        if isinstance(intervention, dict):
            name = intervention.get("name", "Unknown")
            int_type = intervention.get("type", "")
            desc = intervention.get("description", "")

            if int_type:
                lines.append(f"### {name} ({int_type})")
            else:
                lines.append(f"### {name}")

            if desc:
                lines.append(desc)
            lines.append("")
        else:
            lines.append(f"- {intervention}")
    return lines


def _format_eligibility_section(trial: dict[str, Any]) -> list[str]:
    """Format eligibility criteria section."""
    eligibility = trial.get("eligibility", {})
    if not eligibility:
        return []

    lines = ["## Eligibility Criteria", ""]

    # Basic eligibility info
    min_age = eligibility.get("minimum_age")
    max_age = eligibility.get("maximum_age")
    if min_age or max_age:
        age_str = []
        if min_age:
            age_str.append(f"Minimum: {min_age}")
        if max_age:
            age_str.append(f"Maximum: {max_age}")
        lines.append(f"**Age**: {' | '.join(age_str)}")

    if eligibility.get("gender"):
        lines.append(f"**Gender**: {eligibility['gender']}")

    if "accepts_healthy_volunteers" in eligibility:
        accepts = "Yes" if eligibility["accepts_healthy_volunteers"] else "No"
        lines.append(f"**Accepts Healthy Volunteers**: {accepts}")

    lines.append("")

    # Detailed criteria
    if eligibility.get("inclusion_criteria"):
        lines.extend([
            "### Inclusion Criteria",
            "",
            eligibility["inclusion_criteria"],
            "",
        ])

    if eligibility.get("exclusion_criteria"):
        lines.extend([
            "### Exclusion Criteria",
            "",
            eligibility["exclusion_criteria"],
            "",
        ])

    return lines


def _format_biomarker_section(trial: dict[str, Any]) -> list[str]:
    """Format biomarker requirements section."""
    biomarkers = trial.get("biomarkers", [])
    if not biomarkers:
        return []

    lines = ["## Biomarker Requirements", ""]
    for biomarker in biomarkers:
        if isinstance(biomarker, dict):
            name = biomarker.get("name", "Unknown")
            requirement = biomarker.get("requirement", "")
            lines.append(f"- **{name}**: {requirement}")
        else:
            lines.append(f"- {biomarker}")
    lines.append("")

    # Special eligibility notes
    if trial.get("accepts_brain_mets"):
        lines.extend([
            "## Special Eligibility Notes",
            "",
            "- Accepts patients with brain metastases",
            "",
        ])

    return lines


async def _format_organizations_section(
    trial: dict[str, Any],
    api_key: str | None = None,
) -> list[str]:
    """Format organizations section."""
    lead_org_id = trial.get("lead_org_id")
    lead_org_name = trial.get("lead_org", trial.get("sponsor"))

    if not (lead_org_id or lead_org_name):
        return []

    lines = ["## Organizations", "", "### Lead Organization"]

    # Try to get detailed org info if we have an ID
    if lead_org_id and api_key:
        try:
            org_details = await get_organization(lead_org_id, api_key)
            lines.append(
                f"- **Name**: {org_details.get('name', lead_org_name)}"
            )
            if org_details.get("type"):
                lines.append(f"- **Type**: {org_details['type']}")
            if org_details.get("city") and org_details.get("state"):
                lines.append(
                    f"- **Location**: {org_details['city']}, {org_details['state']}"
                )
        except Exception:
            lines.append(f"- **Name**: {lead_org_name}")
    else:
        lines.append(f"- **Name**: {lead_org_name}")

    lines.append("")

    # Collaborators
    collaborators = trial.get("collaborators", [])
    if collaborators:
        lines.append("### Collaborating Organizations")
        for collab in collaborators:
            if isinstance(collab, dict):
                lines.append(f"- {collab.get('name', 'Unknown')}")
            else:
                lines.append(f"- {collab}")
        lines.append("")

    return lines


def _format_locations_section(trial: dict[str, Any]) -> list[str]:
    """Format locations section."""
    locations = trial.get("sites", trial.get("locations", []))
    if not locations:
        return []

    lines = ["## Locations", ""]

    # Group by status
    recruiting_sites = []
    other_sites = []

    for location in locations:
        if isinstance(location, dict):
            status = location.get("recruitment_status", "").lower()
            if "recruiting" in status:
                recruiting_sites.append(location)
            else:
                other_sites.append(location)
        else:
            other_sites.append(location)

    if recruiting_sites:
        lines.append(
            f"### Currently Recruiting ({len(recruiting_sites)} sites)"
        )
        lines.append("")
        for site in recruiting_sites[:10]:
            _format_site(site, lines)
        if len(recruiting_sites) > 10:
            lines.append(
                f"*... and {len(recruiting_sites) - 10} more recruiting sites*"
            )
            lines.append("")

    if other_sites and len(other_sites) <= 5:
        lines.append(f"### Other Sites ({len(other_sites)} sites)")
        lines.append("")
        for site in other_sites:
            _format_site(site, lines)

    return lines


def _format_contact_section(trial: dict[str, Any]) -> list[str]:
    """Format contact information section."""
    contact = trial.get("overall_contact")
    if not contact:
        return []

    lines = ["## Contact Information", ""]
    if isinstance(contact, dict):
        if contact.get("name"):
            lines.append(f"**Name**: {contact['name']}")
        if contact.get("phone"):
            lines.append(f"**Phone**: {contact['phone']}")
        if contact.get("email"):
            lines.append(f"**Email**: {contact['email']}")
    else:
        lines.append(str(contact))
    lines.append("")
    return lines


async def format_nci_trial_details(
    trial: dict[str, Any],
    api_key: str | None = None,
) -> str:
    """
    Format NCI trial details as comprehensive markdown.

    Args:
        trial: Trial data from NCI API
        api_key: Optional API key for organization lookups

    Returns:
        Formatted markdown string
    """
    lines = []

    # Build document sections
    lines.extend(_format_trial_header(trial))
    lines.extend(_format_protocol_section(trial))
    lines.extend(_format_summary_section(trial))
    lines.extend(_format_conditions_section(trial))
    lines.extend(_format_interventions_section(trial))
    lines.extend(_format_eligibility_section(trial))
    lines.extend(_format_biomarker_section(trial))
    lines.extend(await _format_organizations_section(trial, api_key))
    lines.extend(_format_locations_section(trial))
    lines.extend(_format_contact_section(trial))

    # Footer
    lines.extend([
        "---",
        "*Source: NCI Clinical Trials Search API*",
    ])

    return "\n".join(lines)


def _format_site(site: dict[str, Any], lines: list[str]) -> None:
    """Helper to format a single site/location."""
    if isinstance(site, dict):
        name = site.get("org_name", site.get("facility", ""))
        city = site.get("city", "")
        state = site.get("state", "")
        country = site.get("country", "")

        location_parts = [p for p in [city, state] if p]
        if country and country != "United States":
            location_parts.append(country)

        if name:
            lines.append(f"**{name}**")
        if location_parts:
            lines.append(f"*{', '.join(location_parts)}*")

        # Contact info if available
        if site.get("contact_name"):
            lines.append(f"Contact: {site['contact_name']}")
        if site.get("contact_phone"):
            lines.append(f"Phone: {site['contact_phone']}")

        lines.append("")
    else:
        lines.append(f"- {site}")
        lines.append("")

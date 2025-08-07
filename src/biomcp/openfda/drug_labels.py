"""
OpenFDA Drug Labels (SPL) integration.
"""

import logging

from .constants import (
    OPENFDA_DEFAULT_LIMIT,
    OPENFDA_DISCLAIMER,
    OPENFDA_DRUG_LABELS_URL,
    OPENFDA_MAX_LIMIT,
)
from .drug_labels_helpers import (
    build_label_search_query,
    format_label_header,
    format_label_section,
    format_label_summary,
    get_default_sections,
    get_section_titles,
)
from .utils import clean_text, format_count, make_openfda_request

logger = logging.getLogger(__name__)


async def search_drug_labels(
    name: str | None = None,
    indication: str | None = None,
    boxed_warning: bool = False,
    section: str | None = None,
    limit: int = OPENFDA_DEFAULT_LIMIT,
    skip: int = 0,
    api_key: str | None = None,
) -> str:
    """
    Search FDA drug product labels (SPL).

    Args:
        name: Drug name to search for
        indication: Search for drugs indicated for this condition
        boxed_warning: Filter for drugs with boxed warnings
        section: Specific label section to search
        limit: Maximum number of results
        skip: Number of results to skip

        api_key: Optional OpenFDA API key (overrides OPENFDA_API_KEY env var)

    Returns:
        Formatted string with drug label information
    """
    if not name and not indication and not section and not boxed_warning:
        return (
            "⚠️ Please specify a drug name, indication, or label section to search.\n\n"
            "Examples:\n"
            "- Search by name: --name 'pembrolizumab'\n"
            "- Search by indication: --indication 'melanoma'\n"
            "- Search by section: --section 'contraindications'"
        )

    # Build and execute search
    search_query = build_label_search_query(
        name, indication, boxed_warning, section
    )
    params = {
        "search": search_query,
        "limit": min(limit, OPENFDA_MAX_LIMIT),
        "skip": skip,
    }

    response, error = await make_openfda_request(
        OPENFDA_DRUG_LABELS_URL, params, "openfda_drug_labels", api_key
    )

    if error:
        return f"⚠️ Error searching drug labels: {error}"

    if not response or not response.get("results"):
        return _format_no_results(name, indication, section)

    results = response["results"]
    total = (
        response.get("meta", {}).get("results", {}).get("total", len(results))
    )

    # Build output
    output = ["## FDA Drug Labels\n"]
    output.extend(_format_search_summary(name, indication, section, total))

    # Display results
    output.append(
        f"### Results (showing {min(len(results), 5)} of {total}):\n"
    )
    for i, result in enumerate(results[:5], 1):
        output.extend(format_label_summary(result, i))

    # Add tip for getting full labels
    if total > 0 and results and "set_id" in results[0]:
        output.append(
            "\n💡 **Tip**: Use `biomcp openfda label-get <label_id>` to retrieve "
            "the complete label for any drug."
        )

    output.append(f"\n{OPENFDA_DISCLAIMER}")
    return "\n".join(output)


def _format_no_results(
    name: str | None, indication: str | None, section: str | None
) -> str:
    """Format no results message."""
    search_desc = []
    if name:
        search_desc.append(f"drug '{name}'")
    if indication:
        search_desc.append(f"indication '{indication}'")
    if section:
        search_desc.append(f"section '{section}'")
    return f"No drug labels found for {' and '.join(search_desc)}."


def _format_search_summary(
    name: str | None, indication: str | None, section: str | None, total: int
) -> list[str]:
    """Format the search summary."""
    output = []

    search_desc = []
    if name:
        search_desc.append(f"**Drug**: {name}")
    if indication:
        search_desc.append(f"**Indication**: {indication}")
    if section:
        search_desc.append(f"**Section**: {section}")

    if search_desc:
        output.append(" | ".join(search_desc))
    output.append(f"**Total Labels Found**: {format_count(total, 'label')}\n")

    return output


async def get_drug_label(
    set_id: str,
    sections: list[str] | None = None,
    api_key: str | None = None,
) -> str:
    """
    Get detailed drug label information by set ID.

    Args:
        set_id: Label set ID
        sections: Specific sections to retrieve (default: key sections)

        api_key: Optional OpenFDA API key (overrides OPENFDA_API_KEY env var)

    Returns:
        Formatted string with detailed label information
    """
    params = {
        "search": f'set_id:"{set_id}"',
        "limit": 1,
    }

    response, error = await make_openfda_request(
        OPENFDA_DRUG_LABELS_URL, params, "openfda_drug_label_detail", api_key
    )

    if error:
        return f"⚠️ Error retrieving drug label: {error}"

    if not response or not response.get("results"):
        return f"Drug label with ID '{set_id}' not found."

    result = response["results"][0]

    # Use default sections if not specified
    if not sections:
        sections = get_default_sections()

    # Build output
    output = format_label_header(result, set_id)

    # Boxed warning (if exists)
    if "boxed_warning" in result:
        output.extend(_format_boxed_warning(result["boxed_warning"]))

    # Display requested sections
    section_titles = get_section_titles()
    for section in sections:
        output.extend(format_label_section(result, section, section_titles))

    output.append(f"\n{OPENFDA_DISCLAIMER}")
    return "\n".join(output)


def _format_boxed_warning(boxed_warning: list) -> list[str]:
    """Format boxed warning section."""
    output = ["### ⚠️ BOXED WARNING\n"]
    warning_text = clean_text(" ".join(boxed_warning))
    output.append(warning_text)
    output.append("")
    return output

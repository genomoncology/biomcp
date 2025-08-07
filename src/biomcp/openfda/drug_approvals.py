"""
OpenFDA drug approvals (Drugs@FDA) integration.
"""

import logging
from typing import Any

from .constants import (
    OPENFDA_DEFAULT_LIMIT,
    OPENFDA_DISCLAIMER,
    OPENFDA_DRUGSFDA_URL,
)
from .utils import (
    format_count,
    make_openfda_request,
)

logger = logging.getLogger(__name__)


async def search_drug_approvals(
    drug: str | None = None,
    application_number: str | None = None,
    approval_year: str | None = None,
    limit: int = OPENFDA_DEFAULT_LIMIT,
    skip: int = 0,
    api_key: str | None = None,
) -> str:
    """
    Search FDA drug approval records from Drugs@FDA.

    Args:
        drug: Drug name (brand or generic) to search for
        application_number: NDA or BLA application number
        approval_year: Year of approval (YYYY format)
        limit: Maximum number of results to return
        skip: Number of results to skip (for pagination)

        api_key: Optional OpenFDA API key (overrides OPENFDA_API_KEY env var)

    Returns:
        Formatted string with drug approval information
    """
    # Build search query
    search_params = {}

    if drug:
        # Search both brand and generic names
        search_params["search"] = (
            f'(openfda.brand_name:"{drug}" OR '
            f'openfda.generic_name:"{drug}" OR '
            f'openfda.substance_name:"{drug}")'
        )
    elif application_number:
        search_params["search"] = f'application_number:"{application_number}"'
    elif approval_year:
        # Search for approvals in a specific year
        search_params["search"] = (
            f"products.marketing_status_date:[{approval_year}-01-01 TO {approval_year}-12-31]"
        )

    # Add pagination
    search_params["limit"] = str(min(limit, 100))
    search_params["skip"] = str(skip)

    # Sort by submission date (most recent first)
    search_params["sort"] = "submissions.submission_status_date:desc"

    # Make the request
    response, error = await make_openfda_request(
        OPENFDA_DRUGSFDA_URL, search_params, "openfda_approvals", api_key
    )

    if error:
        return f"⚠️ Error searching drug approvals: {error}"

    if not response or not response.get("results"):
        return "No drug approval records found matching your criteria."

    # Format the results
    results = response["results"]
    total = (
        response.get("meta", {}).get("results", {}).get("total", len(results))
    )

    output = ["## FDA Drug Approval Records\n"]

    if drug:
        output.append(f"**Drug**: {drug}")
    if application_number:
        output.append(f"**Application**: {application_number}")
    if approval_year:
        output.append(f"**Approval Year**: {approval_year}")

    output.append(
        f"**Total Records Found**: {format_count(total, 'record')}\n"
    )

    # Show results
    output.append(f"### Results (showing {len(results)} of {total}):\n")

    for i, record in enumerate(results, 1):
        output.extend(_format_approval_summary(record, i))

    output.append(f"\n{OPENFDA_DISCLAIMER}")

    return "\n".join(output)


async def get_drug_approval(
    application_number: str,
    api_key: str | None = None,
) -> str:
    """
    Get detailed drug approval information for a specific application.

    Args:
        application_number: NDA or BLA application number

        api_key: Optional OpenFDA API key (overrides OPENFDA_API_KEY env var)

    Returns:
        Formatted string with detailed approval information
    """
    # Search for the specific application
    search_params = {
        "search": f'application_number:"{application_number}"',
        "limit": 1,
    }

    response, error = await make_openfda_request(
        OPENFDA_DRUGSFDA_URL, search_params, "openfda_approvals", api_key
    )

    if error:
        return f"⚠️ Error retrieving drug approval: {error}"

    if not response or not response.get("results"):
        return f"No approval record found for application {application_number}"

    record = response["results"][0]

    # Format detailed approval information
    output = [f"## Drug Approval Details: {application_number}\n"]

    # Basic information
    output.extend(_format_approval_header(record))

    # Products
    if products := record.get("products"):
        output.extend(_format_products(products))

    # Submissions history
    if submissions := record.get("submissions"):
        output.extend(_format_submissions(submissions))

    # OpenFDA metadata
    if openfda := record.get("openfda"):
        output.extend(_format_openfda_metadata(openfda))

    output.append(f"\n{OPENFDA_DISCLAIMER}")

    return "\n".join(output)


def _format_approval_summary(record: dict[str, Any], num: int) -> list[str]:
    """Format a single approval record summary."""
    output = [
        f"#### {num}. Application {record.get('application_number', 'Unknown')}"
    ]

    # Get sponsor/applicant
    if sponsor := record.get("sponsor_name"):
        output.append(f"**Sponsor**: {sponsor}")

    # Get drug names from OpenFDA data
    openfda = record.get("openfda", {})
    if brand_names := openfda.get("brand_name"):
        output.append(f"**Brand Name(s)**: {', '.join(brand_names[:3])}")
    if generic_names := openfda.get("generic_name"):
        output.append(f"**Generic Name(s)**: {', '.join(generic_names[:3])}")

    # Get products and their approval dates
    if products := record.get("products"):
        output.append("\n**Products**:")
        for prod in products[:3]:
            prod_num = prod.get("product_number", "?")
            dosage = prod.get("dosage_form", "")
            strength = prod.get("strength", "")
            status = prod.get("marketing_status", "")

            prod_line = f"- Product {prod_num}: {dosage}"
            if strength:
                prod_line += f" ({strength})"
            if status:
                prod_line += f" - {status}"
            output.append(prod_line)

    # Get most recent submission
    if submissions := record.get("submissions"):
        # Sort by date to get most recent
        recent = submissions[0]
        sub_type = recent.get("submission_type", "")
        sub_status = recent.get("submission_status", "")
        sub_date = recent.get("submission_status_date", "")

        if sub_date:
            output.append(
                f"\n**Latest Activity**: {sub_type} - {sub_status} ({sub_date})"
            )

    output.append("")
    return output


def _format_approval_header(record: dict[str, Any]) -> list[str]:
    """Format the header section of detailed approval."""
    output = ["### Application Information"]

    output.append(
        f"**Application Number**: {record.get('application_number', 'Unknown')}"
    )

    if sponsor := record.get("sponsor_name"):
        output.append(f"**Sponsor**: {sponsor}")

    # OpenFDA names
    openfda = record.get("openfda", {})
    if brand_names := openfda.get("brand_name"):
        output.append(f"**Brand Names**: {', '.join(brand_names)}")
    if generic_names := openfda.get("generic_name"):
        output.append(f"**Generic Names**: {', '.join(generic_names)}")
    if substances := openfda.get("substance_name"):
        output.append(f"**Active Substances**: {', '.join(substances)}")

    output.append("")
    return output


def _format_products(products: list[dict[str, Any]]) -> list[str]:
    """Format product information."""
    output = ["### Products"]

    for prod in products:
        prod_num = prod.get("product_number", "Unknown")
        output.append(f"\n#### Product {prod_num}")

        if dosage := prod.get("dosage_form"):
            output.append(f"**Dosage Form**: {dosage}")
        if strength := prod.get("strength"):
            output.append(f"**Strength**: {strength}")
        if route := prod.get("route"):
            output.append(f"**Route**: {route}")
        if status := prod.get("marketing_status"):
            output.append(f"**Marketing Status**: {status}")
        if status_date := prod.get("marketing_status_date"):
            output.append(f"**Status Date**: {status_date}")
        if te_code := prod.get("te_code"):
            output.append(f"**Therapeutic Equivalence**: {te_code}")

    output.append("")
    return output


def _format_submissions(submissions: list[dict[str, Any]]) -> list[str]:
    """Format submission history."""
    output = ["### Submission History"]

    # Show most recent 5 submissions
    for sub in submissions[:5]:
        sub_num = sub.get("submission_number", "?")
        sub_type = sub.get("submission_type", "Unknown")
        sub_status = sub.get("submission_status", "")
        sub_date = sub.get("submission_status_date", "")

        output.append(f"\n**Submission {sub_num}**: {sub_type}")
        if sub_status:
            output.append(f"- Status: {sub_status}")
        if sub_date:
            output.append(f"- Date: {sub_date}")

        # Review priority if present
        if priority := sub.get("review_priority"):
            output.append(f"- Review Priority: {priority}")

        # Submission class if present
        if sub_class := sub.get("submission_class_code"):
            class_desc = sub.get("submission_class_code_description", "")
            output.append(f"- Class: {sub_class} - {class_desc}")

    output.append("")
    return output


def _format_openfda_metadata(openfda: dict[str, Any]) -> list[str]:
    """Format OpenFDA metadata."""
    output = ["### Additional Information"]

    if nui := openfda.get("nui"):
        output.append(f"**NUI Codes**: {', '.join(nui[:5])}")

    if pharm_class := openfda.get("pharm_class_epc"):
        output.append(f"**Pharmacologic Class**: {', '.join(pharm_class[:3])}")

    if moa := openfda.get("pharm_class_moa"):
        output.append(f"**Mechanism of Action**: {', '.join(moa[:3])}")

    if unii := openfda.get("unii"):
        output.append(f"**UNII Codes**: {', '.join(unii[:5])}")

    output.append("")
    return output

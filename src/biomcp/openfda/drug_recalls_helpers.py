"""
Helper functions for drug recall search to reduce complexity.
"""


def build_drug_search_query(drug: str) -> str:
    """Build search query for drug name."""
    return (
        f'(openfda.brand_name:"{drug}" OR '
        f'openfda.generic_name:"{drug}" OR '
        f'product_description:"{drug}")'
    )


def build_class_search_query(recall_class: str) -> str | None:
    """Build search query for recall classification."""
    # Handle various input formats
    recall_class = recall_class.strip()

    # If already in "Class X" format, use it directly
    if recall_class.upper().startswith("CLASS "):
        return f'classification:"{recall_class.title()}"'

    # Map single digits/numerals to Class format
    class_map = {
        "1": "Class I",
        "I": "Class I",
        "2": "Class II",
        "II": "Class II",
        "3": "Class III",
        "III": "Class III",
    }
    if mapped_class := class_map.get(recall_class.upper()):
        return f'classification:"{mapped_class}"'
    return None


def build_status_search_query(status: str) -> str | None:
    """Build search query for recall status."""
    status_lower = status.lower()
    if status_lower in ["ongoing", "completed", "terminated"]:
        return f'status:"{status_lower.capitalize()}"'
    return None


def build_date_search_query(since_date: str) -> str | None:
    """Build search query for date range."""
    if len(since_date) == 8:
        formatted_date = f"{since_date[:4]}-{since_date[4:6]}-{since_date[6:]}"
        return f"recall_initiation_date:[{formatted_date} TO *]"
    return None


def format_recall_search_header(
    drug: str | None,
    recall_class: str | None,
    status: str | None,
    since_date: str | None,
    total: int,
) -> list[str]:
    """Format header for recall search results."""
    output = []

    if drug:
        output.append(f"**Drug**: {drug}")
    if recall_class:
        output.append(f"**Classification**: Class {recall_class}")
    if status:
        output.append(f"**Status**: {status}")
    if since_date:
        output.append(f"**Since**: {since_date}")

    return output


def build_recall_search_params(
    drug: str | None,
    recall_class: str | None,
    status: str | None,
    reason: str | None,
    since_date: str | None,
    limit: int,
    skip: int,
) -> dict:
    """Build search parameters for recall API."""
    # Build search query
    search_parts = []

    # Default to human drugs only (exclude veterinary)
    search_parts.append('product_type:"Human"')

    if drug:
        search_parts.append(build_drug_search_query(drug))

    if recall_class and (
        class_query := build_class_search_query(recall_class)
    ):
        search_parts.append(class_query)

    if status and (status_query := build_status_search_query(status)):
        search_parts.append(status_query)

    if reason:
        search_parts.append(f'reason_for_recall:"{reason}"')

    if since_date and (date_query := build_date_search_query(since_date)):
        search_parts.append(date_query)

    # Combine search parts
    search_params = {}
    if search_parts:
        search_params["search"] = " AND ".join(search_parts)

    # Add pagination
    search_params["limit"] = str(min(limit, 100))
    search_params["skip"] = str(skip)

    # Sort by recall date (most recent first)
    search_params["sort"] = "recall_initiation_date:desc"

    return search_params

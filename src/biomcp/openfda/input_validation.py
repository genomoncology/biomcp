"""
Input validation and sanitization for OpenFDA API requests.

This module provides security-focused input validation to prevent injection attacks
and ensure data integrity for all FDA API requests.
"""

import logging
import re
from typing import Any

logger = logging.getLogger(__name__)

# Maximum lengths for different input types
MAX_DRUG_NAME_LENGTH = 100
MAX_REACTION_LENGTH = 200
MAX_GENERAL_QUERY_LENGTH = 500
MAX_DATE_LENGTH = 10

# Patterns for validation
SAFE_CHARS_PATTERN = re.compile(r"^[a-zA-Z0-9\s\-\.\,\(\)\/\*]+$")
DATE_PATTERN = re.compile(r"^\d{4}-\d{2}-\d{2}$")
# Include SQL comment pattern -- and other injection patterns
INJECTION_CHARS = re.compile(r"[<>\"\';&|\\`${}]|--")


def sanitize_input(
    value: str | None, max_length: int = MAX_GENERAL_QUERY_LENGTH
) -> str | None:
    """
    Sanitize user input to prevent injection attacks.

    Args:
        value: Input string to sanitize
        max_length: Maximum allowed length

    Returns:
        Sanitized string or None if input is invalid
    """
    if not value:
        return None

    # Convert to string and strip whitespace
    value = str(value).strip()

    # Check length
    if len(value) > max_length:
        logger.warning(
            f"Input truncated from {len(value)} to {max_length} characters"
        )
        value = value[:max_length]

    # Remove potential injection characters
    cleaned = INJECTION_CHARS.sub("", value)

    # Warn if characters were removed
    if cleaned != value:
        logger.warning("Removed potentially dangerous characters from input")

    # Normalize whitespace
    cleaned = " ".join(cleaned.split())

    return cleaned if cleaned else None


def validate_drug_name(drug: str | None) -> str | None:
    """
    Validate and sanitize drug name input.

    Args:
        drug: Drug name to validate

    Returns:
        Validated drug name or None
    """
    if not drug:
        return None

    sanitized = sanitize_input(drug, MAX_DRUG_NAME_LENGTH)

    if not sanitized:
        return None

    # Drug names should only contain alphanumeric, spaces, hyphens, and slashes
    if not re.match(r"^[a-zA-Z0-9\s\-\/\(\)]+$", sanitized):
        logger.warning(f"Invalid drug name format: {sanitized[:20]}...")
        return None

    return sanitized


def validate_date(date_str: str | None) -> str | None:
    """
    Validate date string format.

    Args:
        date_str: Date string in YYYY-MM-DD format

    Returns:
        Validated date string or None
    """
    if not date_str:
        return None

    sanitized = sanitize_input(date_str, MAX_DATE_LENGTH)

    if not sanitized:
        return None

    # Check date format
    if not DATE_PATTERN.match(sanitized):
        logger.warning(f"Invalid date format: {sanitized}")
        return None

    # Basic date validation
    try:
        year, month, day = map(int, sanitized.split("-"))
        if not (1900 <= year <= 2100 and 1 <= month <= 12 and 1 <= day <= 31):
            logger.warning(f"Date out of valid range: {sanitized}")
            return None
    except (ValueError, IndexError):
        logger.warning(f"Cannot parse date: {sanitized}")
        return None

    return sanitized


def validate_limit(limit: int | None, max_limit: int = 100) -> int:
    """
    Validate and constrain limit parameter.

    Args:
        limit: Requested limit
        max_limit: Maximum allowed limit

    Returns:
        Valid limit value
    """
    if limit is None:
        return 25  # Default

    try:
        limit = int(limit)
    except (ValueError, TypeError):
        logger.warning(f"Invalid limit value: {limit}")
        return 25

    if limit < 1:
        return 1
    elif limit > max_limit:
        logger.warning(f"Limit {limit} exceeds maximum {max_limit}")
        return max_limit

    return limit


def validate_skip(skip: int | None, max_skip: int = 10000) -> int:
    """
    Validate and constrain skip/offset parameter.

    Args:
        skip: Requested skip/offset
        max_skip: Maximum allowed skip

    Returns:
        Valid skip value
    """
    if skip is None:
        return 0

    try:
        skip = int(skip)
    except (ValueError, TypeError):
        logger.warning(f"Invalid skip value: {skip}")
        return 0

    if skip < 0:
        return 0
    elif skip > max_skip:
        logger.warning(f"Skip {skip} exceeds maximum {max_skip}")
        return max_skip

    return skip


def validate_classification(classification: str | None) -> str | None:
    """
    Validate recall classification.

    Args:
        classification: Classification string (Class I, II, or III)

    Returns:
        Validated classification or None
    """
    if not classification:
        return None

    sanitized = sanitize_input(classification, 20)

    if not sanitized:
        return None

    # Normalize classification format
    sanitized = sanitized.upper()

    # Check valid classifications
    valid_classes = [
        "CLASS I",
        "CLASS II",
        "CLASS III",
        "I",
        "II",
        "III",
        "1",
        "2",
        "3",
    ]

    if sanitized not in valid_classes:
        logger.warning(f"Invalid classification: {sanitized}")
        return None

    # Normalize to standard format
    if sanitized in ["I", "1"]:
        return "Class I"
    elif sanitized in ["II", "2"]:
        return "Class II"
    elif sanitized in ["III", "3"]:
        return "Class III"

    return sanitized.title()  # "CLASS I" -> "Class I"


def validate_status(status: str | None) -> str | None:
    """
    Validate status parameter.

    Args:
        status: Status string

    Returns:
        Validated status or None
    """
    if not status:
        return None

    sanitized = sanitize_input(status, 50)

    if not sanitized:
        return None

    # Normalize status
    sanitized = sanitized.lower()

    # Check valid statuses
    valid_statuses = [
        "ongoing",
        "terminated",
        "completed",
        "current",
        "resolved",
    ]

    if sanitized not in valid_statuses:
        logger.warning(f"Invalid status: {sanitized}")
        return None

    return sanitized.title()  # "ongoing" -> "Ongoing"


def validate_boolean(value: Any) -> bool | None:
    """
    Validate boolean parameter.

    Args:
        value: Boolean-like value

    Returns:
        Boolean value or None
    """
    if value is None:
        return None

    if isinstance(value, bool):
        return value

    if isinstance(value, str):
        value = value.lower().strip()
        if value in ["true", "1", "yes", "y"]:
            return True
        elif value in ["false", "0", "no", "n"]:
            return False

    return None


def validate_api_key(api_key: str | None) -> str | None:
    """
    Validate API key format.

    Args:
        api_key: API key string

    Returns:
        Validated API key or None
    """
    if not api_key:
        return None

    # API keys should be alphanumeric with possible hyphens
    if not re.match(r"^[a-zA-Z0-9\-_]+$", api_key):
        logger.warning("Invalid API key format")
        return None

    # Check reasonable length
    if len(api_key) < 10 or len(api_key) > 100:
        logger.warning("API key length out of expected range")
        return None

    return api_key


def _validate_parameter(key: str, value: Any) -> Any:
    """Validate a single parameter based on its key."""
    if key in ["drug", "brand", "generic"]:
        return validate_drug_name(value)
    elif key in ["limit"]:
        return validate_limit(value)
    elif key in ["skip", "offset"]:
        return validate_skip(value)
    elif key in ["classification"]:
        return validate_classification(value)
    elif key in ["status"]:
        return validate_status(value)
    elif key in ["serious", "death", "ongoing"]:
        return validate_boolean(value)
    elif key in ["api_key"]:
        return validate_api_key(value)
    elif "date" in key.lower():
        return validate_date(value)
    else:
        return sanitize_input(value)


def build_safe_query(params: dict[str, Any]) -> dict[str, Any]:
    """
    Build a safe query dictionary with validated parameters.

    Args:
        params: Raw parameters dictionary

    Returns:
        Dictionary with validated parameters
    """
    safe_params = {}

    for key, value in params.items():
        if value is None:
            continue

        # Validate key name
        if not re.match(r"^[a-zA-Z_][a-zA-Z0-9_]*$", key):
            logger.warning(f"Skipping invalid parameter key: {key}")
            continue

        # Validate parameter value
        validated = _validate_parameter(key, value)

        if validated is not None:
            safe_params[key] = validated

    return safe_params

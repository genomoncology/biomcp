"""Validation functions for OpenFDA API responses."""

import logging
from typing import Any

from .exceptions import OpenFDAValidationError

logger = logging.getLogger(__name__)


def validate_fda_response(
    response: dict[str, Any],
    required_fields: list[str] | None = None,
    response_type: str = "generic",
) -> bool:
    """
    Validate FDA API response structure.

    Args:
        response: The FDA API response dictionary
        required_fields: List of required top-level fields
        response_type: Type of response for specific validation

    Returns:
        True if valid

    Raises:
        OpenFDAValidationError: If validation fails
    """
    if not isinstance(response, dict):
        raise OpenFDAValidationError(
            f"Expected dict response, got {type(response).__name__}"
        )

    # Default required fields for most FDA responses
    if required_fields is None:
        required_fields = ["results"] if "results" in response else []

    # Check required fields
    missing_fields = [
        field for field in required_fields if field not in response
    ]
    if missing_fields:
        raise OpenFDAValidationError(
            f"Missing required fields in FDA response: {', '.join(missing_fields)}"
        )

    # Type-specific validation
    if response_type == "search":
        validate_search_response(response)
    elif response_type == "detail":
        validate_detail_response(response)

    return True


def validate_search_response(response: dict[str, Any]) -> bool:
    """
    Validate FDA search response structure.

    Args:
        response: FDA search response

    Returns:
        True if valid

    Raises:
        OpenFDAValidationError: If validation fails
    """
    # Search responses should have results array
    if "results" not in response:
        raise OpenFDAValidationError("Search response missing 'results' field")

    if not isinstance(response["results"], list):
        raise OpenFDAValidationError(
            f"Expected 'results' to be a list, got {type(response['results']).__name__}"
        )

    # If meta is present, validate it
    if "meta" in response:
        validate_meta_field(response["meta"])

    return True


def validate_detail_response(response: dict[str, Any]) -> bool:
    """
    Validate FDA detail response structure.

    Args:
        response: FDA detail response

    Returns:
        True if valid

    Raises:
        OpenFDAValidationError: If validation fails
    """
    # Detail responses usually have a single result
    if "results" in response:
        if not isinstance(response["results"], list):
            raise OpenFDAValidationError(
                f"Expected 'results' to be a list, got {type(response['results']).__name__}"
            )

        if len(response["results"]) == 0:
            # Empty results is valid (not found)
            return True

        if len(response["results"]) > 1:
            logger.warning(
                f"Detail response contains {len(response['results'])} results, expected 1"
            )

    return True


def validate_meta_field(meta: dict[str, Any]) -> bool:
    """
    Validate FDA response meta field.

    Args:
        meta: Meta field from FDA response

    Returns:
        True if valid

    Raises:
        OpenFDAValidationError: If validation fails
    """
    if not isinstance(meta, dict):
        raise OpenFDAValidationError(
            f"Expected 'meta' to be a dict, got {type(meta).__name__}"
        )

    # Check for results metadata
    if "results" in meta:
        results_meta = meta["results"]
        if not isinstance(results_meta, dict):
            raise OpenFDAValidationError(
                f"Expected 'meta.results' to be a dict, got {type(results_meta).__name__}"
            )

        # Validate pagination fields if present
        for field in ["skip", "limit", "total"]:
            if field in results_meta and not isinstance(
                results_meta[field], int | float
            ):
                raise OpenFDAValidationError(
                    f"Expected 'meta.results.{field}' to be numeric, "
                    f"got {type(results_meta[field]).__name__}"
                )

    return True


def validate_adverse_event(event: dict[str, Any]) -> bool:
    """
    Validate an adverse event record.

    Args:
        event: Adverse event record

    Returns:
        True if valid

    Raises:
        OpenFDAValidationError: If validation fails
    """
    if not isinstance(event, dict):
        raise OpenFDAValidationError(
            f"Expected adverse event to be a dict, got {type(event).__name__}"
        )

    # Key fields that should be present (but may be null)
    important_fields = ["patient", "safetyreportid"]

    for field in important_fields:
        if field not in event:
            logger.warning(f"Adverse event missing expected field: {field}")

    return True


def validate_drug_label(label: dict[str, Any]) -> bool:
    """
    Validate a drug label record.

    Args:
        label: Drug label record

    Returns:
        True if valid

    Raises:
        OpenFDAValidationError: If validation fails
    """
    if not isinstance(label, dict):
        raise OpenFDAValidationError(
            f"Expected drug label to be a dict, got {type(label).__name__}"
        )

    # Labels should have OpenFDA section
    if "openfda" not in label:
        logger.warning("Drug label missing 'openfda' section")

    # Should have at least one section
    label_sections = [
        "indications_and_usage",
        "contraindications",
        "warnings_and_precautions",
        "adverse_reactions",
        "dosage_and_administration",
    ]

    has_section = any(section in label for section in label_sections)
    if not has_section:
        logger.warning("Drug label has no standard sections")

    return True


def validate_device_event(event: dict[str, Any]) -> bool:
    """
    Validate a device event record.

    Args:
        event: Device event record

    Returns:
        True if valid

    Raises:
        OpenFDAValidationError: If validation fails
    """
    if not isinstance(event, dict):
        raise OpenFDAValidationError(
            f"Expected device event to be a dict, got {type(event).__name__}"
        )

    # Device events should have MDR report key
    if "mdr_report_key" not in event:
        logger.warning("Device event missing 'mdr_report_key'")

    # Should have device information
    if "device" not in event and "devices" not in event:
        logger.warning("Device event missing device information")

    return True


def validate_recall(recall: dict[str, Any]) -> bool:
    """
    Validate a recall record.

    Args:
        recall: Recall record

    Returns:
        True if valid

    Raises:
        OpenFDAValidationError: If validation fails
    """
    if not isinstance(recall, dict):
        raise OpenFDAValidationError(
            f"Expected recall to be a dict, got {type(recall).__name__}"
        )

    # Required fields for recalls
    required = ["recall_number", "classification", "product_description"]

    for field in required:
        if field not in recall:
            logger.warning(f"Recall missing required field: {field}")

    # Validate classification if present
    if "classification" in recall:
        valid_classes = ["Class I", "Class II", "Class III", "1", "2", "3"]
        if recall["classification"] not in valid_classes:
            logger.warning(
                f"Invalid recall classification: {recall['classification']}"
            )

    return True


def sanitize_response(response: dict[str, Any]) -> dict[str, Any]:
    """
    Sanitize FDA response to handle common issues.

    Args:
        response: Raw FDA response

    Returns:
        Sanitized response
    """
    if not response:
        return {}

    # Handle fields that can be string or list
    if "results" in response and isinstance(response["results"], list):
        for result in response["results"]:
            if isinstance(result, dict):
                # Fields that can be string or list
                polymorphic_fields = [
                    "source_type",
                    "remedial_action",
                    "medical_specialty_description",
                    "manufacturer_name",
                    "brand_name",
                    "generic_name",
                ]

                for field in polymorphic_fields:
                    if field in result:
                        value = result[field]
                        # Ensure consistent list format
                        if not isinstance(value, list):
                            result[field] = [value] if value else []

    return response

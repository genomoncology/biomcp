"""
Utility functions for OpenFDA API integration.
"""

import asyncio
import logging
import os
from typing import Any

from ..http_client import request_api
from .cache import (
    get_cached_response,
    is_cacheable_request,
    set_cached_response,
)
from .exceptions import (
    OpenFDAConnectionError,
    OpenFDARateLimitError,
    OpenFDATimeoutError,
    OpenFDAValidationError,
)
from .input_validation import build_safe_query
from .rate_limiter import FDA_CIRCUIT_BREAKER, FDA_RATE_LIMITER, FDA_SEMAPHORE
from .validation import sanitize_response, validate_fda_response

logger = logging.getLogger(__name__)


def get_api_key() -> str | None:
    """Get OpenFDA API key from environment variable."""
    api_key = os.environ.get("OPENFDA_API_KEY")
    if not api_key:
        logger.debug("No OPENFDA_API_KEY found in environment")
    return api_key


async def make_openfda_request(  # noqa: C901
    endpoint: str,
    params: dict[str, Any],
    domain: str = "openfda",
    api_key: str | None = None,
    max_retries: int = 3,
    initial_delay: float = 1.0,
) -> tuple[dict[str, Any] | None, str | None]:
    """
    Make a request to the OpenFDA API with retry logic and caching.

    Args:
        endpoint: Full URL to the OpenFDA endpoint
        params: Query parameters
        domain: Domain name for metrics tracking
        api_key: Optional API key (overrides environment variable)
        max_retries: Maximum number of retry attempts (default 3)
        initial_delay: Initial delay in seconds for exponential backoff (default 1.0)

    Returns:
        Tuple of (response_data, error_message)
    """
    # Validate and sanitize input parameters
    safe_params = build_safe_query(params)

    # Check cache first (with safe params)
    if is_cacheable_request(endpoint, safe_params):
        cached_response = get_cached_response(endpoint, safe_params)
        if cached_response:
            return cached_response, None

    # Use provided API key or get from environment
    if not api_key:
        api_key = get_api_key()
    if api_key:
        safe_params["api_key"] = api_key

    last_error = None
    delay = initial_delay

    for attempt in range(max_retries + 1):
        try:
            # Apply rate limiting and circuit breaker
            async with FDA_SEMAPHORE:
                await FDA_RATE_LIMITER.acquire()

                # Check circuit breaker state
                if FDA_CIRCUIT_BREAKER.is_open:
                    state = FDA_CIRCUIT_BREAKER.get_state()
                    return None, f"FDA API circuit breaker is open: {state}"

                response, error = await request_api(
                    url=endpoint,
                    request=safe_params,
                    method="GET",
                    domain=domain,
                )

            if error:
                error_msg = (
                    error.message if hasattr(error, "message") else str(error)
                )

                # Check for specific error types
                if "429" in error_msg or "rate limit" in error_msg.lower():
                    if attempt < max_retries:
                        logger.warning(
                            f"Rate limit hit (attempt {attempt + 1}/{max_retries + 1}). "
                            f"Retrying in {delay:.1f} seconds..."
                        )
                        await asyncio.sleep(delay)
                        delay *= 2  # Exponential backoff
                        continue
                    else:
                        raise OpenFDARateLimitError(error_msg)

                # Check if error is retryable
                if _is_retryable_error(error_msg) and attempt < max_retries:
                    logger.warning(
                        f"OpenFDA API error (attempt {attempt + 1}/{max_retries + 1}): {error_msg}. "
                        f"Retrying in {delay:.1f} seconds..."
                    )
                    await asyncio.sleep(delay)
                    delay *= 2  # Exponential backoff
                    continue

                logger.error(f"OpenFDA API error: {error_msg}")
                return None, error_msg

            # Validate and sanitize response
            if response:
                try:
                    validate_fda_response(response, response_type="search")
                    response = sanitize_response(response)
                except OpenFDAValidationError as e:
                    logger.error(f"Invalid FDA response: {e}")
                    return None, str(e)

                # Cache successful response
                if is_cacheable_request(endpoint, safe_params):
                    set_cached_response(endpoint, safe_params, response)

            return response, None

        except asyncio.TimeoutError:
            last_error = "Request timeout"
            if attempt < max_retries:
                logger.warning(
                    f"OpenFDA request timeout (attempt {attempt + 1}/{max_retries + 1}). "
                    f"Retrying in {delay:.1f} seconds..."
                )
                await asyncio.sleep(delay)
                delay *= 2
                continue
            logger.error(
                f"OpenFDA request failed after {max_retries + 1} attempts: {last_error}"
            )
            raise OpenFDATimeoutError(last_error) from None

        except ConnectionError as e:
            last_error = f"Connection error: {e}"
            if attempt < max_retries:
                logger.warning(
                    f"OpenFDA connection error (attempt {attempt + 1}/{max_retries + 1}): {e}. "
                    f"Retrying in {delay:.1f} seconds..."
                )
                await asyncio.sleep(delay)
                delay *= 2
                continue
            logger.error(
                f"OpenFDA request failed after {max_retries + 1} attempts: {last_error}"
            )
            raise OpenFDAConnectionError(last_error) from None

        except (
            OpenFDARateLimitError,
            OpenFDATimeoutError,
            OpenFDAConnectionError,
        ):
            # Re-raise our custom exceptions
            raise
        except Exception as e:
            # Handle unexpected errors gracefully
            logger.error(f"Unexpected OpenFDA request error: {e}")
            return None, str(e)

    return None, last_error


def _is_retryable_error(error_msg: str) -> bool:
    """
    Check if an error is retryable.

    Args:
        error_msg: Error message string

    Returns:
        True if the error is retryable
    """
    retryable_patterns = [
        "rate limit",
        "timeout",
        "connection",
        "503",  # Service unavailable
        "502",  # Bad gateway
        "504",  # Gateway timeout
        "429",  # Too many requests
        "temporary",
        "try again",
    ]

    error_lower = error_msg.lower()
    return any(pattern in error_lower for pattern in retryable_patterns)


def format_count(count: int, label: str) -> str:
    """Format a count with appropriate singular/plural label."""
    if count == 1:
        return f"1 {label}"
    return f"{count:,} {label}s"


def truncate_text(text: str, max_length: int = 500) -> str:
    """Truncate text to a maximum length with ellipsis."""
    if len(text) <= max_length:
        return text
    return text[: max_length - 3] + "..."


def clean_text(text: str | None) -> str:
    """Clean and normalize text from FDA data."""
    if not text:
        return ""

    # Remove extra whitespace and newlines
    text = " ".join(text.split())

    # Remove common FDA formatting artifacts
    text = text.replace("\\n", " ")
    text = text.replace("\\r", " ")
    text = text.replace("\\t", " ")

    return text.strip()


def build_search_query(
    field_map: dict[str, str], operator: str = "AND"
) -> str:
    """
    Build an OpenFDA search query from field mappings.

    Args:
        field_map: Dictionary mapping field names to search values
        operator: Logical operator (AND/OR) to combine fields

    Returns:
        Formatted search query string
    """
    query_parts = []

    for field, value in field_map.items():
        if value:
            # Escape special characters
            escaped_value = value.replace('"', '\\"')
            # Add quotes for multi-word values
            if " " in escaped_value:
                escaped_value = f'"{escaped_value}"'
            query_parts.append(f"{field}:{escaped_value}")

    return f" {operator} ".join(query_parts)


def extract_drug_names(result: dict[str, Any]) -> list[str]:
    """Extract drug names from an OpenFDA result."""
    drug_names = set()

    # Check patient drug info (for adverse events)
    if "patient" in result:
        drugs = result.get("patient", {}).get("drug", [])
        for drug in drugs:
            if "medicinalproduct" in drug:
                drug_names.add(drug["medicinalproduct"])
            # Check OpenFDA fields
            openfda = drug.get("openfda", {})
            if "brand_name" in openfda:
                drug_names.update(openfda["brand_name"])
            if "generic_name" in openfda:
                drug_names.update(openfda["generic_name"])

    # Check direct OpenFDA fields (for labels)
    if "openfda" in result:
        openfda = result["openfda"]
        if "brand_name" in openfda:
            drug_names.update(openfda["brand_name"])
        if "generic_name" in openfda:
            drug_names.update(openfda["generic_name"])

    return sorted(drug_names)


def extract_reactions(result: dict[str, Any]) -> list[str]:
    """Extract reaction terms from an adverse event result."""
    reactions = []

    patient = result.get("patient", {})
    reaction_list = patient.get("reaction", [])

    for reaction in reaction_list:
        if "reactionmeddrapt" in reaction:
            reactions.append(reaction["reactionmeddrapt"])

    return reactions


def format_drug_list(drugs: list[str], max_items: int = 5) -> str:
    """Format a list of drug names for display."""
    if not drugs:
        return "None specified"

    if len(drugs) <= max_items:
        return ", ".join(drugs)

    shown = drugs[:max_items]
    remaining = len(drugs) - max_items
    return f"{', '.join(shown)} (+{remaining} more)"

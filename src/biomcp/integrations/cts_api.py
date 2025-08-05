"""NCI Clinical Trials Search API integration helper."""

import json
import logging
import os
from typing import Any, Literal

from ..constants import NCI_API_KEY_ENV
from ..http_client import request_api

logger = logging.getLogger(__name__)


class CTSAPIError(Exception):
    """Error raised when CTS API requests fail."""

    pass


def _validate_api_key(api_key: str | None) -> str:
    """Validate and return API key."""
    if not api_key:
        api_key = os.getenv(NCI_API_KEY_ENV)

    if not api_key:
        raise CTSAPIError(
            f"NCI API key required. Please set {NCI_API_KEY_ENV} environment "
            "variable or provide api_key parameter.\n"
            "Get a free API key at: https://www.cancer.gov/research/participate/"
            "clinical-trials-search/developers"
        )

    return api_key


def _prepare_request_data(
    method: str,
    params: dict[str, Any] | None,
    json_data: dict[str, Any] | None,
    headers: dict[str, str],
) -> dict[str, Any]:
    """Prepare request data based on method."""
    if method == "GET":
        request_data = params or {}
        logger.debug(f"CTS API GET request with params: {params}")
    else:
        request_data = json_data or {}
        if method == "POST":
            logger.debug(f"CTS API POST request with data: {json_data}")

    # Add headers to request data
    if headers:
        request_data["_headers"] = json.dumps(headers)

    return request_data


def _handle_api_error(error: Any) -> None:
    """Handle API errors with appropriate messages."""
    if error.code == 401:
        raise CTSAPIError(
            f"Invalid API key. Please check your {NCI_API_KEY_ENV} "
            "environment variable or api_key parameter."
        )
    elif error.code == 403:
        raise CTSAPIError(
            "Access forbidden. Your API key may not have permission "
            "to access this resource."
        )
    else:
        raise CTSAPIError(f"CTS API error: {error.message}")


async def make_cts_request(
    url: str,
    method: Literal["GET", "POST"] = "GET",
    params: dict[str, Any] | None = None,
    json_data: dict[str, Any] | None = None,
    api_key: str | None = None,
) -> dict[str, Any]:
    """
    Make a request to the NCI CTS API with proper authentication.

    Args:
        url: Full URL to the CTS API endpoint
        method: HTTP method (GET or POST)
        params: Query parameters
        json_data: JSON data for POST requests
        api_key: Optional API key (if not provided, uses NCI_API_KEY env var)

    Returns:
        JSON response from the API

    Raises:
        CTSAPIError: If the request fails or API key is missing
    """
    # Validate API key
    api_key = _validate_api_key(api_key)

    # Prepare headers
    headers = {"x-api-key": api_key, "Accept": "application/json"}

    try:
        # Prepare request data
        request_data = _prepare_request_data(
            method, params, json_data, headers
        )

        # Make API request
        response, error = await request_api(
            url=url,
            request=request_data,
            method=method,
            cache_ttl=0,  # Disable caching for NCI API to ensure fresh results
        )

        # Handle errors
        if error:
            _handle_api_error(error)

        if response is None:
            raise CTSAPIError("No response received from NCI CTS API")

        return response

    except Exception as e:
        # Re-raise CTSAPIError as-is
        if isinstance(e, CTSAPIError):
            raise

        # Wrap other exceptions
        logger.error(f"CTS API request failed: {e}")
        raise CTSAPIError(f"Failed to connect to NCI CTS API: {e!s}") from e


def get_api_key_instructions() -> str:
    """
    Get user-friendly instructions for obtaining and setting the API key.

    Returns:
        Formatted string with instructions
    """
    return (
        "## NCI Clinical Trials API Key Required\n\n"
        "To use NCI's Clinical Trials Search API, you need an API key.\n\n"
        "**Option 1: Set environment variable (recommended)**\n"
        "```bash\n"
        f"export {NCI_API_KEY_ENV}='your-api-key'\n"
        "```\n\n"
        "**Option 2: Provide via CLI**\n"
        "```bash\n"
        "biomcp trial search --api-key YOUR_KEY --condition melanoma\n"
        "```\n\n"
        "**Get your free API key:**\n"
        "Visit https://www.cancer.gov/research/participate/clinical-trials-search/developers\n\n"
        "The API key provides access to NCI's comprehensive cancer clinical trials "
        "database with advanced search capabilities."
    )

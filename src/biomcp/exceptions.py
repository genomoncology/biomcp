"""Custom exceptions for BioMCP."""

from typing import Any


class BioMCPError(Exception):
    """Base exception for all BioMCP errors."""

    def __init__(self, message: str, details: dict[str, Any] | None = None):
        super().__init__(message)
        self.message = message
        self.details = details or {}


class BioMCPSearchError(BioMCPError):
    """Base exception for search-related errors."""

    pass


class InvalidDomainError(BioMCPSearchError):
    """Raised when an invalid domain is specified."""

    def __init__(self, domain: str, valid_domains: list[str]):
        message = f"Unknown domain: {domain}. Valid domains are: {', '.join(valid_domains)}"
        super().__init__(
            message, {"domain": domain, "valid_domains": valid_domains}
        )


class InvalidParameterError(BioMCPSearchError):
    """Raised when invalid parameters are provided."""

    def __init__(self, parameter: str, value: Any, expected: str):
        message = f"Invalid value for parameter '{parameter}': {value}. Expected: {expected}"
        super().__init__(
            message,
            {"parameter": parameter, "value": value, "expected": expected},
        )


class SearchExecutionError(BioMCPSearchError):
    """Raised when a search fails to execute."""

    def __init__(self, domain: str, error: Exception):
        message = f"Failed to execute search for domain '{domain}': {error!s}"
        super().__init__(
            message, {"domain": domain, "original_error": str(error)}
        )


class ResultParsingError(BioMCPSearchError):
    """Raised when results cannot be parsed."""

    def __init__(self, domain: str, error: Exception):
        message = f"Failed to parse results for domain '{domain}': {error!s}"
        super().__init__(
            message, {"domain": domain, "original_error": str(error)}
        )


class QueryParsingError(BioMCPError):
    """Raised when a query cannot be parsed."""

    def __init__(self, query: str, error: Exception):
        message = f"Failed to parse query '{query}': {error!s}"
        super().__init__(
            message, {"query": query, "original_error": str(error)}
        )


class ThinkingError(BioMCPError):
    """Raised when sequential thinking encounters an error."""

    def __init__(self, thought_number: int, error: str):
        message = f"Error in thought {thought_number}: {error}"
        super().__init__(
            message, {"thought_number": thought_number, "error": error}
        )

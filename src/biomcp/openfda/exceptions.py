"""Custom exceptions for OpenFDA integration."""


class OpenFDAError(Exception):
    """Base exception for OpenFDA-related errors."""

    pass


class OpenFDARateLimitError(OpenFDAError):
    """Raised when FDA API rate limit is exceeded."""

    def __init__(self, message: str = "FDA API rate limit exceeded"):
        super().__init__(message)
        self.message = message


class OpenFDAValidationError(OpenFDAError):
    """Raised when FDA response validation fails."""

    def __init__(self, message: str = "Invalid FDA API response"):
        super().__init__(message)
        self.message = message


class OpenFDAConnectionError(OpenFDAError):
    """Raised when connection to FDA API fails."""

    def __init__(self, message: str = "Failed to connect to FDA API"):
        super().__init__(message)
        self.message = message


class OpenFDANotFoundError(OpenFDAError):
    """Raised when requested resource is not found."""

    def __init__(self, resource_type: str, resource_id: str):
        message = f"{resource_type} not found: {resource_id}"
        super().__init__(message)
        self.resource_type = resource_type
        self.resource_id = resource_id
        self.message = message


class OpenFDATimeoutError(OpenFDAError):
    """Raised when FDA API request times out."""

    def __init__(self, message: str = "FDA API request timeout"):
        super().__init__(message)
        self.message = message


class OpenFDAInvalidParameterError(OpenFDAError):
    """Raised when invalid parameters are provided."""

    def __init__(self, parameter: str, value: str, reason: str):
        message = (
            f"Invalid parameter '{parameter}' with value '{value}': {reason}"
        )
        super().__init__(message)
        self.parameter = parameter
        self.value = value
        self.reason = reason
        self.message = message

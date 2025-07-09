"""Logging filter to suppress non-critical ASGI errors."""

import logging


class ASGIErrorFilter(logging.Filter):
    """Filter out non-critical ASGI/Starlette middleware errors."""

    def filter(self, record: logging.LogRecord) -> bool:
        """Return False to suppress the log record, True to allow it."""

        # Check if this is an ASGI error we want to suppress
        if record.levelname == "ERROR":
            message = str(record.getMessage())

            # Suppress known non-critical ASGI errors
            if "Exception in ASGI application" in message:
                return False
            if "AssertionError" in message and "http.response.body" in message:
                return False
            if (
                "unhandled errors in a TaskGroup" in message
                and hasattr(record, "exc_info")
                and record.exc_info
            ):
                exc_type, exc_value, _ = record.exc_info
                if exc_type and "AssertionError" in str(exc_type):
                    return False

        # Allow all other logs
        return True


def setup_logging_filters():
    """Set up logging filters to suppress non-critical errors."""

    # Add filter to uvicorn error logger
    uvicorn_logger = logging.getLogger("uvicorn.error")
    uvicorn_logger.addFilter(ASGIErrorFilter())

    # Add filter to uvicorn access logger
    uvicorn_access_logger = logging.getLogger("uvicorn.access")
    uvicorn_access_logger.addFilter(ASGIErrorFilter())

    # Add filter to starlette logger
    starlette_logger = logging.getLogger("starlette")
    starlette_logger.addFilter(ASGIErrorFilter())

    # Add filter to fastapi logger
    fastapi_logger = logging.getLogger("fastapi")
    fastapi_logger.addFilter(ASGIErrorFilter())

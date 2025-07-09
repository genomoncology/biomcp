"""Core module for BioMCP containing shared resources."""

from contextlib import asynccontextmanager
from enum import Enum
from typing import Any

from mcp.server.fastmcp import FastMCP
from mcp.server.fastmcp.utilities.logging import get_logger

from .logging_filter import setup_logging_filters

# Set up logger first
logger = get_logger(__name__)

# Set up logging filters to suppress non-critical ASGI errors
setup_logging_filters()


# Define a lifespan function for startup tasks
@asynccontextmanager
async def lifespan(mcp):
    """Lifespan context manager for startup/shutdown tasks."""
    # Startup
    try:
        from .prefetch import start_prefetching

        await start_prefetching()
    except Exception as e:
        # Don't fail startup if prefetching fails
        logger.warning(f"Prefetching failed: {e}")

    yield

    # Shutdown (if needed)


# Initialize the MCP app with lifespan
mcp_app = FastMCP(
    name="BioMCP - Biomedical Model Context Protocol Server",
    description="Biomedical research server with integrated sequential thinking. Use search(domain='thinking') to activate systematic step-by-step analysis before making biomedical queries.",
    version="0.1.10",
    lifespan=lifespan,
)


class StrEnum(str, Enum):
    def __str__(self):
        return self.value

    @classmethod
    def _missing_(cls, value):
        if isinstance(value, str):
            for member in cls:
                if member.lower() == value.lower():
                    return member
                m = member.lower().replace(" ", "_")
                v = value.lower().replace(" ", "_")
                if m == v:
                    return member
        return None


class PublicationState(StrEnum):
    """Publication state of an article."""

    PREPRINT = "preprint"
    PEER_REVIEWED = "peer_reviewed"
    UNKNOWN = "unknown"


def ensure_list(value: Any, split_strings: bool = False) -> list[Any]:
    """
    Convert a value to a list if it's not already.

    This is particularly useful for handling inputs from LLMs that might
    provide comma-separated strings instead of proper lists.

    Args:
        value: The value to convert to a list
        split_strings: If True, splits string values by comma and strips whitespace.
                      If False, wraps the string in a list without splitting.

    Returns:
        A list containing the value(s)
        - If value is None, returns an empty list
        - If value is a string and split_strings is True, splits by comma and strips whitespace
        - If value is a string and split_strings is False, wraps it in a list
        - If value is already a list, returns it unchanged
        - For other types, wraps them in a list
    """
    if value is None:
        return []
    if isinstance(value, str) and split_strings:
        # Split by comma and strip whitespace
        return [item.strip() for item in value.split(",")]
    if isinstance(value, list):
        return value
    # For any other type, wrap it in a list
    return [value]


# Set httpx logger to warn level only
httpx_logger = get_logger("httpx")
httpx_logger.setLevel("WARN")

# Set main logger level
logger.setLevel("INFO")

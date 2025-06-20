"""Core module for BioMCP containing shared resources."""

from enum import Enum
from typing import Any

from mcp.server.fastmcp import FastMCP
from mcp.server.fastmcp.utilities.logging import get_logger

# Initialize the MCP app here
mcp_app = FastMCP(
    name="BioMCP - Biomedical Model Context Protocol Server",
    description="Biomedical research server with integrated sequential thinking. Use search(domain='thinking') to activate systematic step-by-step analysis before making biomedical queries.",
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


logger = get_logger("httpx")
logger.setLevel("WARN")

logger = get_logger(__name__)
logger.setLevel("INFO")

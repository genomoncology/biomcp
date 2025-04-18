"""Core module for BioMCP containing shared resources."""

from enum import Enum
from typing import Any

from mcp.server.fastmcp import FastMCP
from mcp.server.fastmcp.utilities.logging import get_logger

# Initialize the MCP app here
mcp_app = FastMCP(name="BioMCP - Biomedical Model Context Protocol Server")


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


def ensure_list(value: Any) -> list[Any]:
    """Convert a single value to a list if it's not already."""
    if not isinstance(value, list):
        return [value] if value is not None else []
    return value


logger = get_logger("httpx")
logger.setLevel("WARN")

logger = get_logger(__name__)
logger.setLevel("INFO")

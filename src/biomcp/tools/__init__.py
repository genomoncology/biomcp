"""Consolidated domain-specific MCP tools for BioMCP."""

# Import consolidated tools to register them with mcp_app
from .fda_tool import fda
from .nci_tool import nci
from .biothings_tools import gene, disease, drug
from .variant_tool import variant
from .alphagenome_tool import alphagenome
from .trial_tool import trial
from .article_tool import article

__all__ = ["fda", "nci", "gene", "disease", "drug", "variant", "alphagenome", "trial", "article"]

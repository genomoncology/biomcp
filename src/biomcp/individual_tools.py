"""Consolidated MCP tools for BioMCP.

This module imports and registers all consolidated domain-specific tools
that provide access to biomedical data sources.

All individual tools have been consolidated into action-based tools:
- OpenFDA: 'fda' tool with domain and action parameters (12 tools → 1)
- NCI: 'nci' tool with resource and action parameters (6 tools → 1)
- BioThings: 'gene', 'disease', 'drug' tools with action parameter (3 tools, action-based)
- Variants: 'variant' tool (search/get) + 'alphagenome' tool (2 tools → 2, kept separate for optional package)
- Trials: 'trial' tool with action parameter (6 tools → 1)
- Articles: 'article' tool with action parameter - search/get (2 tools → 1)

Total reduction: 32 individual tools → 8 consolidated tools
"""

import logging
from typing import Annotated, Literal

from pydantic import Field

from biomcp.core import mcp_app

# Import consolidated tools to register them
from biomcp.tools import fda, nci, gene, disease, drug, variant, alphagenome, trial, article  # noqa: F401

logger = logging.getLogger(__name__)

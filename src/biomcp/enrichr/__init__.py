"""
Enrichr functional enrichment analysis integration.

Inspired by gget enrichr (https://github.com/pachterlab/gget).
Citation: Luebbert & Pachter (2023). Bioinformatics, 39(1), btac836.
BioMCP directly integrates with Enrichr API rather than using gget as a dependency.
"""

from .client import EnrichrClient
from .databases import ENRICHR_DATABASES, get_database_name

__all__ = ["ENRICHR_DATABASES", "EnrichrClient", "get_database_name"]

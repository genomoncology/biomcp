"""Disease information tools for BioMCP."""

from .getter import get_disease
from .search import search_diseases, get_disease_by_id

__all__ = ["get_disease", "get_disease_by_id", "search_diseases"]

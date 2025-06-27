"""Gene symbol validation utilities."""

import re

# Common gene symbol patterns
GENE_SYMBOL_PATTERN = re.compile(r"^[A-Z][A-Z0-9-]*(\.[0-9]+)?$")

# Known problematic or invalid gene symbols
INVALID_GENES = {
    "INVALID",
    "UNKNOWN",
    "NULL",
    "NONE",
    "TEST",
    "INVALID_GENE_XYZ",
}


def is_valid_gene_symbol(gene: str | None) -> bool:
    """Validate if a string is a valid gene symbol.

    Args:
        gene: The gene symbol to validate

    Returns:
        True if the gene symbol appears valid, False otherwise

    Notes:
        - Gene symbols should start with a letter
        - Can contain letters, numbers, and hyphens
        - May have a version suffix (e.g., .1, .2)
        - Should be uppercase
        - Should not be in the invalid genes list
    """
    if not gene:
        return False

    gene = gene.strip()

    # Check length constraints
    if len(gene) < 2 or len(gene) > 20:
        return False

    # Check against known invalid genes
    if gene.upper() in INVALID_GENES:
        return False

    # Check pattern
    return bool(GENE_SYMBOL_PATTERN.match(gene))


def sanitize_gene_symbol(gene: str) -> str:
    """Sanitize a gene symbol for API calls.

    Args:
        gene: The gene symbol to sanitize

    Returns:
        Sanitized gene symbol in uppercase with whitespace stripped
    """
    return gene.strip().upper()

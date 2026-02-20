"""Diacritics normalization for transparent Czech/ASCII search."""

import unicodedata


def strip_diacritics(text: str) -> str:
    """Strip diacritics from text for search comparison.

    Uses NFD normalization to decompose characters, then removes
    combining marks (category 'Mn'). This handles Czech characters
    like é→e, č→c, ř→r, ž→z, etc.

    The original text is preserved in results; this function is
    only used for search matching.

    Args:
        text: Input text (Czech or ASCII).

    Returns:
        Text with diacritics removed, lowercased.
    """
    nfkd = unicodedata.normalize("NFD", text)
    return "".join(c for c in nfkd if unicodedata.category(c) != "Mn").lower()


def normalize_query(query: str) -> str:
    """Normalize a search query for diacritics-insensitive matching.

    Strips diacritics and lowercases. Use this on both the query
    and the indexed text for transparent matching.

    Args:
        query: User search query.

    Returns:
        Normalized query string.
    """
    return strip_diacritics(query.strip())

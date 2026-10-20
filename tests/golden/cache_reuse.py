"""Assert reuse of admitted HTTP answers after best-effort concurrent writes."""
from __future__ import annotations

import json
from pathlib import Path
import sqlite3
from collections.abc import Callable, Iterable
from urllib.parse import parse_qsl, urlencode, urlsplit


def provider_key(url: str, normalize_path: Callable[[str], str] = str) -> str:
    """Match a response URL or fixture path to the replay server's GET identity."""
    parts = urlsplit(url)
    path = urlsplit(normalize_path(parts.path + "?" + parts.query))
    return f"GET {path.path}?{urlencode(sorted(parse_qsl(path.query, keep_blank_values=True)))}"


def admitted_provider_keys(
    cache: Path, *, normalize_path: Callable[[str], str] = str,
    prefixes: tuple[str, ...] = (),
) -> set[str]:
    """Snapshot identities actually stored, without assuming every write landed."""
    with sqlite3.connect(cache / "biomcp.db") as connection:
        urls = [json.loads(row[0])["response"]["url"] for row in
                connection.execute("select metadata from entries where namespace='http'")]
    return {provider_key(url, normalize_path) for url in urls
            if not prefixes or urlsplit(url).path.startswith(prefixes)}


def assert_admitted_reused(
    admitted: set[str], contacted_before: Iterable[str], contacted_after: Iterable[str],
) -> None:
    """Require a genuine admitted answer and reject every repeat of admitted keys."""
    assert admitted and admitted <= set(contacted_before), (
        "reuse proof must name genuinely admitted provider answers", admitted,
    )
    repeated = admitted & set(contacted_after)
    assert not repeated, ("identical search must reuse cached provider answers", repeated)

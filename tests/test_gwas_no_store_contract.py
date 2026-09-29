"""GWAS keeps NoStore unconditionally (ticket 1268).

`BIOMCP_CACHE_MODE=infinite` once replaced GWAS's no-store mark with
force-cache and served stale bodies — the decode-failure class the
bypass exists to prevent. The invariant is structural: GWAS sends
every request through `request_no_store`, sets `CacheMode::NoStore`
there, and never calls `apply_cache_mode`. reqwest hides request
extensions outside its crate, so the pin lives here at the source
shape, like the external-XML guard.
"""

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
GWAS = ROOT / "src" / "sources" / "gwas.rs"


def test_gwas_pins_no_store_on_every_request() -> None:
    text = GWAS.read_text(encoding="utf-8")
    assert "with_extension(CacheMode::NoStore)" in text, (
        "GWAS requests must carry CacheMode::NoStore"
    )
    # Both request entry points route through the no-store builder.
    entries = re.findall(r"self\.request_no_store\(&plan\)", text)
    assert len(entries) >= 2, (
        f"expected both GWAS request paths to use request_no_store, found {len(entries)}"
    )


def test_gwas_never_applies_the_process_cache_mode() -> None:
    text = GWAS.read_text(encoding="utf-8")
    # The reason comment may NAME apply_cache_mode; only a real call
    # is forbidden.
    code = "\n".join(
        line for line in text.splitlines() if not line.lstrip().startswith("//")
    )
    assert "apply_cache_mode" not in code, (
        "GWAS must not call apply_cache_mode — infinite mode once swapped "
        "its no-store mark for force-cache (ticket 1268)"
    )
    assert "Always bypass persistence" in text, (
        "the decode-failure reason for the bypass must stay recorded"
    )

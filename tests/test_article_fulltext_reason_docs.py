from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def test_every_fulltext_attempt_reason_has_a_documented_meaning() -> None:
    source = (ROOT / "src/entities/article/mod.rs").read_text(encoding="utf-8")
    enum = re.search(r"pub enum ArticleFulltextAttemptReason \{([^}]+)\}", source)
    assert enum, "full-text reason enum must remain discoverable"
    variants = re.findall(r"^    ([A-Z][A-Za-z0-9]+),$", enum[1], re.MULTILINE)
    assert variants, "full-text reason enum must not be empty"
    reasons = {re.sub(r"(?<!^)(?=[A-Z])", "_", name).lower() for name in variants}
    guide = (ROOT / "docs/user-guide/article.md").read_text(encoding="utf-8")
    section = guide.split("### Full-text attempt reasons\n", 1)[1].split(
        "\nIndexing section:", 1
    )[0]
    documented = dict(re.findall(r"^\| `([a-z_]+)` \| ([^|]+) \|$", section, re.MULTILINE))
    assert set(documented) == reasons, "document every current reason and remove stale reasons"
    assert all(meaning.strip().endswith(".") for meaning in documented.values())

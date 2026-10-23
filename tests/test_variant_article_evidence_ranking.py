"""Variant article pages lead with captured variant evidence."""

import json
import os
import subprocess

import pytest

from hang_guard import HANG_TIMEOUT
from variant_article_evidence_replay import REPO_ROOT, akt1_replay

pytestmark = pytest.mark.needs_binary


def test_variant_article_recorded_evidence_reaches_first_page(tmp_path):
    with akt1_replay(tmp_path, cache=False) as replay:
        result = subprocess.run(
            [os.environ.get("BIOMCP_BIN", str(REPO_ROOT / "target/debug/biomcp")),
             "--json", "--no-cache", "variant", "articles", "AKT1 E17K", "--limit", "50"],
            env=replay.env, capture_output=True, text=True, timeout=HANG_TIMEOUT,
        )
        assert result.returncode == 0, result.stdout + result.stderr
        payload = json.loads(result.stdout)
        ids = [row["pmid"] for row in payload["results"]]
        assert {"33123537", "17611497"} <= set(ids), ids
        # The recorded PubMed comparison reports nine variant-evidence papers.
        # Require literal gene+supported-variant text here, without importing
        # private clinical classifications into a public fixture.
        recorded = {row["pmid"]: row for row in replay.capture["rows"]}
        def has_evidence(pmid):
            row = recorded[pmid]
            text = (row["title"] + " " + row.get("abstract", "")).lower()
            return "akt1" in text and ("e17k" in text or "glu17lys" in text)
        assert sum(has_evidence(pmid) for pmid in ids[:10]) >= 9
        assert all(row.get("identity", {}).get("status") != "confirmed"
                   for row in payload["results"])

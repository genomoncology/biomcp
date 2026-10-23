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


def test_variant_article_lexical_evidence_precedes_source_agreement(tmp_path):
    # Native provider order favors generic gene papers. The caller should see
    # title evidence first, then abstract evidence, then gene-only papers.
    rows = [
        {"pmid": "81001", "title": "AKT1 signaling", "abstract": "AKT1 E17K response"},
        {"pmid": "81002", "title": "AKT1 E17K response", "abstract": ""},
        {"pmid": "81003", "title": "AKT1 signaling", "abstract": "Gene pathways"},
        {"pmid": "81004", "title": "AKT1 Glu17Lys response", "abstract": "AKT1 E17K response"},
        {"pmid": "81005", "title": "AKT1 E17KX unrelated spelling", "abstract": ""},
    ]
    for index, row in enumerate(rows):
        row["provenance"] = [{"route": "strict", "source": "europepmc",
                              "native_position": index}]
    rows[0]["provenance"].append({"route": "strict", "source": "pubmed", "native_position": 0})
    with akt1_replay(tmp_path, cache=False,
                     capture={"pubmed_order": [], "rows": rows}) as replay:
        result = subprocess.run(
            [os.environ.get("BIOMCP_BIN", str(REPO_ROOT / "target/debug/biomcp")),
             "--json", "--no-cache", "variant", "articles", "AKT1 E17K", "--limit", "5", "--debug-plan"],
            env=replay.env, capture_output=True, text=True, timeout=HANG_TIMEOUT,
        )
        assert result.returncode == 0, result.stdout + result.stderr
        payload = json.loads(result.stdout)
        ranking = payload["debug_plan"]["ranking"]
        assert ranking["version"] == 1
        assert ranking["method"].startswith("descending lexicographic variant evidence")
        assert len(ranking["inputs"]) == 6
        assert ranking["inputs"][0] == "title_gene_and_supported_variant"
        assert ranking["inputs"][-1] == "stable_identifier"
        assert [row["pmid"] for row in payload["results"]] == [
            "81004", "81002", "81001", "81003", "81005"]
        assert all(row.get("identity", {}).get("status") != "confirmed"
                   for row in payload["results"])

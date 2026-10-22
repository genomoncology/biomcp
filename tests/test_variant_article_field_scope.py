"""Variant article discovery: restricted fields cannot cover full indexed text."""

import json

import pytest

from test_variant_article_recovery import REPO_ROOT, cases, execute_case

pytestmark = [pytest.mark.needs_binary]


def test_variant_articles_retains_all_field_compatible_allele_candidates(tmp_path):
    payload, requests = execute_case({**cases[0], "mode": "field-scope"}, tmp_path)
    rows = {row["pmid"]: row for row in payload["results"]}
    assert {"10486320", "31788995"} <= rows.keys()
    for pmid in ("10486320", "31788995"):
        assert rows[pmid].get("identity", {}).get("status") != "confirmed"
    queries = [params["query"][0] for path, params in requests
               if path == "/search" and "query" in params]
    assert "TITLE_ABS:" in queries[0]
    assert "TITLE_ABS:" not in queries[1]
    planned = [plan for plan in payload["debug_plan"]["provider_queries"]
               if plan["provider"] == "europepmc" and plan["route"] == "exact_lexical"]
    assert [plan["query_template_version"] for plan in planned] == [
        "europepmc-supplied-protein-all-indexed-fields-v1",
        "europepmc-all-indexed-fields-v1",
    ]
    fixture = REPO_ROOT / "tests/fixtures/article/field-scope"
    union = json.loads((fixture / "generated-union.json").read_text())
    union_ids = {row.get("pmid") for row in union["resultList"]["result"]}
    assert "10486320" not in union_ids
    assert "31788995" in union_ids
    assert len(union["resultList"]["result"]) == 25 < union["hitCount"]
    assert payload["truncated"] and not payload["complete"]
    all_fields = [query for query in queries if "TITLE_ABS:" not in query
                  and "EXT_ID:" not in query]
    assert any("Pro1749Arg" in query for query in all_fields)
    assert any("5246C>G" in query for query in all_fields)
    # Every discovery request still carries the supplied gene and an allele.
    assert all("BRCA1" in query and any(change in query for change in
               ("P1749R", "Pro1749Arg", "5246C>G", "rs80357462", "41209100",
                "1702", "1770", "240", "59", "607", "645", "176C", "1820C",
                "1934C", "5105C", "5309C", "719C")) for query in all_fields)

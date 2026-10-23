"""Variant feature: independent provider work leaves the first page attempted."""

import copy
import json

import pytest

from test_variant_article_recovery import cases, execute_case
from golden.replay_env import REPO_ROOT

pytestmark = [pytest.mark.needs_binary]


def first_page_case(name):
    control = copy.deepcopy(cases[0])
    control["mode"] = "first-page"
    gene, protein = name.split()
    control["requested"] = {"gene": gene, "protein_change": protein}
    control["argv"] = ["--json", "--no-cache", "variant", "articles", name,
                       "--limit", "50", "--verify-identity", "--debug-plan"]
    if gene == "ABCC8":
        hit = json.loads((REPO_ROOT / "tests/fixtures/article/coding-aliases/authored.json").read_text())["hit"]
        control["source_protein_forms"] = hit["dbnsfp"]["hgvsp"]
        control["resolution"]["normalized_aliases"] = {
            "protein_changes": hit["dbnsfp"]["hgvsp"],
            "coding_changes": hit["dbnsfp"]["hgvsc"],
            "genomic_ids": [hit["_id"]], "rsids": [],
        }
        control["gene"] = {"entrez_id": "6833", "recorded_aliases": ["SUR1", "HHF1"]}
    elif gene == "CDKN2A":
        # Literal forms from the recorded command output; replies remain authored.
        control["source_protein_forms"] = ["A57V", "A6V", "p.Ala57Val", "p.Ala6Val"]
        control["resolution"]["normalized_aliases"] = {
            "protein_changes": ["A57V", "A6V"],
            "coding_changes": ["c.170C>T", "c.17C>T"],
            "genomic_ids": ["chr9:g.21971188G>A"], "rsids": ["rs372266620"],
        }
        control["gene"] = {"entrez_id": "1029", "recorded_aliases": ["P16", "INK4A"]}
    return control


@pytest.mark.parametrize("strategy", ["union", "lexical"])
@pytest.mark.parametrize("name", ["ABCC8 R1436Q", "ABCC8 R1437Q", "CDKN2A A57V", "BRCA1 P1749R"])
def test_variant_first_page_attempts_independent_provider_work(name, strategy, tmp_path):
    control = first_page_case(name)
    control["argv"] += ["--strategy", strategy]
    overlap = []
    result, _ = execute_case(control, tmp_path, overlap=overlap)
    assert overlap == [True], "strict provider work waited for an independent query"
    strict_s2 = next(row for row in result["source_status"]
                     if row["route"] == "strict" and row["source"] == "semanticscholar")
    assert not any(strict_s2["work"][kind] for kind in
                   ("degraded", "unavailable", "timed_out")), strict_s2
    assert len(result["results"]) == 50
    assert not [row for row in result["source_status"] if row["work"]["not_attempted"]]
    assert not [row for row in result["source_status"] if "invocation_deadline" in row["reason_codes"]]
    assert all(row["work"]["planned"] == sum(row["work"][kind] for kind in
               ("ok", "degraded", "unavailable", "timed_out")) for row in result["source_status"])

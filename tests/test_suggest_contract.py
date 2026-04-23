from __future__ import annotations

import json
import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
RELEASE_BIN = REPO_ROOT / "target" / "release" / "biomcp"


def _run_text(*args: str) -> str:
    assert RELEASE_BIN.exists(), f"missing release binary: {RELEASE_BIN}"
    result = subprocess.run(
        [str(RELEASE_BIN), *args],
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout


def _run_json(*args: str) -> dict[str, object]:
    return json.loads(_run_text("--json", *args))


def test_suggest_markdown_routes_treatment_question() -> None:
    out = _run_text("suggest", "What drugs treat melanoma?")

    assert "# BioMCP Suggestion" in out
    assert "matched_skill: `treatment-lookup`" in out
    assert "biomcp search drug --indication melanoma --limit 5" in out
    assert "biomcp search article -d melanoma --type review --limit 5" in out
    assert "biomcp skill treatment-lookup" in out


def test_suggest_json_routes_ticket_examples() -> None:
    variant = _run_json("suggest", "Is variant rs113488022 pathogenic in melanoma?")
    assert set(variant) == {"matched_skill", "summary", "first_commands", "full_skill"}
    assert variant["matched_skill"] == "variant-pathogenicity"
    assert variant["first_commands"] == [
        "biomcp get variant rs113488022 clinvar predictions population",
        "biomcp get variant rs113488022 civic cgi",
    ]
    assert variant["full_skill"] == "biomcp skill variant-pathogenicity"

    regulatory = _run_json("suggest", "When was imatinib approved?")
    assert regulatory["matched_skill"] == "drug-regulatory"
    assert regulatory["first_commands"] == [
        "biomcp get drug imatinib regulatory",
        "biomcp get drug imatinib approvals",
    ]

    regional = _run_json("suggest", "When was imatinib approved by FDA?")
    assert regional["first_commands"][0] == "biomcp get drug imatinib regulatory --region us"


def test_suggest_resistance_to_drug_prefers_drug_anchor() -> None:
    out = _run_text("suggest", "What is the mechanism of resistance to imatinib?")
    assert "matched_skill: `mechanism-pathway`" in out
    assert "biomcp search drug imatinib --limit 5" in out
    assert "biomcp get drug imatinib targets regulatory" in out

    response = _run_json("suggest", "What is the mechanism of resistance to imatinib?")
    assert set(response) == {"matched_skill", "summary", "first_commands", "full_skill"}
    assert response["matched_skill"] == "mechanism-pathway"
    assert response["first_commands"] == [
        "biomcp search drug imatinib --limit 5",
        "biomcp get drug imatinib targets regulatory",
    ]
    assert response["full_skill"] == "biomcp skill mechanism-pathway"


def test_suggest_resistance_against_and_develop_phrasings_use_drug_anchor() -> None:
    against = _run_json("suggest", "What is the mechanism of resistance against imatinib?")
    assert against["matched_skill"] == "mechanism-pathway"
    assert against["first_commands"] == [
        "biomcp search drug imatinib --limit 5",
        "biomcp get drug imatinib targets regulatory",
    ]

    develop = _run_json("suggest", "How does imatinib resistance develop?")
    assert develop["matched_skill"] == "mechanism-pathway"
    assert develop["first_commands"] == [
        "biomcp search drug imatinib --limit 5",
        "biomcp get drug imatinib targets regulatory",
    ]


def test_suggest_quotes_shell_sensitive_anchor_on_resistance_to_drug() -> None:
    out = _run_text("suggest", "What is the mechanism of resistance to imatinib; rm -rf /?")
    assert 'biomcp search drug "imatinib; rm -rf /" --limit 5' in out
    assert 'biomcp get drug "imatinib; rm -rf /" targets regulatory' in out


def test_suggest_more_question_shapes_and_no_match() -> None:
    cases = {
        "What pharmacogenes affect warfarin dosing?": "pharmacogene-cumulative",
        "Are there recruiting trials for melanoma?": "trial-recruitment",
        "How do I distinguish Goldberg-Shprintzen syndrome vs Shprintzen-Goldberg syndrome?": "syndrome-disambiguation",
        "Is Borna disease virus linked to brain tumor?": "negative-evidence",
    }
    for question, slug in cases.items():
        assert _run_json("suggest", question)["matched_skill"] == slug

    intervention = _run_json("suggest", "Are there recruiting trials with imatinib?")
    assert intervention["first_commands"] == [
        "biomcp search trial -i imatinib --status recruiting --limit 5",
        "biomcp search article --drug imatinib --type review --limit 5",
    ]

    symptom = _run_json("suggest", "symptoms include seizure and developmental delay")
    assert symptom["first_commands"] == [
        'biomcp discover "seizure and developmental delay"',
        'biomcp search phenotype "seizure and developmental delay" --limit 5',
    ]

    quoted = _run_text("suggest", "What drugs treat lung cancer; rm -rf /?")
    assert 'biomcp search drug --indication "lung cancer; rm -rf /" --limit 5' in quoted

    no_match = _run_json("suggest", "What is x?")
    assert no_match == {
        "matched_skill": None,
        "summary": "No confident BioMCP skill match.",
        "first_commands": [],
        "full_skill": None,
    }

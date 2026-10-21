"""Replay drug approval identity, aggregate status and label dates at the CLI."""
from __future__ import annotations

import json
import os
import subprocess
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlparse

import pytest
from golden.replay_env import isolated_command_env

pytestmark = [pytest.mark.needs_binary]


@pytest.fixture
def replay(tmp_path: Path):
    state = {"name": "daratumumab", "brands": [], "mechanisms": None, "orphan_healthy": False, "label_extra": False, "related_label": False, "approvals_unavailable": False}

    class Handler(BaseHTTPRequestHandler):
        def answer(self, payload, status=200):
            body = payload.encode() if isinstance(payload, str) else json.dumps(payload).encode()
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def do_GET(self):  # noqa: N802
            path = urlparse(self.path)
            query = parse_qs(path.query).get("search", [""])[0]
            name = state["name"]
            if "drugsfda" in path.path:
                state["approval_requests"] = state.get("approval_requests", 0) + 1
                if state["approvals_unavailable"]:
                    self.answer({}, 503)
                    return
                rows = []
                if name == "pembrolizumab":
                    fixture = Path(__file__).resolve().parents[1] / "testdata/sources/openfda/drugsfda_keytruda_20261020.json"
                    self.answer(json.loads(fixture.read_text()))
                    return
                if name == "daratumumab" and "products.active_ingredients.name" in query:
                    rows = [{"application_number": app, "products": [{"active_ingredients": [{"name": ingredient}]}]}
                            for app, ingredient in [("BLA761036", name), ("BLA761145", name),
                                                    ("BLA999999", "daratumumab derivative")]]
                if name == "belantamab mafodotin" and "BLENREP" in query.upper():
                    rows = [{"application_number": "BLA761440", "products": [{"brand_name": "BLENREP",
                             "active_ingredients": [{"name": "BELANTAMAB MAFODOTIN-BLMF"}]}]},
                            {"application_number": "BLA999999", "products": [{"brand_name": "BLENREP",
                             "active_ingredients": [{"name": "unrelated"}]}]}]
                if name == "daratumumab" and rows:
                    rows[1]["products"][0]["active_ingredients"].append({"name": "HYALURONIDASE-FIHJ"})
                    rows.append({"application_number": "BLA999998", "products": [{"active_ingredients": [{"name": "DARATUMUMAB-ABCD"}]}]})
                    if state["related_label"]:
                        rows[-1]["products"][0]["brand_name"] = "RELATED"
                        rows.append({"application_number": "BLA999995", "products": [{"brand_name": "RELATED"}]})
                if name == "belantamab mafodotin" and rows:
                    rows.extend([{"application_number": "BLA999997", "products": [{"brand_name": "BLENREP",
                        "active_ingredients": [{"name": "BELANTAMAB MAFODOTIN-ABCD"}]}]},
                        {"application_number": "BLA999996", "products": [{"brand_name": "BLENREP",
                        "active_ingredients": [{"name": "BELANTAMAB MAFODOTIN-BLMF"}, {"name": "unrelated"}]}]}])
                if name == "daratumumab and hyaluronidase-fihj" and "DARZALEX FASPRO" in query.upper():
                    product = {"brand_name": "DARZALEX FASPRO", "active_ingredients": [
                        {"name": "DARATUMUMAB"}, {"name": "HYALURONIDASE-FIHJ"}]}
                    rows = [{"application_number": "BLA761145", "products": [product,
                        {"brand_name": "unrelated", "active_ingredients": [{"name": "unrelated"}]}]},
                        {"application_number": "BLA999994", "products": [{**product, "active_ingredients": [
                            {"name": "DARATUMUMAB"}, {"name": "HYALURONIDASE"}]}]},
                        {"application_number": "BLA999993", "products": [{**product, "active_ingredients": [
                            {"name": "DARATUMUMAB"}, {"name": "HYALURONIDASE-FIHJ"}, {"name": "unrelated"}]}]},
                        {"application_number": "BLA999992", "products": [{**product, "brand_name": "other brand"}]},
                        {"application_number": "BLA999991", "products": [{**product, "active_ingredients": [
                            {"name": "DARATUMUMAB"}, {}]}]}]
                self.answer({"meta": {"results": {"skip": 0, "limit": 8, "total": len(rows)}}, "results": rows})
            elif "label" in path.path:
                ingredient = "BELANTAMAB MAFODOTIN-BLMF" if name == "belantamab mafodotin" else name
                if state["related_label"]:
                    ingredient = "DARATUMUMAB-ABCD"
                substances = ["DARATUMUMAB", "HYALURONIDASE-FIHJ"] if name == "daratumumab and hyaluronidase-fihj" else [ingredient]
                if state["label_extra"]:
                    substances.append("unrelated")
                self.answer({"results": [{"set_id": "label-fixture", "effective_time": "20180429",
                    "openfda": {"generic_name": [name if name == "belantamab mafodotin" else ingredient], "substance_name": substances,
                                "brand_name": ["RELATED"] if state["related_label"] else state["brands"]}, "warnings": ["Recorded label warning."]}]})
            elif "shortages" in path.path:
                self.answer({"meta": {"results": {"skip": 0, "limit": 5, "total": 0}}, "results": []})
            elif "event" in path.path:
                self.answer({"results": []})
            elif "mechanism" in path.path:
                self.answer({"mechanisms": []})
            elif "query" in path.path:
                self.answer({"total": 1, "hits": [{"_id": "fixture", "drugbank": {"name": name},
                    "chembl": {"pref_name": name, "molecule_chembl_id": "CHEMBL1"}}]})
            else:
                self.answer({}, 503)

        def do_POST(self):  # noqa: N802
            self.rfile.read(int(self.headers.get("Content-Length", "0")))
            if "orphan" in self.path:
                if state["orphan_healthy"]:
                    fixture = Path(__file__).resolve().parents[1] / "testdata/sources/fda_orphan/provider-shaped.html"
                    self.answer(fixture.read_text())
                else:
                    self.answer({}, 503)
            elif "civic" in self.path:
                self.answer({"data": {"therapies": {"nodes": []}}})
            else:
                self.answer({"data": {"drug": {"indications": {"rows": []},
                    "mechanismsOfAction": state["mechanisms"]}}})

        def log_message(self, format, *args):
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    base = f"http://127.0.0.1:{server.server_port}"
    env = isolated_command_env(tmp_path, {f"BIOMCP_{source}_BASE": base + suffix for source, suffix in
        [("MYCHEM", ""), ("OPENFDA", ""), ("CHEMBL", ""), ("OPENTARGETS", ""),
         ("CIVIC", "/civic"), ("FDA_ORPHAN", "/orphan")]}, base)
    env["RUST_LOG"] = "warn"
    binary = os.environ["BIOMCP_BIN"]

    def run(section, markdown=False):
        sections = [section] if isinstance(section, str) else section
        result = subprocess.run([binary, *([] if markdown else ["--json"]), "get", "drug",
                                 "--name", state["name"], *sections,
                                 *(["--region", "us"] if len(sections) == 1 and sections[0] in {"regulatory", "safety"} else [])], env=env, text=True,
                                capture_output=True, timeout=30, check=False)
        assert result.returncode == 0, result.stdout + result.stderr
        return result if markdown else (json.loads(result.stdout), result.stderr)

    try:
        yield state, run
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)


@pytest.mark.parametrize("name,brands,expected", [
    ("daratumumab", [], {"BLA761036"}),
    ("belantamab mafodotin", ["BLENREP"], {"BLA761440"}),
    ("daratumumab and hyaluronidase-fihj", ["DARZALEX FASPRO"], {"BLA761145"}),
])
def test_approvals_find_ingredient_and_brand_records_without_other_drugs(replay, name, brands, expected):
    state, run = replay
    state.update(name=name, brands=brands)
    data, _ = run("approvals")
    assert {row["application_number"] for row in data["approvals"]} == expected
    if name == "daratumumab and hyaluronidase-fihj":
        products = data["approvals"][0]["products"]
        assert len(products) == 1
        assert products[0]["brand_name"] == "DARZALEX FASPRO"
        assert products[0]["active_ingredients"] == ["DARATUMUMAB", "HYALURONIDASE-FIHJ"]
        state["label_extra"] = True
        contradictory, _ = run("approvals")
        assert contradictory["approvals"] == []


def test_related_label_cannot_establish_approval_identity_by_suffix(replay):
    state, run = replay
    state["related_label"] = True
    data, _ = run("approvals")
    assert {row["application_number"] for row in data["approvals"]} == {"BLA761036"}


@pytest.mark.parametrize("name,healthy,expected,sources", [
    ("zotiraciclib", False, "unavailable", []),
    ("daratumumab", False, "degraded", ["OpenFDA Drugs@FDA"]),
    ("zotiraciclib", True, "empty", ["OpenFDA Drugs@FDA", "FDA Orphan Drug Designations and Approvals"]),
])
def test_regulatory_rolls_up_data_empty_and_unavailable_parts(replay, name, healthy, expected, sources):
    state, run = replay
    state.update(name=name, orphan_healthy=healthy)
    data, _ = run("regulatory")
    regulatory = next(row for row in data["_meta"]["section_sources"] if row["key"] == "regulatory")
    assert regulatory["outcome"] == expected
    assert regulatory["sources"] == sources


def test_safety_displays_the_source_label_effective_date(replay):
    state, run = replay
    state["name"] = "lorcaserin"
    data, _ = run("safety")
    assert data["us_label_effective_date"] == "2018-04-29"
    markdown = run("safety", markdown=True).stdout
    assert "Label effective date: 2018-04-29" in markdown
    assert "withdraw" not in markdown.lower()


def test_null_mechanisms_are_healthy_empty_but_malformed_mechanisms_are_unavailable(replay):
    state, run = replay
    state["name"] = "trichostatin A"
    data, stderr = run("targets")
    assert not data.get("targets")
    assert data["section_outcomes"]["targets"]["outcome"] == "empty"
    assert "degrading response" not in stderr
    state["mechanisms"] = {"rows": "invalid"}
    malformed, _ = run("targets")
    assert malformed["section_outcomes"]["targets"]["outcome"] == "unavailable"


@pytest.mark.parametrize("brands,unverified,unavailable", [
    ([], False, False),
    (["BLENREP"], True, False),
    ([], False, True),
])
def test_empty_approvals_explain_unverified_identity_without_ruling_out_approval(
    replay, brands, unverified, unavailable,
):
    state, run = replay
    state.update(name="belantamab mafodotin", brands=brands, label_extra=unverified,
                 approvals_unavailable=unavailable)
    for section in ["approvals", "approval-history", "regulatory"]:
        data, _ = run(section)
        assert data["approvals"] == []
        outcome = data["section_outcomes"]["approval-history" if section == "approval-history" else "approvals"]
        markdown = run(section, markdown=True).stdout
        if unavailable:
            assert outcome["outcome"] == "unavailable"
            assert "unavailable" in outcome["message"]
            assert "no exact-name match" not in markdown.lower()
        else:
            assert outcome["outcome"] == "empty"
            message = outcome["message"]
            assert "no exact-name match" in message.lower()
            assert "belantamab mafodotin-blmf" in message
            assert "source-confirmed identity" in message
            assert "FDA approvals are not ruled out" in message
            assert message in markdown
        assert "No approvals found" not in markdown


def test_approval_history_preserves_full_provider_submissions_and_documents(replay):
    state, run = replay
    state.update(name="pembrolizumab", brands=["KEYTRUDA"])
    compact, _ = run("approvals")
    assert len(compact["approvals"][0]["submissions"]) == 8
    assert all("submission_class_code" not in row for row in compact["approvals"][0]["submissions"])
    history, _ = run("approval-history")
    app = history["approvals"][0]
    assert app["application_number"] == "BLA125514"
    fixture = Path(__file__).resolve().parents[1] / "testdata/sources/openfda/drugsfda_keytruda_20261020.json"
    expected = json.loads(fixture.read_text())["results"][0]["submissions"]
    assert len(app["submissions"]) == len(expected) == 127
    by_number = {(row["submission_type"], row["submission_number"]): row for row in app["submissions"]}
    for source in expected:
        row = by_number[(source["submission_type"], source["submission_number"])]
        assert row.get("submission_class_code") == source.get("submission_class_code")
        assert row.get("submission_class_code_description") == source.get("submission_class_code_description")
        date = source.get("submission_status_date")
        assert row.get("status_date") == (f"{date[:4]}-{date[4:6]}-{date[6:]}" if date else None)
        assert row.get("application_docs", []) == source.get("application_docs", [])
    assert history["section_outcomes"]["approval-history"]["outcome"] == "data"
    markdown = run("approval-history", markdown=True).stdout
    assert "EFFICACY" in markdown
    assert "125514s128lbl.pdf" in markdown
    assert "125514Orig1s128ltr.pdf" in markdown
    for section in ["regulatory", "all"]:
        data, _ = run(section)
        assert data["section_outcomes"]["approval-history"]["outcome"] == "not_requested"
        assert len(data["approvals"][0]["submissions"]) == 8


@pytest.mark.parametrize("section", ["approvals", "regulatory", "all"])
def test_combined_approval_views_share_one_provider_result(replay, section):
    state, run = replay
    state.update(name="pembrolizumab", brands=["KEYTRUDA"])
    data, _ = run([section, "approval-history"])
    assert state["approval_requests"] == 1
    assert len(data["approvals"][0]["submissions"]) == 127
    assert data["section_outcomes"]["approvals"] == data["section_outcomes"]["approval-history"]

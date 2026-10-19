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
    state = {"name": "daratumumab", "brands": [], "mechanisms": None, "orphan_healthy": False}

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
                rows = []
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
                if name == "belantamab mafodotin" and rows:
                    rows.extend([{"application_number": "BLA999997", "products": [{"brand_name": "BLENREP",
                        "active_ingredients": [{"name": "BELANTAMAB MAFODOTIN-ABCD"}]}]},
                        {"application_number": "BLA999996", "products": [{"brand_name": "BLENREP",
                        "active_ingredients": [{"name": "BELANTAMAB MAFODOTIN-BLMF"}, {"name": "unrelated"}]}]}])
                self.answer({"meta": {"results": {"skip": 0, "limit": 8, "total": len(rows)}}, "results": rows})
            elif "label" in path.path:
                ingredient = "BELANTAMAB MAFODOTIN-BLMF" if name == "belantamab mafodotin" else name
                self.answer({"results": [{"set_id": "label-fixture", "effective_time": "20180429",
                    "openfda": {"generic_name": [ingredient], "substance_name": [ingredient],
                                "brand_name": state["brands"]}, "warnings": ["Recorded label warning."]}]})
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
        result = subprocess.run([binary, *([] if markdown else ["--json"]), "get", "drug",
                                 "--name", state["name"], section,
                                 *(["--region", "us"] if section in {"regulatory", "safety"} else [])], env=env, text=True,
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
])
def test_approvals_find_ingredient_and_brand_records_without_other_drugs(replay, name, brands, expected):
    state, run = replay
    state.update(name=name, brands=brands)
    data, _ = run("approvals")
    assert {row["application_number"] for row in data["approvals"]} == expected


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

"""Drug targets retain source organisms and only suggest human genes."""
from __future__ import annotations

import json
import os
import subprocess
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

import pytest

from golden.replay_env import isolated_command_env

pytestmark = [pytest.mark.needs_binary]
ROOT = Path(__file__).resolve().parents[1]


def test_drug_targets_label_species_and_only_suggest_human_genes(tmp_path: Path) -> None:
    # Official MyChem response captured 2026-10-19. Reverse rows to expose sorting;
    # remove Human's species in a second replay to exercise missing metadata.
    captured = json.loads((ROOT / "tests/fixtures/drug/bortezomib-targets.json").read_text())
    hit = captured["hits"][0]
    hit["gtopdb"]["interaction_targets"].reverse()

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802
            if "/mechanism" in self.path:
                payload = {"mechanisms": [{"target_pref_name": "psmb5"}]}
            else:
                payload = captured if "/query" in self.path else hit
            body = json.dumps(payload).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, format: str, *args: object) -> None:
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    base = f"http://127.0.0.1:{server.server_port}"
    env = isolated_command_env(tmp_path, {"BIOMCP_MYCHEM_BASE": base}, base)
    env["BIOMCP_DISABLE_SOURCES"] = "chembl,opentargets,civic,openfda"
    binary = os.environ.get("BIOMCP_BIN", str(ROOT / "target/debug/biomcp"))

    def run(json_output: bool) -> str:
        result = subprocess.run(
            [binary, *(["--json"] if json_output else []), "get", "drug", "bortezomib", "targets"],
            env=env, text=True, capture_output=True, timeout=30, check=False,
        )
        assert result.returncode == 0, result.stderr + result.stdout
        return result.stdout

    try:
        result = json.loads(run(True))
        assert result["targets"] == ["PSMB5", "PF3D7_1011400"]
        organisms = result["target_organisms"]
        assert organisms[0]["target"] == "PSMB5"
        assert {(row["target"], row["organism"], row["source"]) for row in organisms} == {
            ("PSMB5", "Human", "Guide to PHARMACOLOGY"),
            ("PF3D7_1011400", "Plasmodium falciparum 3D7", "Guide to PHARMACOLOGY"),
        }
        markdown = run(False)
        assert "PSMB5 (Human; Guide to PHARMACOLOGY)" in markdown
        assert "PF3D7_1011400 (Plasmodium falciparum 3D7; Guide to PHARMACOLOGY)" in markdown
        assert "biomcp get gene PSMB5" in markdown
        assert "biomcp get gene PF3D7_1011400" not in markdown
        for row in hit["gtopdb"]["interaction_targets"]:
            row.pop("species", None)
        unknown = run(False)
        assert "PSMB5 (organism not supplied; Guide to PHARMACOLOGY)" in unknown
        assert "biomcp get gene" not in unknown
        # A synthetic secondary-provider spelling exercises case-folded merging;
        # it supplies no organism or biological mapping.
        hit["chembl"] = {"molecule_chembl_id": "CHEMBL1"}
        env["BIOMCP_CHEMBL_BASE"] = base
        env["BIOMCP_DISABLE_SOURCES"] = "opentargets,civic,openfda"
        merged = json.loads(run(True))
        assert merged["targets"].count("PSMB5") == 1
        assert "psmb5" not in merged["targets"]
        assert {"target": "psmb5", "organism": None, "source": "ChEMBL"} in merged["target_organisms"]
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)

"""Authored controls require paired versioned projections on one source allele."""
from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import sys
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlsplit

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent / "golden"))
from replay_env import REPO_ROOT, isolated_command_env  # noqa: E402

pytestmark = pytest.mark.needs_binary


@pytest.mark.parametrize("gene,change,target,requested,allele,length,residue,clinvar", [
    ("TP53", "R116Q", ("NM_000546.5", "c.743G>A", "p.Arg248Gln"),
     ("NM_001126115.1", "c.347G>A", "p.Arg116Gln"), "chr17:g.7577538C>T", 393, "S", True),
    ("BRCA1", "Q1878R", ("NM_007294.3", "c.5570A>G", "p.Gln1857Arg"),
     ("NM_007300.3", "c.5633A>G", "p.Gln1878Arg"), "chr17:g.41197717T>C", 1863, "A", False),
])
@pytest.mark.parametrize("projection", ["paired", "canonical", "merged_tokens_only", "unversioned", "no_coding",
                                       "deletion_tokens_only", "deletion_unversioned_headline",
                                       "unpaired_mane_row", "conflicting_rows",
                                       "conflicting_rows_reversed"])
def test_other_transcript_acceptance_requires_versioned_same_allele_projection(
    tmp_path, gene, change, target, requested, allele, length, residue, clinvar, projection,
):
    # Transcript spellings reproduce the clinically reviewed recorded shapes.
    # Deletions and synthetic protein sequences isolate the acceptance guard.
    def annotation(row):
        return dict(zip(("feature_id", "hgvs_c", "hgvs_p"), row, strict=True))

    if projection in {"deletion_tokens_only", "deletion_unversioned_headline"}:
        change = change[:-1] + "_" + change[0] + str(int(change[1:-1]) + 1) + "del"
        protein = target[2][2:-3]
        residue_name, position = protein[:3], int(protein[3:])
        target = (*target[:2], f"p.{protein}_{residue_name}{position + 1}del")
    rows = [annotation(target)]
    if projection == "unpaired_mane_row":
        rows.append({"feature_id": target[0].split(".")[0], "hgvs_p": requested[2]})
    if projection not in {"merged_tokens_only", "deletion_tokens_only", "unpaired_mane_row"}:
        row = annotation(requested)
        if projection == "unversioned":
            row["feature_id"] = requested[0].split(".")[0]
        if projection == "no_coding":
            row.pop("hgvs_c")
        if projection in {"conflicting_rows", "conflicting_rows_reversed"}:
            row["feature_id"] = target[0]
        rows.append(row)
    if projection == "conflicting_rows_reversed":
        rows.reverse()
    if projection == "canonical":
        change = target[2]
    if projection == "deletion_unversioned_headline":
        rows = [{"feature_id": requested[0].split(".")[0], "hgvs_c": requested[1],
                 "hgvs_p": "p." + change}]
    hit = {"_id": allele, "dbnsfp": {"genename": gene, "hgvsp": [f"p.{change}", target[2]]},
           "snpeff": {"ann": rows}}
    mane = "NM_000546.6" if gene == "TP53" else "NM_007294.4"
    if clinvar:
        hit["clinvar"] = {"variant_id": 12356, "rcv": [{"accession": "RCV12356",
            "preferred_name": f"{mane}({gene}):{target[1]} ({target[2]})"}]}
    unexpected = []

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self):  # noqa: N802
            path = urlsplit(self.path).path
            if path == "/variant/query":
                value = {"hits": [hit], "total": 1}
            elif path == "/gene/query":
                value = {"total": 1, "hits": [{"_id": "7157", "symbol": gene, "uniprot": {"Swiss-Prot": "P04637"}}]}
            elif path == "/uniprot/uniprotkb/P04637.json":
                value = {"primaryAccession": "P04637", "sequence": {"value": residue * length},
                         "uniProtKBCrossReferences": [{"database": "MANE-Select", "id": "control",
                            "properties": [{"key": "RefSeqNucleotideId", "value": mane}]}]}
            else:
                unexpected.append(self.path)
                value = {"error": "Unexpected authored-control request"}
            body = json.dumps(value).encode()
            self.send_response(200 if not unexpected else 404)
            self.send_header("Content-Length", str(len(body)))
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, *_args):
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    origin = f"http://127.0.0.1:{server.server_port}"
    env = isolated_command_env(tmp_path, {"BIOMCP_MYVARIANT_BASE": origin + "/variant",
        "BIOMCP_MYGENE_BASE": origin + "/gene", "BIOMCP_UNIPROT_BASE": origin + "/uniprot"}, origin)
    binary = os.environ.get("BIOMCP_BIN", str(REPO_ROOT / "target/debug/biomcp"))
    try:
        result = subprocess.run([binary, "--json", "--no-cache", "get", "variant", f"{gene} {change}"],
                                env=env, capture_output=True, text=True, timeout=30)
        message = result.stdout + result.stderr
        assert not unexpected, unexpected
        if projection in {"paired", "canonical"}:
            assert result.returncode == 0, message
            payload = json.loads(result.stdout)
            assert payload["id"] == allele
            assert payload["hgvs_p"] == target[2]
            assert ("follows another transcript's numbering" in message) == (projection == "paired")
        else:
            assert result.returncode != 0, message
            assert "No MANE-numbered variant matches" in message
            assert "versioned transcript projection" in message
            assert "follows another transcript's numbering" not in message
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)

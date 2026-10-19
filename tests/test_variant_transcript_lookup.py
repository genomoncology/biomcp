"""Versioned transcript lookup must retain DNA identity and matched provenance."""
from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import sys
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, unquote, urlsplit

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent / "golden"))
from replay_env import REPO_ROOT, isolated_command_env  # noqa: E402

pytestmark = [pytest.mark.needs_binary]
FIXTURE = json.loads((REPO_ROOT / "tests/fixtures/variant-transcript-lookup/authored.json").read_text())


@pytest.mark.parametrize("identifier", FIXTURE["inputs"])
@pytest.mark.parametrize("normalizer", ["variantvalidator", "car"])
def test_versioned_transcript_resolves_on_proven_dna_and_names_the_match(
    identifier: str, normalizer: str, tmp_path: Path,
) -> None:
    run_lookup(tmp_path, identifier, normalizer, "same_allele")


@pytest.mark.parametrize("case", ["unavailable", "different_allele"])
def test_transcript_lookup_refuses_unverified_or_conflicting_identity(case: str, tmp_path: Path) -> None:
    identifier = FIXTURE["inputs"][1 if case == "different_allele" else 2]
    run_lookup(tmp_path, identifier, "variantvalidator", case)


def test_exact_transcript_alias_fallback_preserves_the_source_assembly(tmp_path: Path) -> None:
    run_lookup(tmp_path, FIXTURE["inputs"][2], "variantvalidator", "alias_fallback")


def run_lookup(tmp_path: Path, identifier: str, normalizer: str, case: str) -> None:
    """All responses are authored controls. Captured provider bytes stay private."""
    hit = json.loads(json.dumps(FIXTURE["hit"]))
    if case == "different_allele":
        hit["_id"] = "chr1:g.45331556C>A"
    if case == "alias_fallback":
        hit["_id"] = FIXTURE["genomic_grch37"].replace("NC_000001.10", "chr1")
        hit["clinvar"]["hgvs"]["coding"].append(identifier)
    requests: list[tuple[str, dict[str, list[str]]]] = []
    misses: list[str] = []

    class Handler(BaseHTTPRequestHandler):
        def reply(self, value: object, status: int = 200) -> None:
            body = json.dumps(value).encode()
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def do_GET(self) -> None:  # noqa: N802
            target = urlsplit(self.path)
            query = parse_qs(target.query)
            path = unquote(target.path)
            requests.append((path, query))
            if path.startswith("/VariantValidator/variantvalidator/"):
                if case != "unavailable" and normalizer == "variantvalidator":
                    return self.reply({identifier: {
                        "submitted_variant": identifier, "hgvs_transcript_variant": identifier,
                        "primary_assembly_loci": {"grch38": {"hgvs_genomic_description": FIXTURE["genomic"]}},
                    }})
                return self.reply({"message": "Authored unavailable normalizer control"}, 404)
            if path.startswith("/normalize/"):
                return self.reply({"message": "Authored unavailable normalizer control"}, 404)
            if path == "/allele":
                assert query["hgvs"] == [identifier]
                if case != "unavailable" and normalizer == "car":
                    return self.reply({"@id": "http://reg.genome.network/allele/CA011561",
                                       "genomicAlleles": [{"hgvs": [FIXTURE["genomic"]], "referenceGenome": "GRCh38"}]})
                return self.reply({"error": "Authored unavailable normalizer control"}, 404)
            if path.startswith("/v1/variant/"):
                if case != "alias_fallback" and path == "/v1/variant/chr1:g.45331556C>T" and query.get("assembly") == ["hg38"]:
                    return self.reply(hit)
                return self.reply({"error": "not found"}, 404)
            if path == "/efetch.fcgi":
                return self.reply({"error": "Authored direct ClinVar unavailable control"}, 404)
            if path == "/v1/query":
                # This hit has only another transcript's spelling. Its presence
                # alone must never confirm the requested transcript.
                return self.reply({"total": 1, "hits": [hit]})
            misses.append(self.path)
            self.reply({"error": "Unexpected fixture request"}, 404)

        def log_message(self, *_args: object) -> None:
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    origin = f"http://127.0.0.1:{server.server_port}"
    providers = {key: origin for key in (
        "BIOMCP_MUTALYZER_BASE_URL", "BIOMCP_VARIANTVALIDATOR_BASE_URL", "BIOMCP_CLINGEN_CAR_BASE",
        "BIOMCP_CLINVAR_BASE",
    )}
    providers["BIOMCP_MYVARIANT_BASE"] = origin + "/v1"
    env = isolated_command_env(tmp_path / "isolated", providers, origin)
    env.pop("BIOMCP_DISABLE_SOURCES", None)
    # The source's GRCh38 evidence must survive an opposite user preference.
    env["BIOMCP_DEFAULT_ASSEMBLY"] = "hg19"
    binary = os.environ.get("BIOMCP_BIN", str(REPO_ROOT / "target/debug/biomcp"))
    command = [binary, "--no-cache", "get", "variant", identifier, "clinvar"]
    try:
        result = subprocess.run([binary, "--json", *command[1:]], env=env, cwd=REPO_ROOT,
                                capture_output=True, text=True, timeout=60)
        assert not misses, misses
        if case not in ("same_allele", "alias_fallback"):
            assert result.returncode != 0, result.stdout
            message = result.stdout + result.stderr
            if case == "unavailable":
                assert "unverified" in message.lower(), message
                assert "belongs to a different transcript" not in message.lower()
            else:
                assert "no matching genomic allele" in message.lower(), message
            return
        assert result.returncode == 0, result.stdout + result.stderr
        payload = json.loads(result.stdout)
        transcript, coding = identifier.split(":", 1)
        assert payload["id"] == ("chr1:g.45797228C>T" if case == "alias_fallback" else "chr1:g.45331556C>T")
        assert payload["rsid"] == "rs36053993"
        assert payload["clinvar"]["variation_id"] == 5294
        assert payload["genome_build"] == ("GRCh37" if case == "alias_fallback" else "GRCh38")
        if case == "alias_fallback":
            assert payload["genome_build_provenance"] == "MyVariant.info provider default"
        assert payload["matched_transcript"] == transcript
        assert payload["matched_hgvs_c"] == coding
        assert payload["transcript"] == "NM_001048174.2"
        assert payload["hgvs_c"] == "c.1103G>A"
        assert payload["hgvs_p"] == "p.Gly368Asp"
        assert any(path.startswith("/v1/variant/") and query.get("assembly") == ["hg38"]
                   for path, query in requests)
        markdown = subprocess.run(command[:-1], env=env, cwd=REPO_ROOT,
                                  capture_output=True, text=True, timeout=60)
        assert markdown.returncode == 0, markdown.stdout + markdown.stderr
        assert f"Matched transcript: {transcript} ({coding})" in markdown.stdout
        assert "Transcript: NM_001048174.2" in markdown.stdout
        assert not misses, misses
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)

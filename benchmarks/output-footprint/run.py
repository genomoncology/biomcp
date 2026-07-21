#!/usr/bin/env -S uv run --no-sync
"""Measure deterministic BioMCP CLI output against committed replay payloads."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import threading
from dataclasses import dataclass
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any
from urllib.parse import parse_qs, urlparse

import tiktoken

REPO_ROOT = Path(__file__).resolve().parents[2]
TOKENIZER = "cl100k_base"
COMPACT_BYTE_CEILINGS = {
    "article_search_compact": 1_600,
    "variant_search": 700,
    "gene_get_sections": 7_000,
    "trial_search": 500,
}


@dataclass(frozen=True)
class Case:
    id: str
    args: tuple[str, ...]
    compact_surface: bool = True


CASES = (
    Case(
        "article_search_compact",
        (
            "--json",
            "search",
            "article",
            "-g",
            "BRAF",
            "--source",
            "pubmed",
            "--limit",
            "2",
        ),
    ),
    Case(
        "article_search_full",
        (
            "--json",
            "search",
            "article",
            "-g",
            "BRAF",
            "--source",
            "pubmed",
            "--limit",
            "2",
            "--full",
        ),
        compact_surface=False,
    ),
    Case(
        "variant_search",
        ("--json", "search", "variant", "-g", "BRAF", "--limit", "1"),
    ),
    Case(
        "gene_get_sections",
        ("--json", "get", "gene", "BRAF", "pathways"),
    ),
    Case(
        "trial_search",
        ("--json", "search", "trial", "-c", "melanoma", "--limit", "1"),
    ),
)


def _fixture(path: str) -> Any:
    return json.loads((REPO_ROOT / path).read_text(encoding="utf-8"))


class ReplayHandler(BaseHTTPRequestHandler):
    """Serve only the provider routes used by the fixed benchmark corpus."""

    def _send_json(self, payload: Any) -> None:
        body = json.dumps(payload, separators=(",", ":")).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self) -> None:  # noqa: N802 - stdlib handler API
        parsed = urlparse(self.path)
        path = parsed.path
        payloads = {
            "/mygene/query": "testdata/sources/mygene/get_braf.json",
            "/myvariant/query": "testdata/sources/myvariant/search_braf.json",
            "/pubmed/esearch.fcgi": "testdata/sources/pubmed/esearch_braf.json",
            "/ctgov/studies": "testdata/sources/clinicaltrials/search.json",
        }
        if path in payloads:
            self._send_json(_fixture(payloads[path]))
            return
        if path == "/pubmed/esummary.fcgi":
            fixture = _fixture("testdata/sources/pubmed/esummary_two_ids.json")
            result = fixture["result"]
            self._send_json(
                {
                    "result": {
                        "uids": ["123", "456"],
                        "123": result["1"] | {"uid": "123"},
                        "456": result["2"] | {"uid": "456"},
                    }
                }
            )
            return
        if path == "/pubtator/publications/export/biocjson":
            pmid = parse_qs(parsed.query)["pmids"][0]
            fixture = _fixture("testdata/sources/pubtator/export_22663011.json")
            fixture["PubTator3"][0]["pmid"] = int(pmid)
            self._send_json(fixture)
            return
        if path == "/europe/search":
            query = parse_qs(parsed.query)["query"][0]
            pmid = query.split("EXT_ID:", 1)[-1].split(" ", 1)[0]
            fixture = _fixture("testdata/sources/europepmc/search_pmid_22663011.json")
            fixture["resultList"]["result"][0].update({"id": pmid, "pmid": pmid})
            self._send_json(fixture)
            return
        if path == "/reactome/search/query":
            self._send_json({"results": [], "totalResults": 0})
            return
        self.send_error(404, f"unexpected replay route: {path}")

    def do_POST(self) -> None:  # noqa: N802 - stdlib handler API
        path = urlparse(self.path).path
        if path == "/opentargets/graphql":
            self._send_json(
                {
                    "data": {
                        "target": {
                            "associatedDiseases": {"rows": []},
                            "drugAndClinicalCandidates": {"rows": []},
                        }
                    }
                }
            )
            return
        self.send_error(404, f"unexpected replay route: {path}")

    def log_message(self, format: str, *args: object) -> None:
        return


def _benchmark_env(base_url: str) -> dict[str, str]:
    env = {
        key: value for key, value in os.environ.items() if not key.startswith("BIOMCP_")
    }
    env.update(
        {
            "BIOMCP_CACHE_MODE": "off",
            "BIOMCP_MYGENE_BASE": f"{base_url}/mygene",
            "BIOMCP_MYVARIANT_BASE": f"{base_url}/myvariant",
            "BIOMCP_PUBMED_BASE": f"{base_url}/pubmed",
            "BIOMCP_PUBTATOR_BASE": f"{base_url}/pubtator",
            "BIOMCP_EUROPEPMC_BASE": f"{base_url}/europe",
            "BIOMCP_CTGOV_BASE": f"{base_url}/ctgov",
            "BIOMCP_OPENTARGETS_BASE": f"{base_url}/opentargets",
            "BIOMCP_REACTOME_BASE": f"{base_url}/reactome",
            "HTTP_PROXY": "http://127.0.0.1:9",
            "HTTPS_PROXY": "http://127.0.0.1:9",
            "ALL_PROXY": "http://127.0.0.1:9",
            "NO_PROXY": "127.0.0.1,localhost",
            "http_proxy": "http://127.0.0.1:9",
            "https_proxy": "http://127.0.0.1:9",
            "all_proxy": "http://127.0.0.1:9",
            "no_proxy": "127.0.0.1,localhost",
        }
    )
    return env


def _run_case(
    binary: Path, case: Case, env: dict[str, str], encoding: Any
) -> dict[str, Any]:
    completed = subprocess.run(
        [str(binary), *case.args],
        cwd=REPO_ROOT,
        env=env,
        check=False,
        capture_output=True,
        timeout=30,
    )
    if completed.returncode != 0:
        stderr = completed.stderr.decode(errors="replace").strip()
        stdout = completed.stdout.decode(errors="replace").strip()
        detail = stderr or stdout or "no diagnostic output"
        raise RuntimeError(f"{case.id} failed ({completed.returncode}): {detail}")
    output = completed.stdout
    text = output.decode("utf-8")
    return {
        "id": case.id,
        "command": "biomcp " + " ".join(case.args),
        "output_bytes": len(output),
        "token_estimate": len(encoding.encode(text)),
        "compact_surface": case.compact_surface,
    }


def _compact_ratchet(output_bytes: dict[str, int]) -> dict[str, Any]:
    regressions = [
        {
            "id": case_id,
            "output_bytes": output_bytes[case_id],
            "byte_ceiling": ceiling,
        }
        for case_id, ceiling in COMPACT_BYTE_CEILINGS.items()
        if output_bytes[case_id] > ceiling
    ]
    return {
        "byte_ceilings": COMPACT_BYTE_CEILINGS,
        "regressions": regressions,
        "passed": not regressions,
    }


def collect(binary: Path) -> dict[str, Any]:
    """Run the fixed corpus against an isolated loopback replay server."""
    encoding = tiktoken.get_encoding(TOKENIZER)
    server = ThreadingHTTPServer(("127.0.0.1", 0), ReplayHandler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        base_url = f"http://127.0.0.1:{server.server_port}"
        commands = [
            _run_case(binary, case, _benchmark_env(base_url), encoding)
            for case in CASES
        ]
    finally:
        server.shutdown()
        server.server_close()
        thread.join()

    by_id = {row["id"]: row for row in commands}
    compact = by_id["article_search_compact"]
    full = by_id["article_search_full"]
    return {
        "schema_version": 1,
        "tokenizer": TOKENIZER,
        "commands": commands,
        "headline": {
            "compact_bytes": compact["output_bytes"],
            "full_bytes": full["output_bytes"],
            "bytes_saved": full["output_bytes"] - compact["output_bytes"],
            "compact_tokens": compact["token_estimate"],
            "full_tokens": full["token_estimate"],
            "tokens_saved": full["token_estimate"] - compact["token_estimate"],
        },
        "ratchet": _compact_ratchet(
            {
                row["id"]: row["output_bytes"]
                for row in commands
                if row["compact_surface"]
            }
        ),
    }


def _parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--binary",
        type=Path,
        default=Path(os.environ.get("BIOMCP_BIN", REPO_ROOT / "target/debug/biomcp")),
        help="BioMCP binary to benchmark (default: BIOMCP_BIN or target/debug/biomcp)",
    )
    return parser.parse_args()


def main() -> int:
    args = _parse_args()
    if not args.binary.is_file():
        raise SystemExit(f"BioMCP binary not found: {args.binary}")
    report = collect(args.binary.resolve())
    print(json.dumps(report, indent=2, sort_keys=True))
    if not report["ratchet"]["passed"]:
        for regression in report["ratchet"]["regressions"]:
            print(
                f"{regression['id']} output is {regression['output_bytes']} bytes; "
                f"ceiling is {regression['byte_ceiling']}",
                file=sys.stderr,
            )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

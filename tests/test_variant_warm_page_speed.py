"""Recorded AKT1 pages reuse discovery; elapsed time is qualification evidence."""

from __future__ import annotations

from collections import Counter
import json
import os
from pathlib import Path
import subprocess
import time

import pytest
from hang_guard import HANG_TIMEOUT
from variant_article_evidence_replay import CAPTURE, akt1_replay

ROOT = Path(__file__).resolve().parents[1]
pytestmark = [pytest.mark.needs_binary, pytest.mark.long]


@pytest.mark.parametrize("scenario", ["complete-control", "recorded"])
def test_recorded_variant_warm_page_reuses_completed_literal_discovery(tmp_path, scenario):
    """Feature: variant. A stored page does not repeat settled discovery."""
    binary = os.environ.get("BIOMCP_BIN", str(ROOT / "target/debug/biomcp"))
    capture = None
    if scenario == "complete-control":
        # The exact-strategy control uses the recorded public identifier list.
        # Its metadata is authored; annotation identity is not exercised.
        pmids = json.loads(CAPTURE.read_text())["pubmed_order"]
        capture = {"pubmed_order": pmids, "rows": [
            {"pmid": pmid, "title": "Authored warm page control",
             "abstract": "Authored metadata control", "citation_count": 7,
             "influential_citation_count": 1, "provenance": []}
            for pmid in pmids
        ]}
    with akt1_replay(tmp_path, cache=True, capture=capture) as replay:
        strategy = []
        if scenario == "complete-control":
            replay.native["pubmed"][:] = capture["pubmed_order"]
            replay.env.pop("BIOMCP_DISABLE_SOURCES", None)
            strategy = ["--strategy", "exact"]

        def page(offset):
            before = len(replay.requests)
            command = [binary, "--json", "variant", "articles", "AKT1 E17K",
                       "--limit", "50", "--offset", str(offset), "--debug-plan", *strategy]
            started = time.monotonic()
            process = subprocess.run(command, env=replay.env, cwd=ROOT,
                                     capture_output=True, text=True, timeout=HANG_TIMEOUT)
            elapsed = time.monotonic() - started
            assert process.returncode in (0, 1), (process.stdout, process.stderr)
            payload = json.loads(process.stdout)
            requests = replay.requests[before:]
            print(json.dumps({
                "scenario": scenario, "load_average": os.getloadavg(),
                "cache_mode": replay.env["BIOMCP_CACHE_MODE"],
                "argv": command[1:], "offset": offset, "elapsed_seconds": elapsed,
                "complete": payload.get("complete"),
                "rows": len(payload.get("results", [])),
                "pagination": payload.get("pagination"),
                "request_paths": dict(Counter(path for path, _ in requests)),
                "route_providers": [
                    {"route": route["route"], **provider}
                    for route in payload.get("debug_plan", {}).get("routes", [])
                    for provider in route["providers"]
                ],
            }), flush=True)
            return payload, requests

        first, _ = page(0)
        continuations = [page(50) for _ in range(3)]
        if scenario == "complete-control":
            assert first["complete"] is True
        assert len(first["results"]) == 50
        expected = [row["pmid"] for row in continuations[0][0]["results"]]
        assert len(expected) == len(set(expected)) == 50
        assert not set(expected) & {row["pmid"] for row in first["results"]}
        for payload, requests in continuations:
            assert [row["pmid"] for row in payload["results"]] == expected
            repeated_literal = [query for path, query in requests
                                if path.endswith("/esearch.fcgi")
                                and query.get("term") == ["AKT1 E17K"]]
            assert repeated_literal == [], "page two repeated settled literal PubMed discovery"
            if first["complete"]:
                # No-store provider replies prevent HTTP cache hits from hiding
                # a rebuild. Visible metadata remains allowed on a stored page.
                discovery = [path for path, query in requests
                             if path.endswith("/esearch.fcgi")
                             or path in ("/search/", "/variant/autocomplete/", "/entity/autocomplete/")
                             or path.startswith("/graph/v1/paper/search")
                             or (path == "/search" and not query.get("query", [""])[0].startswith("EXT_ID:"))]
                assert discovery == [], "a complete warm page repeated discovery"

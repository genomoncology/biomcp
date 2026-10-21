"""Keep recorded paper replies separate from authored discovery controls."""

from __future__ import annotations

import json
from pathlib import Path


def prepare_transcript_alias_replay(capture: Path, destination: Path) -> None:
    destination.mkdir(parents=True)
    (destination / "responses").symlink_to(capture / "responses", target_is_directory=True)
    manifest = json.loads((capture / "manifest.json").read_text())
    # These queries were not captured. Empty authored responses keep the replay
    # about transcript provenance, without claiming these aliases find no papers.
    replies = {
        "europepmc": {"hitCount": 0, "resultList": {"result": []}},
        "pubmed": {"esearchresult": {"idlist": [], "count": "0"}},
        "pubtator": {"results": [], "count": 0, "total_pages": 0, "current": 1},
        "s2": {"total": 0, "data": []},
    }
    for provider, body in replies.items():
        (destination / f"authored-empty-{provider}.json").write_text(json.dumps(body))
    templates = [key for key in manifest["routes"] if "c.211T%3EA" in key]
    for alias in ("c.5140T%3EA", "c.5281T%3EA", "c.5344T%3EA", "c.754T%3EA"):
        for template in templates:
            provider = template.split("/", 2)[1]
            manifest["routes"][template.replace("c.211T%3EA", alias)] = {
                "status": 200, "file": f"authored-empty-{provider}.json",
                "provenance": "AUTHORED empty discovery control; not captured provider bytes",
            }
    # The primary query's recorded S2 reply includes this PMID. Its paper body
    # was not captured; an authored outage must never turn it into verified evidence.
    (destination / "authored-verification-unavailable.json").write_text(
        json.dumps({"error": "Authored unavailable verification control"})
    )
    manifest["routes"]["GET /pubtator/publications/export/biocjson?full=true&pmids=35077220"] = {
        "status": 503, "file": "authored-verification-unavailable.json",
        "provenance": "AUTHORED unavailable verification control; not a captured outage",
    }
    (destination / "manifest.json").write_text(json.dumps(manifest))

#!/usr/bin/env -S uv run --no-project
"""Fetch live PubTator BioC JSON for the fixed corpus into ignored work/."""

from __future__ import annotations

import json
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CORPUS = ROOT / "fixtures" / "corpus.json"
OUTPUT = ROOT / "work" / "pubtator.json"
BASE = "https://www.ncbi.nlm.nih.gov/research/pubtator3-api/publications/export/biocjson"


def main() -> None:
    corpus = json.loads(CORPUS.read_text())
    documents = []
    unavailable = []
    for index, document in enumerate(corpus["documents"]):
        pmid = document["pmid"]
        url = f"{BASE}?{urllib.parse.urlencode({'pmids': pmid})}"
        request = urllib.request.Request(url, headers={"User-Agent": "biomcp-sv-spike/1.0"})
        try:
            with urllib.request.urlopen(request, timeout=60) as response:  # noqa: S310 - fixed NCBI host
                payload = json.load(response)
        except urllib.error.HTTPError as error:
            unavailable.append({"pmid": pmid, "http_status": error.code})
        else:
            documents.extend(payload.get("PubTator3", []))
        if index + 1 < len(corpus["documents"]):
            time.sleep(0.35)
    OUTPUT.parent.mkdir(exist_ok=True)
    OUTPUT.write_text(
        json.dumps({"PubTator3": documents, "unavailable": unavailable}, indent=2) + "\n"
    )
    print(
        f"wrote {len(documents)} PubTator documents to {OUTPUT}; "
        f"{len(unavailable)} unavailable"
    )


if __name__ == "__main__":
    main()

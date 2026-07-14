#!/usr/bin/env -S uv run --no-project
"""Fetch the fixed full-scale PMID set from NCBI into ignored work/.

This acquisition helper is never used by repository gates. The reviewed text snapshot is
checked in as fixtures/blind_corpus.json so routine evaluation is offline.
"""

from __future__ import annotations

import json
import urllib.parse
import urllib.request
import xml.etree.ElementTree as ET
from pathlib import Path

from build_blind_corpus import SELECTION

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "work" / "blind-candidates.json"


def node_text(node: ET.Element | None) -> str:
    return "" if node is None else "".join(node.itertext()).strip()


def main() -> None:
    pmids = [pmid for rows in SELECTION.values() for pmid in rows]
    query = urllib.parse.urlencode(
        {"db": "pubmed", "id": ",".join(pmids), "rettype": "abstract", "retmode": "xml"}
    )
    request = urllib.request.Request(  # noqa: S310 - fixed NCBI host
        f"https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?{query}",
        headers={"User-Agent": "BioMCP-structural-event-experiment/1.0"},
    )
    with urllib.request.urlopen(request, timeout=60) as response:  # noqa: S310
        payload = response.read()
    records = []
    for article in ET.fromstring(payload).findall(".//PubmedArticle"):
        records.append(
            {
                "pmid": node_text(article.find(".//PMID")),
                "title": node_text(article.find(".//ArticleTitle")),
                "abstract": " ".join(
                    node_text(part)
                    for part in article.findall(".//AbstractText")
                    if node_text(part)
                ),
            }
        )
    by_pmid = {record["pmid"]: record for record in records}
    missing = [
        pmid for pmid in pmids if pmid not in by_pmid or not by_pmid[pmid]["abstract"]
    ]
    if missing:
        raise ValueError(f"NCBI response lacked abstracts for: {missing}")
    OUTPUT.parent.mkdir(exist_ok=True)
    OUTPUT.write_text(
        json.dumps([by_pmid[pmid] for pmid in pmids], indent=2, ensure_ascii=False)
        + "\n"
    )
    print(f"wrote {len(pmids)} documents to {OUTPUT}")


if __name__ == "__main__":
    main()

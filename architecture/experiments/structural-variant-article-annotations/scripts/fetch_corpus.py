#!/usr/bin/env -S uv run --no-project
"""Fetch the fixed PMID evaluation set from NCBI EFetch.

This acquisition helper is not used by repository gates. It writes generated source
material to the experiment's ignored work directory; the reviewed compact corpus is
checked in separately.
"""

from __future__ import annotations

import json
import urllib.parse
import urllib.request
import xml.etree.ElementTree as ET
from pathlib import Path

PMIDS = [
    "30709865", "35637217", "37449980", "39796213", "34885058",
    "42426366", "41935330", "42379467", "42366058", "42206448",
    "42404792", "42440233", "42420503", "42436300", "42440172",
    "42435518",
]
ROOT = Path(__file__).resolve().parents[1]
WORK = ROOT / "work"


def node_text(node: ET.Element | None) -> str:
    return "" if node is None else "".join(node.itertext()).strip()


def main() -> None:
    WORK.mkdir(exist_ok=True)
    query = urllib.parse.urlencode({
        "db": "pubmed", "id": ",".join(PMIDS), "rettype": "abstract", "retmode": "xml"
    })
    url = f"https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?{query}"
    with urllib.request.urlopen(url, timeout=60) as response:  # noqa: S310 - fixed NCBI host
        payload = response.read()
    (WORK / "corpus.xml").write_bytes(payload)

    records = []
    for article in ET.fromstring(payload).findall(".//PubmedArticle"):
        pmid = node_text(article.find(".//PMID"))
        title = node_text(article.find(".//ArticleTitle"))
        abstract = " ".join(
            node_text(part) for part in article.findall(".//AbstractText") if node_text(part)
        )
        records.append({"pmid": pmid, "title": title, "abstract": abstract})
    by_pmid = {record["pmid"]: record for record in records}
    ordered = [by_pmid[pmid] for pmid in PMIDS]
    (WORK / "source_documents.json").write_text(
        json.dumps(ordered, indent=2, ensure_ascii=False) + "\n"
    )
    print(f"wrote {len(ordered)} documents to {WORK / 'source_documents.json'}")


if __name__ == "__main__":
    main()

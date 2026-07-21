#!/usr/bin/env bash
set -euo pipefail

workspace_root="${1:-$PWD}"
repo_root="$(git -C "$workspace_root" rev-parse --show-toplevel 2>/dev/null || printf '%s\n' "$workspace_root")"
cache_dir="$workspace_root/.cache"
env_file="$cache_dir/spec-article-fulltext-source-env"
lock_file="$cache_dir/spec-article-fulltext-source.lock"

mkdir -p "$cache_dir"
exec 9>"$lock_file"
flock 9

bash "$repo_root/spec/fixtures/cleanup-article-fulltext-source-fixture.sh" "$workspace_root"

fixture_root="$(mktemp -d "$cache_dir/spec-article-fulltext-source.XXXXXX")"
ready_file="$fixture_root/base-url"
server_log="$fixture_root/server.log"
request_log="$fixture_root/request-log.txt"
: > "$request_log"

python3 - "$ready_file" "$repo_root/tests/fixtures/article/fulltext" "$request_log" 9>&- <<'PY' >"$server_log" 2>&1 &
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, unquote, urlparse
import io
import json
import sys
import tarfile
import threading
import time
import zipfile


FIXTURE_DIR = Path(sys.argv[2])
REQUEST_LOG = Path(sys.argv[3])
REQUEST_LOG_LOCK = threading.Lock()
HTML_FALLBACK = (
    FIXTURE_DIR / "html" / "pmc_article_page.html"
).read_text(encoding="utf-8")
PDF_FALLBACK = (
    FIXTURE_DIR / "pdf" / "pmc_oa_article_pdf.pdf"
).read_bytes()
FIGSHARE_SUPPLEMENT = b"%PDF-1.4\nFigshare supplemental fixture bytes\n%%EOF\n"
FIGSHARE_TABLE_S1 = b"PK\x03\x04\nS1 workbook fixture bytes\n"
FIGSHARE_TABLE_S2 = b"PK\x03\x04\nS2 workbook fixture bytes\n"
FIGSHARE_UNRELATED_TABLE = b"PK\x03\x04\nUnrelated workbook fixture bytes\n"
FIGSHARE_COLD_STORAGE = b"%PDF-1.4\nFigshare cold-storage fixture bytes\n%%EOF\n"
LINKED_JATS_SUPPLEMENT = b"linked JATS supplement fixture bytes\n"
LINKED_HTML_SUPPLEMENT = b"PK\x03\x04\nlinked PMC HTML supplement fixture bytes\n"
COLD_STORAGE_LOCK = threading.Lock()
COLD_STORAGE_HITS = {}

AUTHOR_ENTITY_SEARCH = {
    "total": 2,
    "offset": 0,
    "data": [
        {"authorId": "2269573451", "name": "Louis S. Williams", "affiliations": ["Cleveland Clinic"], "paperCount": 42, "citationCount": 900, "hIndex": 15},
        {"authorId": "1994488914", "name": "Louis S. Williams", "affiliations": ["Cleveland Clinic"], "paperCount": 18, "citationCount": 250, "hIndex": 8},
    ],
}
AUTHOR_ENTITY_DETAIL = {"authorId": "1716151", "name": "A. Butte", "affiliations": ["University of California, San Francisco"], "paperCount": 548, "citationCount": 50000, "hIndex": 100}
AUTHOR_FORBIDDEN = {"email": "private-author@example.invalid", "homepage": "https://private.example.invalid/author/1716151", "private_profile": "fixture-private-profile", "gender": "fixture-inferred-demographic", "race": "fixture-inferred-demographic", "ethnicity": "fixture-inferred-demographic", "externalIds": {"ORCID": "0000-0002-7433-2740"}}
for row in AUTHOR_ENTITY_SEARCH["data"]:
    row.update(AUTHOR_FORBIDDEN)
AUTHOR_ENTITY_DETAIL.update(AUTHOR_FORBIDDEN)

AUTHOR_SEARCH = {
    "europepmc": {
        "pmid": "51300001",
        "title": "Williams LS Europe PMC byline match",
    },
    "pubmed": {
        "pmid": "51300002",
        "title": "Williams LS PubMed byline match",
    },
    "pubtator": {
        "pmid": "51300003",
        "title": "Williams syndrome PubTator lexical false positive",
    },
    "semanticscholar": {
        "pmid": "51300004",
        "title": "Williams syndrome Semantic Scholar lexical false positive",
    },
    "bounded_pubmed": {
        "pmid": "51700001",
        "title": "Taylor EJ PubMed byline match",
    },
}


ARTICLE_XML = """<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE article PUBLIC
  "-//NLM//DTD JATS (Z39.96) Journal Archiving and Interchange DTD v1.4 20241031//EN"
  "https://example.invalid/JATS-archivearticle1.dtd">
<article xmlns:xlink="http://www.w3.org/1999/xlink">
  <front>
    <article-meta>
      <title-group><article-title>Europe full text winner</article-title></title-group>
      <abstract><p>Abstract text.</p></abstract>
    </article-meta>
  </front>
  <body>
    <sec>
      <title>Fixture results</title>
      <p>Europe PMC body text with callout (<xref ref-type="fig" rid="fig2">Figure 2</xref>) and B-RAF<sup>V600E</sup>.PLX4032 boundary text.</p>
      <p>External DTD numeric-reference evidence measures 70 &#181;m.</p>
      <fig id="fig1">
        <label>Figure 1</label>
        <caption><p>Inline figure caption preserves n=10 cell counts.</p></caption>
        <graphic xlink:href="figure-inline.png" />
      </fig>
      <table-wrap id="t1">
        <label>Table 1</label>
        <caption><p>Fixture quality table.</p></caption>
        <table>
          <tr><th>Signal</th><th>Value</th></tr>
          <tr><td>full text</td><td>present</td></tr>
        </table>
      </table-wrap>
      <table-wrap id="t2">
        <label>Table 2</label>
        <caption><p>Merged treatment table.</p></caption>
        <table>
          <tr><th rowspan="2">Cohort</th><th>Baseline</th><th>Week 8</th></tr>
          <tr><td>10</td><td>4</td></tr>
        </table>
      </table-wrap>
      <supplementary-material id="s1" xlink:href="traces-s1.csv">
        <label>Supplementary Data S1</label>
        <caption><p>Measurement traces for the treatment cohort.</p></caption>
        <media xlink:href="traces-s1.csv" />
      </supplementary-material>
      <supplementary-material id="s2">
        <label>Supplementary Data S2</label>
        <caption><p>Linked-only JATS measurements.</p></caption>
        <media xlink:href="linked-jats-s2.csv" mimetype="text" mime-subtype="csv" />
      </supplementary-material>
    </sec>
  </body>
  <floats-group>
    <fig id="fig2">
      <label>Figure 2</label>
      <caption><p>Floats-group figure reports measurement bar is 70 μm.</p></caption>
      <graphic xlink:href="figure-floats.png" />
    </fig>
  </floats-group>
  <back>
    <ref-list>
      <ref id="R1"><mixed-citation>Fixture reference.</mixed-citation></ref>
    </ref-list>
  </back>
</article>"""

PMC_OA_ONLY_XML = """<article xmlns:xlink="http://www.w3.org/1999/xlink">
  <front>
    <article-meta>
      <title-group><article-title>PMC OA archive full text winner</article-title></title-group>
      <abstract><p>PMC OA abstract text.</p></abstract>
    </article-meta>
  </front>
  <body>
    <sec>
      <title>PMC OA results</title>
      <p>PMC OA Archive XML body text with fixture-only provenance.</p>
    </sec>
  </body>
</article>"""

ABSTRACT_ONLY_XML = """<article>
  <!-- SENSITIVE-ABSTRACT-SOURCE-BODY signed.example.invalid token=secret -->
  <front>
    <article-meta>
      <title-group><article-title>SENSITIVE-ABSTRACT-TITLE-CANARY</article-title></title-group>
      <abstract><p>Abstract-only fixture evidence.</p></abstract>
    </article-meta>
  </front>
</article>"""

METADATA_ONLY_HTML = """<!doctype html>
<html>
  <head>
    <title>SENSITIVE-METADATA-TITLE-CANARY</title>
    <!-- SENSITIVE-METADATA-SOURCE-BODY signed.example.invalid token=secret -->
  </head>
  <body>
    <main><h1>SENSITIVE-METADATA-TITLE-CANARY</h1></main>
  </body>
</html>"""

ABSTRACT_ONLY_HTML = """<!doctype html>
<html>
  <head>
    <title>SENSITIVE-HTML-TITLE-CANARY</title>
    <!-- SENSITIVE-HTML-ABSTRACT-BODY signed.example.invalid token=secret -->
  </head>
  <body>
    <main>
      <h1>SENSITIVE-HTML-TITLE-CANARY</h1>
      <section class="abstract"><h2>Abstract</h2><p>HTML abstract fixture evidence.</p></section>
    </main>
  </body>
</html>"""


def make_oa_assets_tgz():
    entries = {
        "article.nxml": ARTICLE_XML.encode("utf-8"),
        "figure-inline.png": b"fixture-inline-figure-bytes\n",
        "figure-floats.png": b"fixture-floats-figure-bytes\n",
        "traces-s1.csv": b"time,value\n0,1\n",
        "readme.txt": b"package sidecar\n",
    }
    out = io.BytesIO()
    with tarfile.open(fileobj=out, mode="w:gz") as archive:
        for name, body in entries.items():
            info = tarfile.TarInfo(name)
            info.size = len(body)
            info.mode = 0o644
            archive.addfile(info, io.BytesIO(body))
    return out.getvalue()


def make_pmc_oa_only_tgz():
    entries = {
        "pmc-oa-only.nxml": PMC_OA_ONLY_XML.encode("utf-8"),
    }
    out = io.BytesIO()
    with tarfile.open(fileobj=out, mode="w:gz") as archive:
        for name, body in entries.items():
            info = tarfile.TarInfo(name)
            info.size = len(body)
            info.mode = 0o644
            archive.addfile(info, io.BytesIO(body))
    return out.getvalue()


def make_europe_pmc_supplementary_zip():
    # Scrubbed from the observed PMC11143360 supplementaryFiles archive shape.
    # Preserve a real member name, but replace the copyrighted document contents.
    out = io.BytesIO()
    with zipfile.ZipFile(out, mode="w", compression=zipfile.ZIP_DEFLATED) as archive:
        archive.writestr(
            "41408_2024_1068_MOESM1_ESM.docx",
            b"scrubbed Europe PMC supplementary DOCX fixture bytes\n",
        )
    return out.getvalue()


OA_ASSETS_TGZ = make_oa_assets_tgz()
PMC_OA_ONLY_TGZ = make_pmc_oa_only_tgz()
EUROPE_PMC_SUPPLEMENTARY_ZIP = make_europe_pmc_supplementary_zip()


ARTICLES = {
    "22663011": {
        "pmcid": "PMC123456",
        "title": "Europe full text winner",
        "abstract": "Abstract text.",
        "authors": [
            "Ada First",
            "Ben Second",
            "Cyra Middle",
            "Dev Fourth",
            "Eli Fifth",
            "Fay Last",
        ],
        "paper_id": "paper-1",
    },
    "22663012": {
        "pmcid": "PMC123457",
        "title": "PMC HTML fallback winner",
        "abstract": "Abstract text.",
        "paper_id": "paper-2",
    },
    "22663013": {
        "pmcid": "PMC123458",
        "title": "Open access PDF fallback winner",
        "abstract": "Abstract text.",
        "paper_id": "paper-3",
    },
    "22663014": {
        "pmcid": None,
        "title": "Resolver order miss",
        "abstract": "Abstract text.",
        "paper_id": "paper-4",
    },
    "22663015": {
        "pmcid": None,
        "title": "Figshare asset fallback winner",
        "abstract": "Abstract text.",
        "paper_id": "paper-5",
    },
    "22663016": {
        "pmcid": "PMC123460",
        "title": "PMC OA archive full text winner",
        "abstract": "Abstract text.",
        "paper_id": "paper-6",
    },
    "22663017": {
        "pmcid": None,
        "title": "Figshare cold storage asset winner",
        "abstract": "Abstract text.",
        "paper_id": "paper-7",
    },
    "22663018": {
        "pmcid": "PMC123461",
        "title": "Europe PMC supplementary asset fallback winner",
        "abstract": "Abstract text.",
        "paper_id": "paper-8",
    },
    "22663019": {
        "pmcid": "PMC123462",
        "title": "Resolver failure control",
        "abstract": "Abstract text.",
        "paper_id": "paper-9",
    },
    "22663020": {
        "pmcid": "PMC123463",
        "title": "Abstract-only XML fixture",
        "abstract": "",
        "paper_id": "paper-10",
    },
    "22663021": {
        "pmcid": "PMC123464",
        "title": "Metadata-only HTML fixture",
        "abstract": "",
        "paper_id": "paper-11",
    },
    "22663022": {
        "pmcid": "PMC123465",
        "title": "Abstract-only HTML fixture",
        "abstract": "",
        "paper_id": "paper-12",
    },
}


PUBMED_INDEXING_XML = """<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE PubmedArticleSet PUBLIC "-//NLM//DTD PubMedArticle, 1st January 2025//EN" "https://example.invalid/pubmed_250101.dtd">
<PubmedArticleSet>
  <PubmedArticle>
    <MedlineCitation Status="MEDLINE">
      <PMID Version="1">22663011</PMID>
      <Article>
        <AuthorList CompleteYN="Y">
          <Author ValidYN="Y">
            <LastName>First</LastName><ForeName>Ada</ForeName><Initials>AF</Initials>
            <Identifier Source="ORCID">https://orcid.org/0000-0002-1825-0097</Identifier>
            <AffiliationInfo>
              <Affiliation>Precision Oncology Unit, Fixture University</Affiliation>
              <Identifier Source="ROR">https://ror.org/03yrm5c26</Identifier>
            </AffiliationInfo>
            <AffiliationInfo>
              <Affiliation>Translational Genomics Center, Fixture Hospital</Affiliation>
              <Identifier Source="GRID">grid.fixture.200</Identifier>
            </AffiliationInfo>
          </Author>
          <Author ValidYN="Y">
            <LastName>Second</LastName><ForeName>Ben</ForeName><Initials>BS</Initials>
            <AffiliationInfo>
              <Affiliation>Precision Oncology Unit, Fixture University</Affiliation>
              <Identifier Source="ROR">https://ror.org/03yrm5c26</Identifier>
            </AffiliationInfo>
          </Author>
          <Author ValidYN="Y">
            <LastName>Becker</LastName><ForeName>J&#xfc;rgen</ForeName><Initials>JB</Initials>
          </Author>
          <Author ValidYN="Y">
            <CollectiveName>Fixture Study Group</CollectiveName>
          </Author>
        </AuthorList>
      </Article>
      <MeshHeadingList>
        <MeshHeading>
          <DescriptorName UI="D008545" MajorTopicYN="Y">Melanoma</DescriptorName>
          <QualifierName UI="Q000235" MajorTopicYN="N">genetics</QualifierName>
          <QualifierName UI="Q000401" MajorTopicYN="Y">metabolism</QualifierName>
        </MeshHeading>
      </MeshHeadingList>
    </MedlineCitation>
  </PubmedArticle>
</PubmedArticleSet>
"""


def append_request_log(line):
    with REQUEST_LOG_LOCK:
        with REQUEST_LOG.open("a", encoding="utf-8") as handle:
            handle.write(f"{line}\n")


def send_json(handler, status, payload):
    body = json.dumps(payload).encode("utf-8")
    handler.send_response(status)
    handler.send_header("Content-Type", "application/json")
    handler.send_header("Content-Length", str(len(body)))
    handler.end_headers()
    handler.wfile.write(body)


def send_text(handler, status, body, content_type):
    payload = body.encode("utf-8")
    handler.send_response(status)
    handler.send_header("Content-Type", content_type)
    handler.send_header("Content-Length", str(len(payload)))
    handler.end_headers()
    handler.wfile.write(payload)


def send_cacheable_text(handler, status, body, content_type):
    payload = body.encode("utf-8")
    handler.send_response(status)
    handler.send_header("Content-Type", content_type)
    handler.send_header("Cache-Control", "public, max-age=3600")
    handler.send_header("Content-Length", str(len(payload)))
    handler.end_headers()
    handler.wfile.write(payload)


def send_bytes(handler, status, body, content_type):
    handler.send_response(status)
    handler.send_header("Content-Type", content_type)
    handler.send_header("Content-Length", str(len(body)))
    handler.end_headers()
    handler.wfile.write(body)


def pubtator_payload(pmid):
    article = ARTICLES[pmid]
    record = {
        "pmid": int(pmid),
        "authors": article.get("authors", []),
        "passages": [
            {"infons": {"type": "title"}, "text": article["title"]},
            {"infons": {"type": "abstract"}, "text": article["abstract"]},
        ],
    }
    if article["pmcid"]:
        record["pmcid"] = article["pmcid"]
    return {
        "PubTator3": [record]
    }


def europepmc_search_payload(pmid):
    article = ARTICLES[pmid]
    result = {
        "id": pmid,
        "pmid": pmid,
        "title": article["title"],
        "journalTitle": "Journal One",
        "firstPublicationDate": "2025-01-01",
    }
    if article["pmcid"]:
        result["pmcid"] = article["pmcid"]
        result["isOpenAccess"] = "Y"
        result["fullTextIdList"] = {"fullTextId": [article["pmcid"]]}
        result["fullTextUrlList"] = {
            "fullTextUrl": [
                {
                    "availability": "Open access",
                    "availabilityCode": "OA",
                    "documentStyle": "html",
                    "site": "Europe PMC",
                    "url": f"https://europepmc.org/articles/{article['pmcid']}",
                }
            ]
        }
    if pmid == "22663011":
        result["license"] = "CC BY"
    if pmid == "22663015":
        result["doi"] = "10.1158/fixture.figshare"
    if pmid == "22663016":
        result["license"] = "CC BY-NC"
    if pmid == "22663017":
        result["doi"] = "10.1158/fixture.figshare-cold"
    return {
        "hitCount": 1,
        "resultList": {
            "result": [result]
        },
    }


class Handler(BaseHTTPRequestHandler):
    def do_POST(self):
        parsed = urlparse(self.path)
        decoded_path = unquote(parsed.path)
        length = int(self.headers.get("Content-Length") or 0)
        if length:
            self.rfile.read(length)

        if decoded_path == "/v2/articles/search":
            send_json(self, 200, [
                {
                    "id": 22474820,
                    "title": "Figshare asset fallback winner",
                    "doi": "10.1158/fixture.figshare",
                    "url_api": f"http://127.0.0.1:{self.server.server_port}/v2/articles/22474820",
                    "url_public_html": "https://aacr.figshare.com/articles/journal_contribution/Fixture_Figshare_supplement/22474820",
                },
                {
                    "id": 22474817,
                    "title": "Supplementary Table S1 from Figshare asset fallback winner",
                    "doi": "10.1158/1078-0432.22474817.v1",
                    "url_api": f"http://127.0.0.1:{self.server.server_port}/v2/articles/22474817",
                    "url_public_html": "https://aacr.figshare.com/articles/dataset/Supplementary_Table_S1_from_Figshare_asset_fallback_winner/22474817",
                },
                {
                    "id": 22474818,
                    "title": "Supplementary Data S2 from Figshare asset fallback winner",
                    "doi": "10.1158/1078-0432.22474818.v1",
                    "url_api": f"http://127.0.0.1:{self.server.server_port}/v2/articles/22474818",
                    "url_public_html": "https://aacr.figshare.com/articles/dataset/Supplementary_Data_S2_from_Figshare_asset_fallback_winner/22474818",
                },
                {
                    "id": 99999999,
                    "title": "Unrelated figshare supplement",
                    "doi": "10.1158/unrelated.fixture",
                    "url_api": f"http://127.0.0.1:{self.server.server_port}/v2/articles/99999999",
                    "url_public_html": "https://figshare.com/articles/dataset/Unrelated/99999999",
                },
            ])
            return

        send_json(self, 404, {"error": "not found"})

    def do_GET(self):
        parsed = urlparse(self.path)
        decoded_path = unquote(parsed.path)
        query = parse_qs(parsed.query)

        pmids = query.get("pmids")
        if decoded_path == "/publications/export/biocjson" and pmids and pmids[0] in ARTICLES:
            send_json(self, 200, pubtator_payload(pmids[0]))
            return

        if decoded_path == "/search/" and query.get("text") == ["Williams LS"]:
            row = AUTHOR_SEARCH["pubtator"]
            send_json(self, 200, {
                "results": [{
                    "_id": f"pmid:{row['pmid']}",
                    "pmid": row["pmid"],
                    "title": row["title"],
                    "journal": "Lexical Fixture Journal",
                    "date": "2025-01-03",
                    "score": 99.0,
                }],
                "count": 1,
                "total_pages": 1,
                "current": 1,
                "page_size": 100,
            })
            return

        if decoded_path == "/search/" and query.get("text") == ["MSH2 p.L341P"]:
            send_json(self, 200, {
                "results": [{
                    "_id": "pmid:26951660",
                    "pmid": "26951660",
                    "title": "MSH2 p.L341P exact-variant literature fixture",
                    "journal": "Variant Literature Fixture Journal",
                    "date": "2016-05-01",
                    "score": 99.0,
                }],
                "count": 1,
                "total_pages": 1,
                "current": 1,
                "page_size": 100,
            })
            return

        search_query = query.get("query")
        if (
            decoded_path == "/search"
            and search_query
            and query.get("format") == ["json"]
            and any("AUTH:" in value and "Taylor EJ" in value for value in search_query)
        ):
            time.sleep(65)
            send_json(self, 200, {
                "hitCount": 0,
                "resultList": {"result": []},
            })
            return

        if (
            decoded_path == "/search"
            and search_query
            and query.get("format") == ["json"]
            and any("AUTH:" in value and "Williams LS" in value for value in search_query)
        ):
            row = AUTHOR_SEARCH["europepmc"]
            send_json(self, 200, {
                "hitCount": 1,
                "resultList": {"result": [{
                    "id": row["pmid"],
                    "pmid": row["pmid"],
                    "title": row["title"],
                    "journalTitle": "Byline Fixture Journal",
                    "firstPublicationDate": "2025-01-01",
                    "authorString": "Williams LS, Exact Coauthor",
                }]},
            })
            return

        if decoded_path == "/esearch.fcgi" and any(
            "Taylor EJ" in value and "[author]" in value.lower()
            for value in query.get("term", [])
        ):
            row = AUTHOR_SEARCH["bounded_pubmed"]
            ids = [] if query.get("retstart") != ["0"] else [row["pmid"]]
            send_json(self, 200, {
                "esearchresult": {"count": "1", "idlist": ids},
            })
            return

        if decoded_path == "/esearch.fcgi" and any(
            "Williams LS" in value and "[author]" in value.lower()
            for value in query.get("term", [])
        ):
            row = AUTHOR_SEARCH["pubmed"]
            send_json(self, 200, {
                "esearchresult": {"count": "1", "idlist": [row["pmid"]]},
            })
            return

        author_summary_rows = {
            AUTHOR_SEARCH["pubmed"]["pmid"]: AUTHOR_SEARCH["pubmed"],
            AUTHOR_SEARCH["bounded_pubmed"]["pmid"]: AUTHOR_SEARCH["bounded_pubmed"],
        }
        summary_ids = query.get("id")
        if decoded_path == "/esummary.fcgi" and summary_ids and summary_ids[0] in author_summary_rows:
            row = author_summary_rows[summary_ids[0]]
            send_json(self, 200, {
                "result": {
                    "uids": [row["pmid"]],
                    row["pmid"]: {
                        "uid": row["pmid"],
                        "title": row["title"],
                        "sortpubdate": "2025/01/02 00:00",
                        "pubdate": "2025 Jan 2",
                        "fulljournalname": "Byline Fixture Journal",
                        "source": "Byline Fixture Journal",
                    },
                },
            })
            return

        if decoded_path == "/graph/v1/author/search":
            send_json(self, 200, AUTHOR_ENTITY_SEARCH)
            return

        if decoded_path == "/graph/v1/author/1716151":
            send_json(self, 200, AUTHOR_ENTITY_DETAIL)
            return

        if decoded_path == "/graph/v1/paper/search" and query.get("query") == ["Williams LS"]:
            row = AUTHOR_SEARCH["semanticscholar"]
            send_json(self, 200, {
                "total": 1,
                "data": [{
                    "paperId": "author-lexical-false-positive",
                    "externalIds": {"PubMed": row["pmid"]},
                    "title": row["title"],
                    "venue": "Lexical Fixture Journal",
                    "year": 2025,
                    "citationCount": 50,
                    "influentialCitationCount": 5,
                    "abstract": "Williams syndrome lexical match without the requested byline.",
                }],
            })
            return

        if (
            decoded_path == "/search"
            and search_query
            and query.get("format") == ["json"]
            and query.get("page") == ["1"]
            and query.get("pageSize") == ["1"]
        ):
            for pmid in ARTICLES:
                if search_query == [f"EXT_ID:{pmid} AND SRC:MED"]:
                    send_json(self, 200, europepmc_search_payload(pmid))
                    return

        if (
            decoded_path == "/"
            and query.get("idtype") == ["pmid"]
            and query.get("ids") == ["22663014"]
        ):
            append_request_log("fulltext:identity:ncbi-idconv")
            send_json(self, 200, {"records": [{"pmid": 22663014, "pmcid": "PMC123459"}]})
            return

        if (
            decoded_path == "/"
            and query.get("idtype") == ["pmid"]
            and query.get("ids") in (["22663015"], ["22663017"])
        ):
            send_json(self, 200, {"records": [{"pmid": int(query.get("ids")[0])}]})
            return

        if (
            decoded_path == "/"
            and query.get("idtype") == ["doi"]
            and query.get("ids") in (["10.1158/fixture.figshare"], ["10.1158/fixture.figshare-cold"])
        ):
            send_json(self, 200, {"records": [{"doi": query.get("ids")[0]}]})
            return

        if decoded_path == "/PMC123456/fullTextXML":
            send_text(self, 200, ARTICLE_XML, "application/xml")
            return

        if decoded_path == "/PMC123461/supplementaryFiles":
            send_bytes(self, 200, EUROPE_PMC_SUPPLEMENTARY_ZIP, "application/zip")
            return

        if decoded_path == "/PMC123459/fullTextXML":
            append_request_log("fulltext:xml:europepmc-pmc")
            send_text(self, 404, "not found", "text/plain")
            return

        if decoded_path == "/PMC123462/fullTextXML":
            send_text(
                self,
                500,
                "SENSITIVE-UPSTREAM-DETAIL https://signed.example.invalid/article?token=secret",
                "text/plain",
            )
            return

        if decoded_path == "/PMC123463/fullTextXML":
            send_text(self, 200, ABSTRACT_ONLY_XML, "application/xml")
            return

        if decoded_path in {"/22663014/fullTextXML", "/22663019/fullTextXML", "/22663020/fullTextXML", "/22663021/fullTextXML", "/22663022/fullTextXML"}:
            append_request_log("fulltext:xml:europepmc-med")
            send_text(self, 404, "not found", "text/plain")
            return

        if decoded_path in {"/PMC123457/fullTextXML", "/PMC123458/fullTextXML", "/PMC123460/fullTextXML", "/PMC123464/fullTextXML", "/PMC123465/fullTextXML", "/22663012/fullTextXML", "/22663013/fullTextXML", "/22663016/fullTextXML"}:
            send_text(self, 404, "not found", "text/plain")
            return

        if decoded_path == "/" and query.get("id") == ["PMC123456"]:
            send_text(self, 200, f"""<records><record license=\"CC BY\" retracted=\"no\"><link format=\"tgz\" href=\"http://127.0.0.1:{self.server.server_port}/oa-assets-22663011.tgz\" /></record></records>""", "application/xml")
            return

        if decoded_path == "/oa-assets-22663011.tgz":
            send_bytes(self, 200, OA_ASSETS_TGZ, "application/gzip")
            return

        if decoded_path == "/articles/instance/123456/bin/linked-jats-s2.csv":
            send_bytes(self, 200, LINKED_JATS_SUPPLEMENT, "text/csv")
            return

        if decoded_path == "/articles/instance/123457/bin/linked-html-s1.xlsx":
            send_bytes(
                self,
                200,
                LINKED_HTML_SUPPLEMENT,
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            )
            return

        if decoded_path == "/" and query.get("id") == ["PMC123460"]:
            send_text(self, 200, f"""<records><record license=\"CC BY-NC\" retracted=\"no\"><link format=\"tgz\" href=\"http://127.0.0.1:{self.server.server_port}/oa-assets-22663016.tgz\" /></record></records>""", "application/xml")
            return

        if decoded_path == "/oa-assets-22663016.tgz":
            send_bytes(self, 200, PMC_OA_ONLY_TGZ, "application/gzip")
            return

        if decoded_path == "/" and query.get("id") == ["PMC123461"]:
            send_text(self, 200, f"""<records><record license=\"CC BY\" retracted=\"no\"><link format=\"tgz\" href=\"http://127.0.0.1:{self.server.server_port}/stale-oa-assets-22663018.tgz\" /></record></records>""", "application/xml")
            return

        if decoded_path == "/stale-oa-assets-22663018.tgz":
            send_text(self, 404, "not found", "text/plain")
            return

        if decoded_path == "/" and query.get("id") in (["PMC123459"], ["PMC123462"], ["PMC123463"], ["PMC123464"], ["PMC123465"]):
            append_request_log("fulltext:xml:pmc-oa-archive")
            send_text(self, 200, "<records></records>", "application/xml")
            return

        if decoded_path == "/" and query.get("id") in (["PMC123457"], ["PMC123458"]):
            send_text(self, 200, "<records></records>", "application/xml")
            return

        if decoded_path == "/articles/PMC123457/":
            send_text(self, 200, HTML_FALLBACK, "text/html; charset=utf-8")
            return

        if decoded_path == "/articles/PMC123458/":
            send_text(self, 404, "not found", "text/plain")
            return

        if decoded_path in {"/articles/PMC123459/", "/articles/PMC123462/", "/articles/PMC123463/"}:
            append_request_log("fulltext:html:pmc")
            send_text(self, 404, "not found", "text/plain")
            return

        if decoded_path == "/articles/PMC123464/":
            send_text(self, 200, METADATA_ONLY_HTML, "text/html; charset=utf-8")
            return

        if decoded_path == "/articles/PMC123465/":
            send_cacheable_text(self, 200, ABSTRACT_ONLY_HTML, "text/html; charset=utf-8")
            return

        if decoded_path == "/articles/PMC123460/":
            send_text(self, 404, "not found", "text/plain")
            return

        if decoded_path.startswith("/graph/v1/paper/PMID:"):
            pmid = decoded_path.rsplit(":", 1)[-1]
            article = ARTICLES.get(pmid)
            if article is None:
                send_json(self, 404, {"error": "not found"})
                return
            payload = {
                "paperId": article["paper_id"],
                "title": article["title"],
            }
            if pmid == "22663013":
                payload["openAccessPdf"] = {
                    "url": f"http://127.0.0.1:{self.server.server_port}/pdf/22663013.pdf",
                    "status": "GREEN",
                    "license": "CC BY",
                }
            if pmid == "22663014":
                payload["openAccessPdf"] = {
                    "url": f"http://127.0.0.1:{self.server.server_port}/pdf/22663014.pdf",
                    "status": "GREEN",
                    "license": "CC BY",
                }
            if pmid == "22663015":
                payload["openAccessPdf"] = {
                    "url": "https://aacr.figshare.com/articles/journal_contribution/Fixture_Figshare_supplement/22474820?file=39926318",
                    "status": "GREEN",
                    "license": "CC BY 4.0",
                }
            if pmid == "22663017":
                payload["openAccessPdf"] = {
                    "url": "https://aacr.figshare.com/articles/journal_contribution/Fixture_Figshare_cold_storage/22474830?file=39926330",
                    "status": "GREEN",
                    "license": "CC BY 4.0",
                }
            if pmid == "22663020":
                payload["openAccessPdf"] = {
                    "url": f"http://127.0.0.1:{self.server.server_port}/pdf/22663020.pdf?token=secret",
                    "status": "GREEN",
                    "license": "CC BY",
                }
            if pmid == "22663022":
                payload["openAccessPdf"] = {
                    "url": f"http://127.0.0.1:{self.server.server_port}/pdf/22663022.pdf?token=secret",
                    "status": "GREEN",
                    "license": "CC BY",
                }
            send_json(self, 200, payload)
            return

        if decoded_path == "/v2/articles/22474820":
            send_json(self, 200, {
                "id": 22474820,
                "title": "Figshare asset fallback winner",
                "doi": "10.1158/fixture.figshare",
                "url_public_html": "https://aacr.figshare.com/articles/journal_contribution/Fixture_Figshare_supplement/22474820",
                "url_api": f"http://127.0.0.1:{self.server.server_port}/v2/articles/22474820",
                "license": {
                    "name": "CC BY 4.0",
                    "url": "https://creativecommons.org/licenses/by/4.0/",
                },
                "files": [
                    {
                        "id": 39926318,
                        "name": "figshare-supplement.pdf",
                        "size": len(FIGSHARE_SUPPLEMENT),
                        "md5": "0123456789abcdef0123456789abcdef",
                        "mimetype": "application/pdf",
                        "download_url": f"http://127.0.0.1:{self.server.server_port}/figshare/files/39926318/figshare-supplement.pdf",
                    }
                ],
            })
            return

        if decoded_path == "/v2/articles/22474817":
            send_json(self, 200, {
                "id": 22474817,
                "title": "Supplementary Table S1 from Figshare asset fallback winner",
                "doi": "10.1158/1078-0432.22474817.v1",
                "url_public_html": "https://aacr.figshare.com/articles/dataset/Supplementary_Table_S1_from_Figshare_asset_fallback_winner/22474817",
                "url_api": f"http://127.0.0.1:{self.server.server_port}/v2/articles/22474817",
                "license": {"name": "CC BY 4.0"},
                "files": [
                    {
                        "id": 39926317,
                        "name": "supplementary-table-s1.xlsx",
                        "size": len(FIGSHARE_TABLE_S1),
                        "mimetype": "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                        "download_url": f"http://127.0.0.1:{self.server.server_port}/figshare/files/39926317/supplementary-table-s1.xlsx",
                    }
                ],
            })
            return

        if decoded_path == "/v2/articles/22474818":
            send_json(self, 200, {
                "id": 22474818,
                "title": "Supplementary Data S2 from Figshare asset fallback winner",
                "doi": "10.1158/1078-0432.22474818.v1",
                "url_public_html": "https://aacr.figshare.com/articles/dataset/Supplementary_Data_S2_from_Figshare_asset_fallback_winner/22474818",
                "url_api": f"http://127.0.0.1:{self.server.server_port}/v2/articles/22474818",
                "license": {"name": "CC BY 4.0"},
                "files": [
                    {
                        "id": 39926316,
                        "name": "supplementary-table-s2.xlsx",
                        "size": len(FIGSHARE_TABLE_S2),
                        "mimetype": "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                        "download_url": f"http://127.0.0.1:{self.server.server_port}/figshare/files/39926316/supplementary-table-s2.xlsx",
                    }
                ],
            })
            return

        if decoded_path == "/v2/articles/22474830":
            with COLD_STORAGE_LOCK:
                COLD_STORAGE_HITS.pop("/figshare/files/39926330/cold-storage-supplement.pdf", None)
            send_json(self, 200, {
                "id": 22474830,
                "title": "Figshare cold storage asset winner",
                "doi": "10.1158/fixture.figshare-cold",
                "url_public_html": "https://aacr.figshare.com/articles/journal_contribution/Fixture_Figshare_cold_storage/22474830",
                "url_api": f"http://127.0.0.1:{self.server.server_port}/v2/articles/22474830",
                "license": {"name": "CC BY 4.0"},
                "files": [
                    {
                        "id": 39926330,
                        "name": "cold-storage-supplement.pdf",
                        "size": len(FIGSHARE_COLD_STORAGE),
                        "mimetype": "application/pdf",
                        "download_url": f"http://127.0.0.1:{self.server.server_port}/figshare/files/39926330/cold-storage-supplement.pdf",
                    }
                ],
            })
            return

        if decoded_path == "/v2/articles/99999999":
            send_json(self, 200, {
                "id": 99999999,
                "title": "Unrelated figshare supplement",
                "doi": "10.1158/unrelated.fixture",
                "url_public_html": "https://figshare.com/articles/dataset/Unrelated/99999999",
                "url_api": f"http://127.0.0.1:{self.server.server_port}/v2/articles/99999999",
                "license": {"name": "CC BY 4.0"},
                "files": [
                    {
                        "id": 39926999,
                        "name": "unrelated-table.xlsx",
                        "size": len(FIGSHARE_UNRELATED_TABLE),
                        "mimetype": "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                        "download_url": f"http://127.0.0.1:{self.server.server_port}/figshare/files/39926999/unrelated-table.xlsx",
                    }
                ],
            })
            return

        if decoded_path == "/figshare/files/39926318/figshare-supplement.pdf":
            send_bytes(self, 200, FIGSHARE_SUPPLEMENT, "application/pdf")
            return

        if decoded_path == "/figshare/files/39926317/supplementary-table-s1.xlsx":
            send_bytes(self, 200, FIGSHARE_TABLE_S1, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
            return

        if decoded_path == "/figshare/files/39926316/supplementary-table-s2.xlsx":
            send_bytes(self, 200, FIGSHARE_TABLE_S2, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
            return

        if decoded_path == "/figshare/files/39926999/unrelated-table.xlsx":
            send_bytes(self, 200, FIGSHARE_UNRELATED_TABLE, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
            return

        if decoded_path == "/figshare/files/39926330/cold-storage-supplement.pdf":
            with COLD_STORAGE_LOCK:
                hits = COLD_STORAGE_HITS.get(decoded_path, 0)
                COLD_STORAGE_HITS[decoded_path] = hits + 1
            if hits == 0:
                send_bytes(self, 202, b"", "application/octet-stream")
            else:
                send_bytes(self, 200, FIGSHARE_COLD_STORAGE, "application/pdf")
            return

        if decoded_path == "/pdf/22663013.pdf":
            send_bytes(self, 200, PDF_FALLBACK, "application/pdf")
            return

        if decoded_path == "/pdf/22663014.pdf":
            append_request_log("fulltext:pdf:semantic-scholar")
            send_text(self, 404, "not found", "text/plain")
            return

        if decoded_path in {"/pdf/22663020.pdf", "/pdf/22663022.pdf"}:
            send_bytes(self, 200, PDF_FALLBACK, "application/pdf")
            return

        if decoded_path == "/efetch.fcgi":
            if (
                query.get("db") == ["pubmed"]
                and query.get("retmode") == ["xml"]
                and query.get("id") == ["22663011"]
            ):
                append_request_log("indexing:xml:pubmed-efetch")
                send_text(self, 200, PUBMED_INDEXING_XML, "application/xml")
                return
            if query.get("id") in (["123459"], ["123462"], ["123463"], ["123464"], ["123465"]):
                append_request_log("fulltext:xml:ncbi-efetch-pmc")
                send_text(self, 200, "", "application/xml")
                return
            send_text(self, 404, "not found", "text/plain")
            return

        send_json(self, 404, {"error": "not found"})

    def log_message(self, format, *args):
        return


ready_path = Path(sys.argv[1])
server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
ready_path.write_text(f"http://127.0.0.1:{server.server_port}\n", encoding="utf-8")
server.serve_forever()
PY
server_pid=$!
cleanup_failed_setup() {
  kill "$server_pid" 2>/dev/null || true
  rm -rf "$fixture_root"
  rm -f "$env_file"
}
trap cleanup_failed_setup EXIT

for _ in $(seq 1 50); do
  if [ -s "$ready_file" ]; then
    break
  fi
  if ! kill -0 "$server_pid" 2>/dev/null; then
    cat "$server_log" >&2
    exit 1
  fi
  sleep 0.1
done

test -s "$ready_file"
base_url="$(cat "$ready_file")"

printf 'export BIOMCP_TEST_UNPACED_ORIGIN=%q\n' "$base_url" >"$env_file"
printf 'export BIOMCP_PUBTATOR_BASE=%q\n' "$base_url" >>"$env_file"
printf 'export BIOMCP_EUROPEPMC_BASE=%q\n' "$base_url" >>"$env_file"
printf 'export BIOMCP_PUBMED_BASE=%q\n' "$base_url" >>"$env_file"
printf 'export BIOMCP_PMC_OA_BASE=%q\n' "$base_url" >>"$env_file"
printf 'export BIOMCP_PMC_HTML_BASE=%q\n' "$base_url" >>"$env_file"
printf 'export BIOMCP_NCBI_IDCONV_BASE=%q\n' "$base_url" >>"$env_file"
printf 'export BIOMCP_S2_BASE=%q\n' "$base_url" >>"$env_file"
printf 'export BIOMCP_FIGSHARE_BASE=%q\n' "$base_url" >>"$env_file"
printf 'export BIOMCP_OLS4_BASE=%q\n' "$base_url" >>"$env_file"
printf 'export BIOMCP_CACHE_MIN_DISK_FREE=1B\n' >>"$env_file"
printf 'unset NCBI_API_KEY\n' >>"$env_file"
printf 'unset S2_API_KEY\n' >>"$env_file"
printf 'export BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE_PID=%q\n' "$server_pid" >>"$env_file"
printf 'export BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE_ROOT=%q\n' "$fixture_root" >>"$env_file"
printf 'export BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE_READY_FILE=%q\n' "$ready_file" >>"$env_file"
printf 'export BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE_REQUEST_LOG=%q\n' "$request_log" >>"$env_file"

trap - EXIT
printf '%s\n' "$fixture_root"

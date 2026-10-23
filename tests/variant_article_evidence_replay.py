"""Replay public recorded article metadata through isolated provider endpoints."""

from contextlib import contextmanager
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import sys
import threading
from types import SimpleNamespace
from urllib.parse import parse_qs, urlsplit

sys.path.insert(0, str(Path(__file__).resolve().parent / "golden"))
from replay_env import REPO_ROOT, isolated_command_env  # noqa: E402

CAPTURE = REPO_ROOT / "tests/fixtures/article/evidence-ranking/akt1-e17k.json"


@contextmanager
def akt1_replay(tmp_path, cache=True):
    capture = json.loads(CAPTURE.read_text())
    rows = {row["pmid"]: row for row in capture["rows"]}
    for pmid in capture["pubmed_order"]:
        rows.setdefault(pmid, {"pmid": pmid, "title": "", "provenance": []})
    native = {}
    for source in ("pubmed", "europepmc", "semanticscholar"):
        positions = {}
        for row in rows.values():
            for fact in row["provenance"]:
                if fact["source"] == source:
                    positions[row["pmid"]] = min(positions.get(row["pmid"], 999999),
                                                  fact["native_position"])
        native[source] = sorted(positions, key=lambda pmid: (positions[pmid], pmid))
    requests = []
    misses = []
    hit = {"_id": "chr14:g.105246551C>T", "dbnsfp": {
        "genename": "AKT1", "hgvsp": "p.Glu17Lys", "hgvsc": "c.49G>A",
        "rsid": "rs121434592"}}

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self):  # noqa: N802
            target = urlsplit(self.path)
            query = parse_qs(target.query)
            requests.append((target.path, query))
            path = target.path
            if path == "/v1/query":
                body = {"hits": [hit], "total": 1}
            elif path.startswith("/v1/variant/"):
                body = hit
            elif path == "/query":
                body = {"hits": [{"symbol": "AKT1", "entrezgene": 207}], "total": 1}
            elif path == "/allele":
                body = {"errorType": "NotFound", "description": "Replay control"}
            elif path in ("/entity/autocomplete/", "/variant/autocomplete/"):
                body = []
            elif path == "/search/":
                body = {"results": [], "count": 0, "total_pages": 0, "current": 1}
            elif path.endswith("/esearch.fcgi"):
                term = query.get("term", [""])[0]
                ids = capture["pubmed_order"] if term == "AKT1 E17K" else native["pubmed"]
                offset = int(query.get("retstart", [0])[0])
                limit = int(query.get("retmax", [len(ids)])[0])
                body = {"esearchresult": {"count": str(len(ids)),
                        "retstart": str(offset), "idlist": ids[offset:offset + limit]}}
            elif path.endswith("/esummary.fcgi"):
                ids = query.get("id", [""])[0].split(",")
                body = {"result": {"uids": ids, **{
                    pmid: {"uid": pmid, "title": rows[pmid]["title"], "articleids": []}
                    for pmid in ids if pmid in rows}}}
            elif path == "/search":
                term = query.get("query", [""])[0]
                if "EXT_ID:" in term:
                    ids = [pmid for pmid in rows if pmid in term]
                else:
                    ids = native["europepmc"]
                page = int(query.get("page", [1])[0])
                limit = int(query.get("pageSize", [1000])[0])
                body = {"hitCount": len(ids), "resultList": {"result": [
                    {"id": pmid, "pmid": pmid, "source": "MED", "title": rows[pmid]["title"],
                     "abstractText": rows[pmid].get("abstract", "")}
                    for pmid in ids[(page - 1) * limit:page * limit]]}}
            elif path in ("/graph/v1/paper/search", "/graph/v1/paper/search/bulk"):
                ids = native["semanticscholar"]
                offset = int(query.get("offset", [0])[0])
                limit = int(query.get("limit", [100])[0])
                body = {"total": len(ids), "data": [
                    {"paperId": "recorded-" + pmid, "externalIds": {"PubMed": pmid},
                     "title": rows[pmid]["title"], "abstract": rows[pmid].get("abstract")}
                    for pmid in ids[offset:offset + limit]]}
            elif path.endswith("/efetch.fcgi"):
                import xml.etree.ElementTree as ET

                root = ET.Element("PubmedArticleSet")
                for pmid in query.get("id", [""])[0].split(","):
                    if pmid not in rows:
                        continue
                    citation = ET.SubElement(ET.SubElement(root, "PubmedArticle"), "MedlineCitation")
                    ET.SubElement(citation, "PMID").text = pmid
                    article = ET.SubElement(citation, "Article")
                    ET.SubElement(article, "ArticleTitle").text = rows[pmid]["title"]
                    ET.SubElement(ET.SubElement(article, "Abstract"), "AbstractText").text = rows[pmid].get("abstract", "")
                self.reply(ET.tostring(root), "application/xml")
                return
            elif path == "/publications/export/biocjson":
                body = []
            else:
                misses.append(self.path)
                self.send_error(404, "Replay miss")
                return
            self.reply(json.dumps(body).encode(), "application/json")

        def reply(self, body, content_type):
            self.send_response(200)
            self.send_header("Content-Type", content_type)
            self.send_header("Cache-Control", "no-store")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def do_POST(self):  # noqa: N802
            length = int(self.headers.get("Content-Length", 0))
            content = self.rfile.read(length).decode()
            self.path += ("&" if "?" in self.path else "?") + content
            self.do_GET()

        def log_message(self, *_args):
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    origin = f"http://127.0.0.1:{server.server_port}"
    providers = {key: origin for key in (
        "BIOMCP_EUROPEPMC_BASE", "BIOMCP_PUBTATOR_BASE", "BIOMCP_S2_BASE",
        "BIOMCP_LITVAR_BASE", "BIOMCP_CLINGEN_CAR_BASE", "BIOMCP_MYGENE_BASE",
    )}
    providers.update(BIOMCP_MYVARIANT_BASE=origin + "/v1",
                     BIOMCP_PUBMED_BASE=origin + "/entrez/eutils")
    env = isolated_command_env(tmp_path, providers, origin)
    env["BIOMCP_DISABLE_SOURCES"] = "pubtator3,litvar2,mygene-info"
    if cache:
        env["BIOMCP_CACHE_ENABLED"] = "true"
    try:
        yield SimpleNamespace(env=env, requests=requests, capture=capture)
    finally:
        server.shutdown()
        server.server_close()
        thread.join()
        assert not misses, misses

# Live Article Supplement Assets

This operator-run contract checks that article-document supplement discovery
still works against real NCBI/PMC representations. It is intentionally outside
the routine fixture lane because upstream document availability and access
policy can change independently of BioMCP releases.

## PMID 20516115 linked supplements

This article names a PDF and a workbook in NCBI JATS and PMC HTML. Each named
file must remain visible with both document routes. The complete bounded views
share one cache: a file is either retrievable, or it remains named as PMC
proof-of-work or transient source-unavailable coverage without a handle. A
generic package miss, partial page, or file appearing in both states is not
sufficient.

```bash
scratch="$(mktemp -d)"
trap 'rm -rf "$scratch"' EXIT
asset_cache_dir="$scratch/cache"
BIOMCP_CACHE_DIR="$asset_cache_dir" ../../tools/biomcp-ci --json get article 20516115 --asset-view coverage --asset-limit 100 --asset-offset 0 assets >"$scratch/coverage.json"
BIOMCP_CACHE_DIR="$asset_cache_dir" ../../tools/biomcp-ci --json get article 20516115 --asset-view retrievable --asset-limit 100 --asset-offset 0 assets >"$scratch/retrievable.json"
env -i PATH="$PATH" python3 /dev/fd/3 "$scratch/coverage.json" "$scratch/retrievable.json" 3<<'PY' | mustmatch like "live article asset pages are complete and mutually exclusive"
import json
import sys


def fail(message):
    raise SystemExit(message)


def load(path):
    with open(path, encoding="utf-8") as stream:
        value = json.load(stream)
    if not isinstance(value, dict):
        fail(f"{path} must contain a JSON object")
    return value


def complete_rows(document, key):
    rows = document.get(key)
    if not isinstance(rows, list) or not all(isinstance(row, dict) for row in rows):
        fail(f"{key} must be an array of objects")
    for row in rows:
        if not isinstance(row.get("filename"), str) or not row["filename"]:
            fail(f"{key} contains a malformed filename")
    page = document.get("pagination")
    if not isinstance(page, dict):
        fail(f"{key} pagination must be an object")
    returned = page.get("returned")
    total = page.get("total")
    if type(returned) is not int or type(total) is not int:
        fail(f"{key} pagination counts must be integers")
    if returned != len(rows) or total != returned:
        fail(f"{key} view is not complete")
    if page.get("has_more") is not False:
        fail(f"{key} view unexpectedly has another page")
    if page.get("next_offset") is not None or page.get("continuation_command") is not None:
        fail(f"{key} view unexpectedly advertises continuation")
    return rows


def require_document_routes(row):
    routes = row.get("discovery_routes")
    if not isinstance(routes, list):
        fail("named file discovery_routes must be an array")
    documents = set()
    for route in routes:
        if not isinstance(route, dict):
            fail("named file has a malformed discovery route")
        provider = route.get("provider")
        if not isinstance(provider, dict):
            fail("named file route has a malformed provider")
        if not all(
            isinstance(provider.get(field), str) and provider[field].strip()
            for field in ("label", "source")
        ):
            fail("named file route lacks provider labels")
        source_document = route.get("source_document")
        if not isinstance(source_document, str):
            fail("named file route has a malformed source document")
        documents.add(source_document)
    if not {"jats_xml", "pmc_html"}.issubset(documents):
        fail("named file must be discovered in both JATS XML and PMC HTML")


coverage_document = load(sys.argv[1])
retrievable_document = load(sys.argv[2])
if "assets" in coverage_document or "coverage" in retrievable_document:
    fail("asset views must remain separate")
coverage = complete_rows(coverage_document, "coverage")
assets = complete_rows(retrievable_document, "assets")

ignored = {"coverage", "assets", "pagination", "_meta"}
coverage_manifest = {
    key: value for key, value in coverage_document.items() if key not in ignored
}
retrievable_manifest = {
    key: value for key, value in retrievable_document.items() if key not in ignored
}
if coverage_manifest != retrievable_manifest:
    fail("asset views do not describe the same cached manifest")
if coverage_document.get("pmid") != "20516115":
    fail("asset views returned the wrong PMID")

for suffix in (
    "Supplementary_Methods__Figures__Tables.pdf",
    "Supplementary_Tables.xls",
):
    coverage_matches = [row for row in coverage if row["filename"].endswith(suffix)]
    asset_matches = [row for row in assets if row["filename"].endswith(suffix)]
    if (len(coverage_matches), len(asset_matches)) not in {(1, 0), (0, 1)}:
        fail(f"{suffix} must appear in exactly one asset state")
    if asset_matches:
        require_document_routes(asset_matches[0])
        continue
    row = coverage_matches[0]
    require_document_routes(row)
    if row.get("outcome") not in {"pmc_proof_of_work", "source_unavailable"}:
        fail(f"{suffix} has an unacceptable coverage outcome")
    if row.get("handle") is not None:
        fail(f"{suffix} must not advertise a handle without retrievable bytes")

print("live article asset pages are complete and mutually exclusive")
PY
```

# Article Queries

Article workflows mix typed biomedical anchors with broader keyword discovery.
These canaries keep the blocking lane honest about search structure, annotation
paths, and fulltext fallback behavior without depending on optional API keys.

## Article Request Planning Happens Before Federated Search

Article search normalizes CLI flags into a request-command seam before any
federated article backend executes. The request records filters, source, sort,
ranking, exact-keyword lookup intent, and the pre-execution `BackendPlan`, so
tests can prove routing decisions without depending on live PubMed, Europe PMC,
PubTator, LitSense2, or Semantic Scholar responses.

## Deterministic Source Contracts

Ticket 376 moves routine article-source proof from public upstream canaries to
source-local request-plan and fixture-backed contracts. Any irreducible public
availability check belongs in an explicit release/live-smoke lane; routine specs
must instead prove PubMed, Europe PMC, PubTator, LitSense2, and Semantic Scholar
request shape, status mapping, and redacted auth behavior locally.

## Default Article Source Plan Excludes LitSense2

`--source all` should keep the default federated source set to PubTator3,
Europe PMC, PubMed, and compatible Semantic Scholar. LitSense2 remains
individually selectable for callers who explicitly ask for it.

## Semantic Scholar Is Individually Selectable

`--source semanticscholar` should use the same Semantic Scholar search client as
federation, while keeping the returned rows attributable to Semantic Scholar.
The fixture points only the Semantic Scholar base URL at a local handler, so the
contract fails before the source value is accepted and passes without touching
other article backends.

```bash
bash ../fixtures/run-article-semanticscholar-source-search.sh ../.. \
  | mustmatch like '"semantic_scholar_enabled": true
"source": "semanticscholar"
"Semantic Scholar selectable source fixture"
"planner=semanticscholar_only"'
```

## Federated Article Search Bounds Slow Sources

When one article source is slow, the default federated search should still return
bounded results from the healthy sources and say which source degraded. The
fixture points every article source base URL at local HTTP handlers: PubTator3,
PubMed, Semantic Scholar, and LitSense2 respond quickly, while Europe PMC holds
its response long enough to prove the per-source timeout contract.

```bash
bash ../fixtures/run-article-federated-timeout-search.sh ../.. \
  | mustmatch like '"source": "europepmc"
"status": "degraded"
timed out
BRAF melanoma bounded federation fixture'
```

## Deterministic Renderer Envelope Contracts

Ticket 377 moves routine article renderer/envelope proof into fixture-result
contracts. The deterministic tests should cover article JSON `_meta.next_commands`,
`_meta.source_status`, source degradation guidance, and markdown result-table
anchors without live PubMed, Europe PMC, PubTator, LitSense2, or Semantic Scholar
calls.

## Explicit Fixtures Do Not Inherit Live-Source Pacing
<!-- mustmatch-lint: skip -->

A fixture-backed full-text request can traverse its normal multi-source resolver
without waiting between requests as though it were calling live providers. The
five-second bound still requires a successful Europe PMC XML result, so a fast
error or empty response cannot satisfy the contract.

```bash run id=unpaced-fixture-fulltext timeout=5 exit=0
../../tools/biomcp-ci get article 22663011 fulltext
```

```text expect=unpaced-fixture-fulltext contains
## Full Text (Europe PMC XML)
```

## MYD88 Protein-Alias Article Precision

<!-- mustmatch-lint: skip -->

Exact gene plus protein-alias literature searches should preserve both anchors
before ranking so a clinically specific alias does not drift into generic MYD88
papers. The deterministic Rust contract uses fixture rows rather than live
PubMed or LitSense2 ranking, because BioMCP owns query planning and local
relevance scoring but not upstream result order.

## Gene Search

Gene-linked article search should still read like a literature intake surface:
clear heading, ranking note, and a PMID-first table.

## Keyword Search

Keyword search is a different planning path from typed gene search. The query
echo and source-aware table should make that distinction visible.

## Search Table & Source Ranking

The JSON contract should preserve the top article follow-up and keep per-result
source identity plus ranking metadata available to automation.

## PubTator Annotations

Annotations remain a first-class deepen path. The section should keep the
PubTator heading and explain that the extracted entities are normalized.

## Full-Text HTML Fallback

When the XML ladder misses, BioMCP should fall back to the PMC HTML article page
and still keep the saved-file contract on stdout.

```bash
rm -rf ../../.cache/biomcp-specs/downloads
mkdir -p ../../.cache/biomcp-specs/downloads
../../tools/biomcp-ci get article 22663012 fulltext | mustmatch like '## Full Text (PMC HTML)
...'
rg -l 'PMC HTML fallback body text' ../../.cache/biomcp-specs/downloads >/dev/null
```

## PDF Fallback Is Opt-In

Semantic Scholar PDF is a last resort, not the default resolver order. The same
fixture-backed article should fail cleanly without `--pdf` and succeed with it.

```bash
../../tools/biomcp-ci get article 22663013 fulltext | mustmatch like "XML and HTML sources did not return full text"
../../tools/biomcp-ci get article 22663013 fulltext | mustmatch not like "Semantic Scholar PDF"
rm -rf ../../.cache/biomcp-specs/downloads
mkdir -p ../../.cache/biomcp-specs/downloads
../../tools/biomcp-ci get article 22663013 fulltext --pdf | mustmatch like '## Full Text (Semantic Scholar PDF)
...'
test "$(find ../../.cache/biomcp-specs/downloads -maxdepth 1 -type f -name '*.txt' | wc -l)" -ge 1
```

## JATS Converter Keeps Evidence-Carrying Floats, Supplements, and Complex Table Markers

Saved Markdown should surface evidence-bearing JATS content that is already
present in the XML. Figures in the body and floats group, declared supplement
files, and unflattened merged-cell tables must be visible to an agent reading
the saved article.

```bash
bash ../fixtures/render-article-fulltext-jats-markdown.sh ../.. | mustmatch like "Europe PMC body text with callout (Figure 2) and B-RAF^V600E^. PLX4032 boundary text."
```

```bash
bash ../fixtures/render-article-fulltext-jats-markdown.sh ../.. | mustmatch like "> **Figure 1.** Inline figure caption preserves n=10 cell counts."
```

```bash
bash ../fixtures/render-article-fulltext-jats-markdown.sh ../.. | mustmatch like "> **Figure 2.** Floats-group figure reports measurement bar is 70 μm."
```

```bash
bash ../fixtures/render-article-fulltext-jats-markdown.sh ../.. | mustmatch like "Supplementary Data S1"
```

```bash
bash ../fixtures/render-article-fulltext-jats-markdown.sh ../.. | mustmatch like "Measurement traces for the treatment cohort."
```

```bash
bash ../fixtures/render-article-fulltext-jats-markdown.sh ../.. | mustmatch like "traces-s1.csv"
```

```bash
bash ../fixtures/render-article-fulltext-jats-markdown.sh ../.. | mustmatch like "**Table 2.** Merged treatment table."
```

```bash
bash ../fixtures/render-article-fulltext-jats-markdown.sh ../.. | mustmatch like "*[complex table omitted: 2×3, merged cells]*"
```

```bash
bash ../fixtures/render-article-fulltext-jats-markdown.sh ../.. | mustmatch not like "((Figure 2))"
```

## Fulltext Provenance, Reuse, and Quality Metadata

Saved fulltext Markdown is evidence material, so the JSON response must carry a
machine-readable manifest for the artifact. The manifest identifies the source,
records whether the representation has useful structure, and separates known
license context from unknown reuse state.

```bash
jats_json="$(../../tools/biomcp-ci --json get article 22663011 fulltext)"
mustmatch like '"full_text_source"' <<<"$jats_json"
ARTICLE_JSON="$jats_json" uv run --no-sync python3 - <<'PY'
import json, os
doc = json.loads(os.environ["ARTICLE_JSON"])
manifest = doc.get("full_text_manifest") or {}
assert manifest.get("source_kind") == "jats_xml", "missing JATS full_text_manifest source_kind"
assert manifest.get("source_identifier") == "PMC123456"
provider = manifest.get("provider") or {}
assert provider.get("label") == "Europe PMC XML"
assert provider.get("source") == "Europe PMC"
quality = manifest.get("quality") or {}
assert quality.get("has_sections") is True
assert quality.get("has_tables") is True
assert quality.get("has_references") is True
assert quality.get("has_fulltext_signal") is True
assert quality.get("has_entity_annotations") is False
provenance = manifest.get("provenance") or {}
assert provenance.get("open_access") is True
reuse = manifest.get("reuse") or {}
assert reuse.get("license_present") is True
assert "CC BY" in str(reuse.get("license", ""))
PY
```

PMC HTML fallback can still provide useful readable Markdown, but it is weaker
than source XML and can lack article-level license context. Unknown reuse state
must stay explicit instead of serializing as a safe or blank license.

```bash
html_json="$(../../tools/biomcp-ci --json get article 22663012 fulltext)"
mustmatch like '"full_text_source"' <<<"$html_json"
ARTICLE_JSON="$html_json" uv run --no-sync python3 - <<'PY'
import json, os
doc = json.loads(os.environ["ARTICLE_JSON"])
manifest = doc.get("full_text_manifest") or {}
assert manifest.get("source_kind") == "pmc_html", "missing PMC HTML full_text_manifest source_kind"
assert manifest.get("source_identifier") == "PMC123457"
provider = manifest.get("provider") or {}
assert provider.get("label") == "PMC HTML"
assert provider.get("source") == "PMC"
quality = manifest.get("quality") or {}
assert quality.get("has_fulltext_signal") is True
assert quality.get("has_entity_annotations") is False
provenance = manifest.get("provenance") or {}
assert provenance.get("open_access") is True
reuse = manifest.get("reuse") or {}
assert reuse.get("license_present") is False
assert not reuse.get("license")
warning = str(reuse.get("reuse_warning", "")).lower()
assert "license" in warning or "reuse" in warning
PY
```

PDF remains an opt-in fallback. The manifest must mark PDF-derived fulltext so an
agent can decide whether PDF conversion is adequate for evidence ingestion and
can carry any license fact returned by Semantic Scholar.

```bash
pdf_json="$(../../tools/biomcp-ci --json get article 22663013 fulltext --pdf)"
mustmatch like '"full_text_source"' <<<"$pdf_json"
ARTICLE_JSON="$pdf_json" uv run --no-sync python3 - <<'PY'
import json, os
doc = json.loads(os.environ["ARTICLE_JSON"])
manifest = doc.get("full_text_manifest") or {}
assert manifest.get("source_kind") == "pdf", "missing PDF full_text_manifest source_kind"
assert "/pdf/22663013.pdf" in str(manifest.get("source_identifier", ""))
provider = manifest.get("provider") or {}
assert provider.get("label") == "Semantic Scholar PDF"
assert provider.get("source") == "Semantic Scholar"
quality = manifest.get("quality") or {}
assert quality.get("has_fulltext_signal") is True
provenance = manifest.get("provenance") or {}
assert provenance.get("pdf_fallback_used") is True
reuse = manifest.get("reuse") or {}
assert reuse.get("license_present") is True
assert "CC BY" in str(reuse.get("license", ""))
PY
```

## PMC OA Archive XML Fulltext Manifest Carries Source and Reuse Fields

When the XML ladder falls through to the PMC OA Archive package, the JSON
manifest should identify that package-backed source instead of collapsing it
into a generic XML winner. The package URL, license, and retraction fact are
machine-readable provenance for downstream reuse decisions.

```bash
../../tools/biomcp-ci --json get article 22663016 fulltext | uv run --no-sync python3 -c '
import json, sys

doc = json.load(sys.stdin)
manifest = doc.get("full_text_manifest") or {}
assert manifest.get("source_kind") == "jats_xml"
assert manifest.get("source_identifier") == "PMC123460"
provider = manifest.get("provider") or {}
assert provider.get("label") == "PMC OA Archive XML"
assert provider.get("source") == "PMC OA"
provenance = manifest.get("provenance") or {}
assert provenance.get("open_access") is True
assert provenance.get("retracted") is False
assert "oa-assets-22663016.tgz" in str(provenance.get("package_url", ""))
reuse = manifest.get("reuse") or {}
assert reuse.get("license_present") is True
assert "CC BY-NC" in str(reuse.get("license", ""))
print("pmc oa archive fulltext manifest ok")
' | mustmatch like "pmc oa archive fulltext manifest ok"
```

## OA Package Assets Manifest

Article assets are resolved from the canonical PMC OA package on demand, even
when another XML rung supplied the saved full text. The JSON-only manifest keeps
byte-level grounding and retrieval handles for downstream converters without
parsing or inlining the assets.

```bash
../../tools/biomcp-ci --json get article 22663011 assets | uv run --no-sync python3 -c '
import json, re, sys

doc = json.load(sys.stdin)
assets = {row.get("filename"): row for row in doc.get("assets") or []}
fig = assets.get("figure-floats.png") or {}
inline_fig = assets.get("figure-inline.png") or {}
supp = assets.get("traces-s1.csv") or {}
other = assets.get("readme.txt") or {}
assert fig.get("kind") == "figure-image"
assert inline_fig.get("kind") == "figure-image"
assert supp.get("kind") == "supplementary-file"
assert other.get("kind") == "other"
assert isinstance(fig.get("size_bytes"), int) and fig["size_bytes"] > 0
assert re.fullmatch(r"[0-9a-f]{64}", str(fig.get("sha256", "")))
assert supp.get("size_bytes") == len(b"time,value\n0,1\n")
assert supp.get("sha256") == "7e31a103261f1075aa93cfa4da9d83479724c9fa9ed0aff644e26795a5038841"
provider = fig.get("provider") or {}
assert provider.get("label") == "PMC OA Archive"
assert provider.get("source") == "PMC OA"
reuse = fig.get("reuse") or {}
assert reuse.get("license_present") is True
assert "CC BY" in str(reuse.get("license", ""))
provenance = fig.get("provenance") or {}
assert provenance.get("retracted") is False
assert "oa-assets-22663011.tgz" in str(provenance.get("package_url", ""))
jats = fig.get("jats") or {}
assert jats.get("label") == "Figure 2"
assert "measurement bar" in str(jats.get("caption", ""))
supp_jats = supp.get("jats") or {}
assert supp_jats.get("label") == "Supplementary Data S1"
assert "Measurement traces" in str(supp_jats.get("caption", ""))
assert supp.get("handle") == "biomcp get article 22663011 asset traces-s1.csv"
commands = (doc.get("_meta") or {}).get("next_commands") or []
assert "biomcp get article 22663011 asset traces-s1.csv" in commands
print("article assets manifest ok")
' | mustmatch like "article assets manifest ok"
```

## OA Package Asset Retrieval Returns Bytes

The retrieval handle returns the selected archive member bytes as-is. BioMCP is
the canonical fetcher here; conversion of CSV, XLSX, DOC, PDF, or image assets
belongs downstream.

```bash
../../tools/biomcp-ci get article 22663011 asset traces-s1.csv | mustmatch like "time,value
0,1"
```

## Non-PMC Figshare Assets Manifest

When an article has no PMC OA package but Semantic Scholar points at a supported
AACR/Figshare article, the same asset manifest surface should return a
provider-labelled Figshare manifest. The handle remains a BioMCP command, not a
transient provider URL, so downstream tools can retrieve bytes through one stable
article-asset grammar.

```bash
../../tools/biomcp-ci --json get article 22663015 assets | uv run --no-sync python3 -c '
import json, re, sys

raw = sys.stdin.read()
try:
    doc = json.loads(raw)
except Exception:
    print("figshare article assets manifest missing")
    raise SystemExit(0)

assets = {row.get("filename"): row for row in doc.get("assets") or []}
supp = assets.get("figshare-supplement.pdf") or {}
provider = doc.get("provider") or {}
asset_provider = supp.get("provider") or {}
reuse = supp.get("reuse") or {}
provenance = supp.get("provenance") or {}
commands = (doc.get("_meta") or {}).get("next_commands") or []

ok = True
ok = ok and ("pmcid" not in doc or doc.get("pmcid") in (None, ""))
ok = ok and provider.get("label") == "Figshare"
ok = ok and provider.get("source") == "Figshare"
ok = ok and supp.get("kind") == "supplementary-file"
ok = ok and isinstance(supp.get("size_bytes"), int) and supp.get("size_bytes") > 0
ok = ok and re.fullmatch(r"[0-9a-f]{64}", str(supp.get("sha256", ""))) is not None
ok = ok and asset_provider.get("label") == "Figshare"
ok = ok and asset_provider.get("source") == "Figshare"
ok = ok and reuse.get("license_present") is True
ok = ok and "CC BY" in str(reuse.get("license", ""))
ok = ok and "figshare" in str(provenance.get("package_url", "")).lower()
ok = ok and supp.get("handle") == "biomcp get article 22663015 asset figshare-supplement.pdf"
ok = ok and "biomcp get article 22663015 asset figshare-supplement.pdf" in commands

print("figshare article assets manifest ok" if ok else "figshare article assets manifest missing")
' | mustmatch like "figshare article assets manifest ok"
```

## Non-PMC Figshare Asset Retrieval Returns Bytes

The Figshare asset handle should re-resolve provider metadata and stream the
current file bytes without conversion. A supplemental PDF remains an asset, not a
fulltext substitute or parsed text source.

```bash
../../tools/biomcp-ci get article 22663015 asset figshare-supplement.pdf | mustmatch like "%PDF-1.4
Figshare supplemental fixture bytes"
```

## Non-PMC Figshare Assets Manifest Includes Same-Paper Sibling Records

AACR/Figshare supplements can split a paper across one linked contribution and
separate sibling records for individual tables. The article asset manifest should
merge same-paper sibling files into the stable BioMCP handle list so downstream
agents do not have to rediscover provider records themselves.

```bash
../../tools/biomcp-ci --json get article 22663015 assets | uv run --no-sync python3 -c '
import json, sys

doc = json.load(sys.stdin)
assets = {row.get("filename"): row for row in doc.get("assets") or []}
commands = (doc.get("_meta") or {}).get("next_commands") or []
for filename in ["figshare-supplement.pdf", "supplementary-table-s1.xlsx", "supplementary-table-s2.xlsx"]:
    row = assets.get(filename) or {}
    assert row.get("kind") == "supplementary-file", filename
    assert isinstance(row.get("size_bytes"), int) and row["size_bytes"] > 0, filename
    provider = row.get("provider") or {}
    assert provider.get("label") == "Figshare", filename
    assert provider.get("source") == "Figshare", filename
    handle = f"biomcp get article 22663015 asset {filename}"
    assert row.get("handle") == handle, filename
    assert handle in commands, filename
assert "unrelated-table.xlsx" not in assets
print("figshare sibling assets manifest ok")
' | mustmatch like "figshare sibling assets manifest ok"
```

## Non-PMC Figshare Sibling Asset Retrieval Returns Bytes

Every handle listed in the Figshare manifest should be fetchable through BioMCP.
Sibling table bytes remain raw provider bytes; BioMCP does not parse workbook
contents.

```bash
../../tools/biomcp-ci get article 22663015 asset supplementary-table-s1.xlsx | mustmatch like "S1 workbook fixture bytes"
```

## Figshare Cold-Storage Asset Retrieval Retries Accepted Downloads

Figshare can answer a download request with `202 Accepted` while a cold-storage
file is being staged. A BioMCP asset handle should wait through that bounded
staging state and still stream the final provider bytes.

```bash
../../tools/biomcp-ci get article 22663017 asset cold-storage-supplement.pdf | mustmatch like "Figshare cold-storage fixture bytes"
```

## Fulltext Reports Assets Not Included

Full text Markdown remains text-first, but JSON must tell agents which evidence
bytes were not inlined and how to retrieve them. The summary is structured so a
consumer can branch without scraping prose.

```bash
../../tools/biomcp-ci --json get article 22663011 fulltext | uv run --no-sync python3 -c '
import json, sys

doc = json.load(sys.stdin)
not_included = doc.get("not_included") or {}
figures = not_included.get("figure_images") or {}
supplements = not_included.get("supplementary_files") or {}
complex_tables = not_included.get("complex_tables") or {}
assert figures.get("count") == 2
assert supplements.get("count") == 1
assert complex_tables.get("count") == 1
assert figures.get("retrieve_with") == "biomcp --json get article 22663011 assets"
commands = (doc.get("_meta") or {}).get("next_commands") or []
assert "biomcp --json get article 22663011 assets" in commands
assert "biomcp get article 22663011 asset traces-s1.csv" in commands
print("article fulltext not_included ok")
' | mustmatch like "article fulltext not_included ok"
```

Markdown carries the retrieval command as a pointer instead of embedding the
JSON manifest or listing individual package members.

```bash
../../tools/biomcp-ci get article 22663011 fulltext | mustmatch like "biomcp --json get article 22663011 assets"
../../tools/biomcp-ci get article 22663011 fulltext | mustmatch not like "figure-floats.png
traces-s1.csv
sha256
size_bytes"
```

## Semantic Scholar Degrades Truthfully Without a Key

The blocking lane is intentionally keyless. Article search should stay usable
and explicit about the no-key path rather than hard-failing or pretending the
keyed data plane ran.

## Semantic Scholar Source Status Appears in Debug Plans

Debug plans are for operators and benchmark agents who need to explain the
route BioMCP used. The Semantic Scholar leg should carry the same redacted
auth and availability state there, without requiring stderr parsing.

## Authenticated Source Status Is Redacted

When an operator provides `S2_API_KEY`, article search should identify the
authenticated mode but never echo the key, a prefix, or any secret-derived
string in JSON metadata.

## Markdown Notes Semantic Scholar Unavailability

Markdown should stay quiet on healthy paths, but a failed Semantic Scholar leg is
operator-relevant. When the source is unavailable, the page should still render
primary article rows and include one concise source-status note.

## Entity Follow-Up

`article entities` is the compact follow-up in this bootstrap slice. It should
still expose the gene subsection and typed follow-up commands.

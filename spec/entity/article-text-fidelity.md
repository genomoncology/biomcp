# Article Text Fidelity

`get article` returns whole abstracts with no cut-off, and saved full text
reads cleanly (ticket 1294). Whole abstracts ride the detail surface: `get
article` and `batch article --mode detail` carry the complete abstract. Search
rows keep a 240-byte abstract snippet cap — compact rows and `--full` rows
alike — and `biomcp get article <pmid>` is the way to ask for the whole one.

The page replays recorded captures through the article source fixture: PMID
30738221's whole abstract from the receipted PubTator3 export, the TAILORx
JATS paper (PMC6172658) from the receipted NCBI EFetch response, and the
recorded PMC HTML page for PMC6695558. PMID 30738221 has no PMC record, so
its abstract's production source is the PubTator export, not the EFetch rung.

## Whole Abstracts

The whole abstract in `get article` JSON equals the abstract passage in the
recorded PubTator3 bytes, with no truncation marker.

```bash run id=whole-abstract-pubtator exit=0
abstract_json="$(../../tools/biomcp-ci --json get article 30738221)"
ARTICLE_JSON="$abstract_json" RECORDED_JSON=../../testdata/sources/pubtator/export_30738221.json \
  uv run --no-sync python3 - <<'PY'
import json, os
doc = json.loads(os.environ["ARTICLE_JSON"])
record = json.load(open(os.environ["RECORDED_JSON"], encoding="utf-8"))["PubTator3"][0]
recorded_abstract = next(
    passage["text"]
    for passage in record["passages"]
    if passage.get("infons", {}).get("type") == "abstract"
)
abstract = doc.get("abstract_text") or ""
assert abstract == recorded_abstract, "abstract differs from the recorded bytes"
assert "(truncated" not in abstract, "abstract still carries a truncation marker"
assert len(abstract) == 2369, f"expected the whole 2369-char abstract, got {len(abstract)}"
PY
mustmatch like '"abstract_text"' <<<"$abstract_json"
```

A blank base abstract is filled from the recorded NCBI EFetch JATS bytes,
whole and uncapped, while the saved full text renders through the same
response.

```bash run id=whole-abstract-jats exit=0
rm -rf "${BIOMCP_CACHE_DIR:?}/downloads"
jats_json="$(../../tools/biomcp-ci --json get article 29860917 fulltext)"
test "$(jq -r '.full_text_source.label' <<<"$jats_json")" = "NCBI EFetch PMC XML"
jq -r '.abstract_text' <<<"$jats_json" \
  | mustmatch '/^BACKGROUND The recurrence score based on the 21-gene breast cancer assay predicts/'
jq -r '.abstract_text' <<<"$jats_json" \
  | mustmatch '/number, \[NCT00310180\]\(NCT00310180\)\.\)$/'
jq '.abstract_text | length == 2231 and (contains("(truncated") | not)' <<<"$jats_json" \
  | mustmatch 'true'
```

## Search Rows Keep the 240-Byte Snippet Cap

The same recorded abstract returns as a capped snippet on every search row,
`--full` included; the whole abstract stays one `get article` away.

```bash run id=search-snippet-cap exit=0
whole="$(../../tools/biomcp-ci --json get article 30738221 | jq -r '.abstract_text')"
../../tools/biomcp-ci --json search article --keyword 'KRAS-mutant NSCLC' \
    --source europepmc --limit 1 --full \
  | jq -r --arg whole "$whole" \
      '.results[0].abstract_snippet as $s
       | ($s != null) and ($s | endswith("...")) and (($s | length) <= 243)
         and ($whole | startswith($s[0:-3]))' \
  | mustmatch 'true'
```

## JATS Full Text Reads Cleanly

The recorded TAILORx JATS response renders adjacent `surname` and
`given-names` elements as separate words, labels `PMID` and `PMCID` as their
own values instead of gluing them to citation prose, and keeps the merged-cell
table banner. Source-text defects stay as the source wrote them; BioMCP does
not rewrite prose.

```bash run id=jats-fidelity exit=0
rm -rf "${BIOMCP_CACHE_DIR:?}/downloads"
jats_json="$(../../tools/biomcp-ci --json get article 29860917 fulltext)"
path="$(jq -r '.full_text_path' <<<"$jats_json")"
rg -F 'Jemal A, Center MM, DeSantis C, Ward EM' "$path" >/dev/null
! rg -F 'JemalA' "$path"
rg -F '1893–907. PMID: 20647400.' "$path" >/dev/null
! rg -F '907.PMID' "$path"
! rg -F 'PMC4580552PMID' "$path"
rg -F 'recurrencescore' "$path" >/dev/null
test "$(rg -F -c 'merged-cell layout may be lossy' "$path")" -eq 3
mustmatch like '"full_text_path"' <<<"$jats_json"
```

A JATS table whose rows have uneven widths and no merged cells prints its raw
source rows behind a ragged-table banner instead of disappearing.

```bash run id=ragged-table exit=0
ragged_json="$(../../tools/biomcp-ci --json get article 29860918 fulltext)"
test "$(jq -r '.full_text_source.label' <<<"$ragged_json")" = "NCBI EFetch PMC XML"
ragged_path="$(jq -r '.full_text_path' <<<"$ragged_json")"
rg -F '*[Ragged table: 3 rows of uneven width. Raw source rows follow.]*' "$ragged_path" >/dev/null
rg -F 'Row 1: Group | Median | Range' "$ragged_path" >/dev/null
rg -F 'Row 3: B | 8.2 | 3-12' "$ragged_path" >/dev/null
mustmatch like '"full_text_path"' <<<"$ragged_json"
```

## PMC HTML Keeps the Title and Byline and Drops Page Furniture

The recorded PMC HTML page for PMC6695558 saves with its title first, its
byline under the title, and none of the page furniture: no tileshop viewer
links, no `[Open in a new tab]` lines, no Google Scholar `scholar_lookup`
URLs. Short canonical links (DOI, PubMed, PMC free article) stay.

```bash run id=pmc-html-fidelity exit=0
html_json="$(../../tools/biomcp-ci --json get article 31648294 fulltext)"
test "$(jq -r '.full_text_source.label' <<<"$html_json")" = "PMC HTML"
test "$(jq -r '.full_text_manifest.source_kind' <<<"$html_json")" = "pmc_html"
html_path="$(jq -r '.full_text_path' <<<"$html_json")"
head -n 1 "$html_path" \
  | mustmatch '# High rate of durable complete remission in follicular lymphoma after CD19 CAR-T cell immunotherapy'
rg -F 'Alexandre V Hirayama, Jordan Gauthier, Kevin A Hay' "$html_path" >/dev/null
for furniture in 'tileshop' '[Open in a new tab]' 'scholar.google.com/scholar_lookup'; do
  ! rg -F "$furniture" "$html_path"
done
rg -F 'https://doi.org/' "$html_path" >/dev/null
mustmatch like '"full_text_path"' <<<"$html_json"
```

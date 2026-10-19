# Gene Queries

Gene search is the fastest way to anchor a BioMCP session in a stable entity.
These canaries keep the focus on durable identity, deepen-path guidance, and
opt-in sections instead of volatile upstream counts or copy-edit trivia. A
page edit like this one must never cost a Rust compile.

The [gene discovery and identity specification](gene-discovery.md) checks the canonical BRAF search row and identity card against recorded replies.

## Exact Symbol Ranking and Pagination

An exact canonical symbol is promoted from the complete bounded provider set,
with duplicates removed before pagination and follow-up commands built from the
first retained row.

```bash
for offset in 0 3; do
  actual="$(../../tools/biomcp-ci --json search gene ODC1 --limit 1 --offset "$offset")"
  case "$offset" in
    0) expected='["ODC1"]' ;;
    3) expected='[]' ;;
  esac
  jq -e --argjson expected "$expected" '.results | map(.symbol) == $expected' \
    <<<"$actual" | mustmatch 'true'
done
../../tools/biomcp-ci --json search gene odc1 --limit 2 \
  | jq -e '.results | map(.symbol) == ["ODC1", "SLC25A21"]' \
  | mustmatch 'true'
../../tools/biomcp-ci search gene ODC1 --limit 1 \
  | mustmatch like 'ODC1
Showing 1-1 of 3 results. Use --offset 1 for more.'

# Local filters run against the complete retained set.
../../tools/biomcp-ci --json search gene FILTERCASE --type protein-coding --chromosome chr7 --region chr7:150-250 \
  | jq -e '((.results | map(.symbol)) == ["FILTERCASE","FILTER_ALIAS"]) and .count == 2' \
  | mustmatch 'true'

# Lucene metacharacters remain escaped free-text terms.
lucene_query='+-=&&||><!(){}[]^"~*?:/\'
../../tools/biomcp-ci --json search gene "$lucene_query" --limit 1 \
  | jq -e '.results == [] and .pagination.total == 0' \
  | mustmatch 'true'

# Empty/invalid windows reject before any provider work.
set +e
../../tools/biomcp-ci --json search gene ODC1 --limit 1 --offset 10000 >/dev/null
status=$?
set -e
test "$status" -eq 2
```

## Raw and Typed MCP Gene Search Match the CLI

Both MCP search surfaces reuse the CLI's ordered rows and follow-up commands;
MCP adds only its existing next-command section to Markdown.

## Adversarial identity and public-surface proof

The production parser receives the exact emitted follow-up command, including a
provider symbol that exercises shell quoting. It must remain one argument and
gene-get validation must reject it before a second provider request.

## Search Table Contract

The recorded [gene-search golden cases](../../tests/golden/cases/search-gene-braf.json) check the JSON follow-up commands.

## Disabled KEGG Drops Carried Pathway Names

MyGene carries KEGG pathway rows inside its own gene record. With kegg off
through `--disable-sources`, those rows leave the pathways section while the
gene record itself still resolves.

```bash
../../tools/biomcp-ci --json get gene BRAF pathways \
  | jq -e '(.pathways // []) | any(.source == "KEGG" and .id == "hsa04010")' \
  | mustmatch 'true'
kegg_disabled="$(BIOMCP_DISABLE_SOURCES=kegg ../../tools/biomcp-ci --json get gene BRAF pathways)"
jq -e '(.symbol == "BRAF") and ((.pathways // []) | all(.source != "KEGG"))' <<<"$kegg_disabled" | mustmatch 'true'
! grep -Fq 'Homo sapiens (human)' <<<"$kegg_disabled"
```

## Shipped schema matches the fixture-backed CLI payload

The checked-in gene schema must validate the actual JSON emitted from the captured MyGene BRAF response. This closes the gate if the Rust serializer and shipped skill schema drift apart, even when the static example still agrees with the stale schema.

## Common Alias Get Resolves Canonical Gene

Clinical reports and papers often use common aliases instead of the HGNC symbol.
For an alias that maps to one canonical gene, `get gene` should return the same
stable gene card a user would get from the official symbol.

```bash
../../tools/biomcp-ci --json get gene PD-L1 | mustmatch like '"symbol": "CD274"
"entrez_id": "29126"
"PD-L1"'
```

## Diagnostics and Pathways Pivots

The base gene view advertises its diagnostic and pathway deepen paths without
requiring any optional enrichment provider.

```bash
../../tools/biomcp-ci --json get gene BRCA1 | mustmatch like '"next_commands":'
../../tools/biomcp-ci --json get gene BRCA1 | jq -e '._meta.next_commands | any(. == "biomcp get gene BRCA1 diagnostics")' >/dev/null
../../tools/biomcp-ci --json get gene BRCA1 | jq -e '._meta.next_commands | any(. == "biomcp get gene BRCA1 pathways")' >/dev/null
```

## Observed MyGene Requests

The local fixture records the exact-symbol identity plan. The replay specification checks the bounded BRAF search request.

```bash
grep -F 'GET /mygene/v3/query?q=symbol%3A%22BRCA1%22' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" | mustmatch like 'symbol%3A%22BRCA1%22'
```

## Typed optional-section outcomes

Requested sections keep a bounded state even when providers return no rows or
are temporarily unavailable. Provenance carries the same state rather than
inferring success from an empty collection.

```bash
../../tools/biomcp-ci --json get gene BRAF go interactions \
  | jq '. as $root | ["go", "interactions"] | all(.[]; . as $key | $root.section_outcomes[$key] as $outcome | ($outcome.outcome | IN("data", "empty", "unavailable")) and ($root._meta.section_sources | any(.key == $key and .outcome == $outcome.outcome and .sources == $outcome.sources)) and ($root._meta.section_sources | all(.key != $key or (.outcome == $outcome.outcome and .sources == $outcome.sources))))' \
  | mustmatch 'true'
grep -F 'GET /quickgo/QuickGO/services/annotation/search?geneProductId=P15056&limit=20' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" | mustmatch like 'geneProductId=P15056'
grep -F 'GET /string/api/json/network?identifiers=BRAF&species=9606&limit=15' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" | mustmatch like 'species=9606'
```

## Partial ClinGen evidence

ClinGen validity, dosage sensitivity, and the shared gene lookup start together
under independent deadlines. A slow dosage download keeps completed validity
evidence and reports the exact degraded aggregate on both provenance surfaces.

```bash
BIOMCP_TEST_UNPACED_ORIGIN="$BIOMCP_PROVIDER_CONTRACT_BASE" BIOMCP_CLINGEN_BASE="$BIOMCP_PROVIDER_CONTRACT_BASE/clingen/dosage-timeout" BIOMCP_GENE_OPTIONAL_TIMEOUT_MS=40 \
  ../../tools/biomcp-ci --json get gene TP53 clingen \
  | jq -e '.clingen.validity[0].disease == "Li-Fraumeni syndrome" and .clingen.validity_status == {status:"data", op:"gene_validity_download"} and .clingen.dosage_status == {status:"timed_out", op:"gene_dosage_download", message:"ClinGen gene-dosage download timed out."} and .section_outcomes.clingen == {outcome:"degraded", sources:["ClinGen"], message:"ClinGen gene evidence is partial; one result family is unavailable."} and (._meta.section_sources | any(.key == "clingen" and .label == "ClinGen" and .outcome == "degraded" and .sources == ["ClinGen"]))' \
  | mustmatch 'true'
```

The inverse failure retains the newest dosage row, including a literal ClinGen
no-evidence classification. Markdown exposes both family states and never
invents a missing classification.

```bash
BIOMCP_TEST_UNPACED_ORIGIN="$BIOMCP_PROVIDER_CONTRACT_BASE" BIOMCP_CLINGEN_BASE="$BIOMCP_PROVIDER_CONTRACT_BASE/clingen/validity-fail" BIOMCP_GENE_OPTIONAL_TIMEOUT_MS=200 \
  ../../tools/biomcp-ci --json get gene TP53 clingen \
  | jq -e '.clingen.validity_status == {status:"failed", op:"gene_validity_download", message:"ClinGen gene-validity download failed."} and .clingen.dosage_status == {status:"data", op:"gene_dosage_download"} and .clingen.haploinsufficiency == "Sufficient Evidence for Haploinsufficiency" and .clingen.triplosensitivity == "No Evidence for Triplosensitivity" and .section_outcomes.clingen.outcome == "degraded"' \
  | mustmatch 'true'
BIOMCP_TEST_UNPACED_ORIGIN="$BIOMCP_PROVIDER_CONTRACT_BASE" BIOMCP_CLINGEN_BASE="$BIOMCP_PROVIDER_CONTRACT_BASE/clingen/validity-fail" BIOMCP_GENE_OPTIONAL_TIMEOUT_MS=200 \
  ../../tools/biomcp-ci get gene TP53 clingen \
  | mustmatch like 'Gene-Disease Validity Status
Status: `failed`
ClinGen gene-validity download failed.
Dosage-Sensitivity Status
Status: `data`
Haploinsufficiency: Sufficient Evidence for Haploinsufficiency
Triplosensitivity: No Evidence for Triplosensitivity'
```

Raw MCP text and JSON plus typed MCP `get` travel through the same section
contract. The typed request schema remains the existing gene/section request.

```bash
source ../fixtures/cache-policy.sh
set_fixture_cache_root "${BIOMCP_PROVIDER_CONTRACT_READY_FILE%/base-url}/clingen-mcp-cache"
BIOMCP_TEST_UNPACED_ORIGIN="$BIOMCP_PROVIDER_CONTRACT_BASE" BIOMCP_CLINGEN_BASE="$BIOMCP_PROVIDER_CONTRACT_BASE/clingen/validity-fail" BIOMCP_GENE_OPTIONAL_TIMEOUT_MS=200 \
  bash ../fixtures/run-section-outcome-mcp.sh ../.. clingen-surfaces \
  | mustmatch like 'RAW TEXT
Gene-Disease Validity Status
ClinGen gene-validity download failed.
Triplosensitivity: No Evidence for Triplosensitivity
RAW JSON
"validity_status": {
"status": "failed"
"dosage_status": {
"status": "data"
TYPED JSON
"section_outcomes": {
"clingen": {
"outcome": "degraded"
"section_sources":'
```

## GenCC submission-level validity

The receipt-backed new-format fixture preserves the three independent ODC1
assertions. JSON, Markdown, all-section, and batch surfaces keep their
evaluated-date order and source-specific lifecycle state without claiming a
consensus.

```bash
export BIOMCP_TEST_UNPACED_ORIGIN="$BIOMCP_PROVIDER_CONTRACT_BASE"
../../tools/biomcp-ci list gene | mustmatch like 'get gene <symbol> gencc`
GenCC submission-level gene-disease validity'
../../tools/biomcp-ci get gene --help | mustmatch like 'clingen, gencc, constraint'
../../tools/biomcp-ci gencc --help | mustmatch like 'sync  Revalidate the local GenCC gene-disease validity dataset'
../../tools/biomcp-ci --json get gene ODC1 gencc \
  | jq -e '.gencc.assertions | ((length == 3) and (map(.submitter.label) == ["G2P", "PanelApp Australia", "Labcorp Genetics (formerly Invitae)"]))' \
  | mustmatch 'true'
../../tools/biomcp-ci --json get gene ODC1 gencc \
  | jq -e '.gencc.total_matching_assertions == 3 and (.gencc.truncated | not) and .gencc.status.freshness == "fresh" and .gencc.status.result == "data" and .section_outcomes.gencc == {outcome:"data", sources:["GenCC"]} and (._meta.section_sources | any(.key == "gencc" and .label == "GenCC gene-disease validity" and .outcome == "data" and .sources == ["GenCC"]))' \
  | mustmatch 'true'
../../tools/biomcp-ci get gene ODC1 gencc \
  | mustmatch like '## GenCC gene-disease validity
G2P (GENCC:000112)
PanelApp Australia (GENCC:000111)
Labcorp Genetics (formerly Invitae) (GENCC:000106)
[PMID 30239107](<https://pubmed.ncbi.nlm.nih.gov/30239107/>)'
BIOMCP_GENE_OPTIONAL_TIMEOUT_MS=200 ../../tools/biomcp-ci --json get gene ODC1 all \
  | jq -e '.gencc.assertions | length == 3' \
  | mustmatch 'true'
../../tools/biomcp-ci --json batch gene ODC1,ODC1 --sections gencc \
  | jq -e '.summary == {total:2,succeeded:2,failed:0} and (.items | all(.status == "ok" and (.result.gencc.assertions | length == 3)))' \
  | mustmatch 'true'
unset BIOMCP_TEST_UNPACED_ORIGIN
```

GenCC and ClinGen remain independent sections and provenance owners. The same
GenCC projection is available through raw MCP text/JSON and typed MCP `get`.

```bash
export BIOMCP_TEST_UNPACED_ORIGIN="$BIOMCP_PROVIDER_CONTRACT_BASE"
BIOMCP_TEST_UNPACED_ORIGIN="$BIOMCP_PROVIDER_CONTRACT_BASE" BIOMCP_CLINGEN_BASE="$BIOMCP_PROVIDER_CONTRACT_BASE/clingen" BIOMCP_GENE_OPTIONAL_TIMEOUT_MS=200 ../../tools/biomcp-ci --json get gene TP53 clingen gencc \
  | jq -e '.clingen.validity[0].disease == "Li-Fraumeni syndrome" and .gencc.status.result == "empty" and .section_outcomes.clingen.sources == ["ClinGen"] and .section_outcomes.gencc.sources == ["GenCC"]' \
  | mustmatch 'true'
source ../fixtures/cache-policy.sh
set_fixture_cache_root "${BIOMCP_PROVIDER_CONTRACT_READY_FILE%/base-url}/gencc-mcp-cache"
  bash ../fixtures/run-section-outcome-mcp.sh ../.. gencc-surfaces \
  | jq -e '.raw_text | contains("## GenCC gene-disease validity") and contains("G2P (GENCC:000112)")' \
  | mustmatch 'true'
source ../fixtures/cache-policy.sh
set_fixture_cache_root "${BIOMCP_PROVIDER_CONTRACT_READY_FILE%/base-url}/gencc-mcp-cache"
  bash ../fixtures/run-section-outcome-mcp.sh ../.. gencc-surfaces \
  | jq -e '.raw_json.gencc == .typed_json.gencc and .raw_json.section_outcomes.gencc == {outcome:"data", sources:["GenCC"]} and .typed_json.section_outcomes.gencc == .raw_json.section_outcomes.gencc and (.raw_json.gencc.assertions | map(.submitter.label)) == ["G2P", "PanelApp Australia", "Labcorp Genetics (formerly Invitae)"]' \
  | mustmatch 'true'
unset BIOMCP_TEST_UNPACED_ORIGIN
```

## GenCC adapter projection parity

The adapter's available, identity-match, and unavailable representatives are
enough to prove the public surfaces consume one projection. The four calls in
each representative compare the section payload, outcome, and provenance;
they do not repeat the full lifecycle matrix through every transport.

```bash timeout=600
python3 - <<'PY' | mustmatch like 'GenCC adapter projections converge across CLI, raw MCP, typed MCP, and batch'
import json
import os
import shutil
import subprocess
import tempfile

binary = os.environ["BIOMCP_BIN"]
base_env = os.environ.copy()
base_env["BIOMCP_TEST_UNPACED_ORIGIN"] = base_env["BIOMCP_PROVIDER_CONTRACT_BASE"]

def cli(args, env):
    return json.loads(subprocess.check_output(
        [binary, "--json", *args], text=True, env=env))

def raw_and_typed(symbol, env):
    mcp_env = env.copy()
    mcp_env["BIOMCP_GENCC_SURFACES_GENE"] = symbol
    output = subprocess.check_output(
        ["bash", "../fixtures/run-section-outcome-mcp.sh", "../..", "gencc-surfaces"],
        text=True, env=mcp_env)
    surfaces = json.loads(output)
    return surfaces["raw_json"], surfaces["typed_json"]

def section_projection(card):
    return {
        "gencc": card["gencc"],
        "outcome": card["section_outcomes"]["gencc"],
        "provenance": [entry for entry in card["_meta"]["section_sources"]
                       if entry["key"] == "gencc"],
    }

def assert_surfaces(symbol, env, expected):
    cli_card = cli(["get", "gene", symbol, "gencc"], env)
    raw_card, typed_card = raw_and_typed(symbol, env)
    batch = cli(["batch", "gene", symbol, "--sections", "gencc"], env)
    batch_card = batch["items"][0]["result"]
    projections = [section_projection(card) for card in
                   (cli_card, raw_card, typed_card, batch_card)]
    assert all(projection == projections[0] for projection in projections), projections
    assert projections[0]["outcome"] == expected, projections[0]
    return projections[0]

# Available data: warm once so all four representative calls use local_query.
available_env = base_env.copy()
cli(["get", "gene", "ODC1", "gencc"], available_env)
available = assert_surfaces("ODC1", available_env,
    {"outcome": "data", "sources": ["GenCC"]})
assert len(available["gencc"]["assertions"]) == 3

# A resolved HGNC that belongs to a different symbol is identity_match, not an
# empty GenCC result. The fixture route supplies that conflicting MyGene card.
identity = assert_surfaces("GENCCIDENTITY", base_env,
    {"outcome": "unavailable", "sources": [],
     "message": "GenCC gene identity is inconclusive; no GenCC absence can be concluded."})
assert identity["gencc"]["status"]["operation"] == "identity_match"
assert identity["gencc"]["assertions"] == []

# Establish the failed acquisition once; subsequent calls share its durable
# retry-suppressed projection without multiplying the unavailable matrix.
unavailable_env = base_env.copy()
unavailable_env["BIOMCP_GENCC_BASE"] = base_env["BIOMCP_PROVIDER_CONTRACT_BASE"] + "/gencc/missing"
unavailable_env["BIOMCP_GENCC_TEST_NOW"] = "2026-09-09T00:00:00Z"
unavailable_dir = tempfile.mkdtemp(prefix="gencc-adapter-unavailable-",
                                    dir=base_env["BIOMCP_GENCC_FIXTURE_PARENT"])
unavailable_env["BIOMCP_GENCC_DIR"] = unavailable_dir
try:
    first = cli(["get", "gene", "ODC1", "gencc"], unavailable_env)
    assert first["gencc"]["status"]["freshness"] == "unavailable"
    unavailable = assert_surfaces("ODC1", unavailable_env,
        {"outcome": "unavailable", "sources": [],
         "message": "GenCC data is unavailable; no GenCC absence can be concluded."})
    assert unavailable["gencc"]["status"]["result"] == "unknown"
    assert unavailable["gencc"]["assertions"] == []
finally:
    shutil.rmtree(unavailable_dir)

print("GenCC adapter projections converge across CLI, raw MCP, typed MCP, and batch")
PY
```

Health uses a metadata-only HEAD. An explicit sync then revalidates the
immutable generation with both validators and a zero-body `304`; ordinary
fresh reuse above performs no second download.

```bash
export BIOMCP_TEST_UNPACED_ORIGIN="$BIOMCP_PROVIDER_CONTRACT_BASE"
../../tools/biomcp-ci --json health --api GenCC \
  | jq -e '.total == 1 and .healthy == 1 and .rows[0].api == "GenCC" and .rows[0].status == "ok"' \
  | mustmatch 'true'
grep -F 'HEAD /gencc/download/action/submissions-export-csv?format=new' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" \
  | mustmatch like 'format=new'
../../tools/biomcp-ci --json gencc sync | jq -e '.source == "gencc" and .status == "synchronized" and (.changed | not) and (has("updated") | not)' | mustmatch 'true'
grep -F 'GET /gencc/download/action/submissions-export-csv?format=new If-None-Match="6ebdbf28b305e99e349ed827a219214b" If-Modified-Since=Sun, 06 Sep 2026 06:00:29 GMT' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" \
  | mustmatch like 'If-None-Match="6ebdbf28b305e99e349ed827a219214b"'
test "$(grep -c '^GET /gencc/download/action/submissions-export-csv?format=new' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG")" -eq 2
unset BIOMCP_TEST_UNPACED_ORIGIN
```

## All-Section Warm Budget

Quarantined from routine executable specs by ticket 372 because this timing-only
canary failed twice during routine `make spec-pr` at 45599ms and 43332ms against
a 12000ms ceiling. Ticket 371's request-contract strategy keeps performance
canaries out of the default gate; restore this only as a deterministic
benchmark/ratchet or explicit performance lane.

## Tissue-Expression Context

Human Protein Atlas data belongs in an opt-in deepen path and retains its
source reliability and subcellular context.

```bash
../../tools/biomcp-ci get gene BRAF hpa | mustmatch like '## Human Protein Atlas
| Adipose tissue | Low |
Reliability: Supported
Subcellular main locations: cytosol, vesicles'
grep -F 'GET /hpa/ENSG00000157764.xml' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" | mustmatch like 'ENSG00000157764.xml'
```

## Druggability & Targets

Targetability context stays separate from the default card while combining
Open Targets tractability and DGIdb interaction evidence.

```bash
../../tools/biomcp-ci get gene EGFR druggability | mustmatch like '## Druggability
OpenTargets tractability
| antibody | yes | Approved Drug'
grep -F 'POST /dgidb/api/graphql' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" | mustmatch like '"gene":"EGFR"'
grep -F 'POST /opentargets/api/v4/graphql' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" | mustmatch like '"ensemblId":"ENSG00000146648"'
```

## Funding

Funding remains opt-in and retains its source-attributed bounded table.

```bash
../../tools/biomcp-ci get gene ERBB2 funding | mustmatch like '## Funding (NIH Reporter)
| Project | PI | Organization | FY | Amount |
Showing top 8 unique grants from 187 matching NIH project-year records'
grep -F 'POST /nih/v2/projects/search' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" | mustmatch like '"search_text":"\"ERBB2\""'
```

## Diagnostic Local Data

The diagnostic deepen path consumes the bounded local GTR bundle rather than
downloading provider data during the routine gate.

```bash
../../tools/biomcp-ci get gene BRCA1 diagnostics | mustmatch like '## Diagnostics
GTR000000001.1
NCBI Genetic Testing Registry'
```

## Cell Line Expression Across One Cancer Group

`gene cell-lines` reports the HPA nTPM of one gene in every cell line of one
cancer group and joins each name to Cellosaurus in batches of twenty. The whole
group answers in five identifier searches.

```bash
../../tools/biomcp-ci --json gene cell-lines FLT3 --group leukemia \
  | jq -e '.total == 93 and (.rows | map(select(.name == "MOLM-13")) | .[0].accession) == "CVCL_2119"' \
  | mustmatch 'true'
grep -c 'GET /cellosaurus/search/cell-line?q=id%3A%28' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" | mustmatch '5'
```

The Markdown ends with the Human Protein Atlas attribution line, naming the
recorded file date and the licence the Atlas publishes.

```bash
../../tools/biomcp-ci gene cell-lines FLT3 --group leukemia --offset 47 --limit 2 \
  | mustmatch like '| MOLM-13 | CVCL_2119 | 166.1 |
Human Protein Atlas, files dated 2025-11-05, CC BY 4.0. proteinatlas.org'
```

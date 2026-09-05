# Gene Queries

Gene search is the fastest way to anchor a BioMCP session in a stable entity.
These canaries keep the focus on durable identity, deepen-path guidance, and
opt-in sections instead of volatile upstream counts or copy-edit trivia.

## Symbol-Based Search

Symbol search should still surface the canonical BRAF row in a human-scannable
table before the user pivots into deeper sections.

```bash
../../tools/biomcp-ci search gene BRAF --limit 3 | mustmatch like '# Genes: BRAF
B-Raf proto-oncogene'
```

## Search Table Contract

The search surface needs to stay readable for humans and still expose machine
follow-ups through `_meta.next_commands`.

```bash
../../tools/biomcp-ci --json search gene BRAF --limit 3 | mustmatch like '"next_commands":'
../../tools/biomcp-ci --json search gene BRAF --limit 3 | jq -e '._meta.next_commands[0] | test("^biomcp get gene .+$")' >/dev/null
../../tools/biomcp-ci --json search gene BRAF --limit 3 | jq -e '._meta.next_commands | any(. == "biomcp list gene")' >/dev/null
```

## Identity Card

The default card should keep the persistent identifier and the progressive
disclosure hints that let readers deepen into the right follow-up section.

```bash
../../tools/biomcp-ci get gene BRAF | mustmatch like 'Entrez ID: 673
biomcp get gene BRAF pathways
biomcp get gene BRAF diagnostics'
```

## Shipped schema matches the fixture-backed CLI payload

The checked-in gene schema must validate the actual JSON emitted from the captured MyGene BRAF response. This closes the gate if the Rust serializer and shipped skill schema drift apart, even when the static example still agrees with the stale schema.

```bash
gene_json="$(../../tools/biomcp-ci --json get gene BRAF)"
GENE_JSON="$gene_json" uv run --no-sync python3 - ../../skills/schemas/gene.json <<'PY' | mustmatch like 'gene schema matches fixture-backed CLI payload'
import json
import os
from copy import deepcopy
from pathlib import Path
import sys

from jsonschema import Draft202012Validator, ValidationError

schema = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
Draft202012Validator.check_schema(schema)
validator = Draft202012Validator(schema)
payload = json.loads(os.environ["GENE_JSON"])
validator.validate(payload)

without_coordinates = deepcopy(payload)
without_coordinates.pop("genomic_coordinates")
validator.validate(without_coordinates)

null_coordinates = deepcopy(payload)
null_coordinates["genomic_coordinates"] = None
validator.validate(null_coordinates)

def require_rejection(candidate, label):
    try:
        validator.validate(candidate)
    except ValidationError:
        return
    raise AssertionError(f"gene schema accepted invalid {label}")

for required_field in ("coordinate", "genome_build", "source"):
    missing_required = deepcopy(payload)
    missing_required["genomic_coordinates"].pop(required_field)
    require_rejection(missing_required, f"coordinate object without {required_field}")

null_provenance = deepcopy(payload)
null_provenance["genomic_coordinates"]["provenance"] = None
require_rejection(null_provenance, "null coordinate provenance")

extra_property = deepcopy(payload)
extra_property["genomic_coordinates"]["unexpected"] = "value"
require_rejection(extra_property, "extra coordinate property")
print("gene schema matches fixture-backed CLI payload")
PY
```

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

The local fixture records requests emitted by the production client, including
the bounded search and exact-symbol identity plans.

```bash
grep -F 'GET /mygene/v3/query?q=%28symbol%3ABRAF+OR+alias%3ABRAF%29' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" | mustmatch like '&size=3&from=0'
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
BIOMCP_CLINGEN_BASE="$BIOMCP_PROVIDER_CONTRACT_BASE/clingen/dosage-timeout" BIOMCP_GENE_OPTIONAL_TIMEOUT_MS=40 \
  ../../tools/biomcp-ci --json get gene TP53 clingen \
  | jq -e '.clingen.validity[0].disease == "Li-Fraumeni syndrome" and .clingen.validity_status == {status:"data", op:"gene_validity_download"} and .clingen.dosage_status == {status:"timed_out", op:"gene_dosage_download", message:"ClinGen gene-dosage download timed out."} and .section_outcomes.clingen == {outcome:"degraded", sources:["ClinGen"], message:"ClinGen gene evidence is partial; one result family is unavailable."} and (._meta.section_sources | any(.key == "clingen" and .label == "ClinGen" and .outcome == "degraded" and .sources == ["ClinGen"]))' \
  | mustmatch 'true'
```

The inverse failure retains the newest dosage row, including a literal ClinGen
no-evidence classification. Markdown exposes both family states and never
invents a missing classification.

```bash
BIOMCP_CLINGEN_BASE="$BIOMCP_PROVIDER_CONTRACT_BASE/clingen/validity-fail" BIOMCP_GENE_OPTIONAL_TIMEOUT_MS=200 \
  ../../tools/biomcp-ci --json get gene TP53 clingen \
  | jq -e '.clingen.validity_status == {status:"failed", op:"gene_validity_download", message:"ClinGen gene-validity download failed."} and .clingen.dosage_status == {status:"data", op:"gene_dosage_download"} and .clingen.haploinsufficiency == "Sufficient Evidence for Haploinsufficiency" and .clingen.triplosensitivity == "No Evidence for Triplosensitivity" and .section_outcomes.clingen.outcome == "degraded"' \
  | mustmatch 'true'
BIOMCP_CLINGEN_BASE="$BIOMCP_PROVIDER_CONTRACT_BASE/clingen/validity-fail" BIOMCP_GENE_OPTIONAL_TIMEOUT_MS=200 \
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
BIOMCP_CLINGEN_BASE="$BIOMCP_PROVIDER_CONTRACT_BASE/clingen/validity-fail" BIOMCP_GENE_OPTIONAL_TIMEOUT_MS=200 \
  BIOMCP_CACHE_DIR="${BIOMCP_PROVIDER_CONTRACT_READY_FILE%/base-url}/clingen-mcp-cache" \
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

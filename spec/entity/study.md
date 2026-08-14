# Study Queries

The `study` surface is BioMCP's local cBioPortal analytics layer, separate from
the remote trial registry surface. These canaries keep the local catalog,
typed query grammar, validation messages, and chartable summaries visible
without pinning install-specific row totals.

## Local Study Discovery

Listing studies should still look like a local dataset catalog, with stable
identity and availability columns that tell operators what data is actually on
disk, including structural-variant files when a study package ships them.

```bash
../../tools/biomcp-ci study list | mustmatch like '# Study Datasets
| Study ID | Name | Cancer Type | Samples | Available Data |
msk_impact_2017
structural_variants'
```

## Gene-Frequency Summary

Per-study mutation queries should keep a human-readable summary heading and the
variant-class breakout that explains what was counted. When the same study has
structural-variant data, the mutation summary also says that fusions/SV are not
part of the mutation count and points to the SV query.

```bash
../../tools/biomcp-ci study query --study msk_impact_2017 --gene TP53 --type mutations | mustmatch like '# Study Mutation Frequency: TP53 (msk_impact_2017)
## Top Variant Classes
## Top Protein Changes
excludes fusions/SV
--type sv'
```

Queries for studies outside the local cBioPortal snapshot should return a
coverage signal with the download hint, not a hollow local result. The spec
fixture's local cohort list is intentionally small: `msk_impact_2017`,
`brca_tcga_pan_can_atlas_2018`, and `paad_qcmg_uq_2016`.

```bash
../../tools/biomcp-ci study query --study skcm_tcga_pan_can_atlas_2018 --gene BRAF --type mutations | mustmatch like '# Study Not in Local cBioPortal Cohorts: skcm_tcga_pan_can_atlas_2018
not_in_local_cohorts
biomcp study download skcm_tcga_pan_can_atlas_2018
msk_impact_2017
brca_tcga_pan_can_atlas_2018
paad_qcmg_uq_2016'
../../tools/biomcp-ci --json study query --study skcm_tcga_pan_can_atlas_2018 --gene BRAF --type mutations | mustmatch like '"query_type": "not_in_local_cohorts"
"coverage_status": "not_in_local_cohorts"
biomcp study download skcm_tcga_pan_can_atlas_2018'
```

## Structural Variant Queries

Structural-variant queries use the same per-study query command, but return a
fusion-oriented row shape with both breakpoints and the event label from the
local `data_sv.txt` file.

```bash
../../tools/biomcp-ci study query --help | mustmatch like 'Canonical values:
sv
Accepted aliases:
fusion'
```

```bash
../../tools/biomcp-ci study query --study msk_impact_2017 --gene RET --type sv | mustmatch like '# Study Structural Variants: RET (msk_impact_2017)
| Sample | Site 1 Gene | Site 2 Gene | Frame/Effect | Split Reads | Event Info |
KIF5B
RET
in-frame
KIF5B-RET Fusion'
```

```bash
../../tools/biomcp-ci study query --study msk_impact_2017 --gene RET --type fusion | mustmatch like '# Study Structural Variants: RET (msk_impact_2017)
KIF5B-RET Fusion'
```

## Top Mutated Genes

Cohort-wide mutation rankings stay mutation-specific. If a study also includes
structural variants, the output tells users that fusions/SV need the SV query
instead of implying the ranking covers every actionable lesion type.

```bash
../../tools/biomcp-ci study top-mutated --study msk_impact_2017 | mustmatch like '# Study Top Mutated Genes: msk_impact_2017
| Gene | Mutated Samples | Mutation Events | Total Samples | Mutation Rate |
excludes fusions/SV
--type sv'
```

## Remote Download Contracts

Remote DataHub archives can be large, but routine specs should not depend on a
mock server or public network behavior. The no-network source tests prove the
download request shape, archive HTTP error mapping, and local archive extraction
contract.

## Filter Validation

Filter workflows should reject missing criteria explicitly instead of silently
returning the full cohort.

```bash
../../tools/biomcp-ci study filter --study brca_tcga_pan_can_atlas_2018 2>&1 | mustmatch like 'At least one filter criterion is required.
--mutated, --amplified, --deleted'
```

Expression thresholds are measurements, so both comparison flags reject NaN,
infinity, and overflowing numeric forms before looking for a local study. The
JSON CLI envelope and raw MCP tool result keep that failure machine-detectable.

```bash
set +e
cli_json="$(../../tools/biomcp-ci --json study filter --study definitely-not-a-local-study --expression-above MYC:NaN 2>/dev/null)"
cli_status=$?
set -e
test "$cli_status" -eq 2
CLI_JSON="$cli_json" python3 - <<'PY'
import json
import os

payload = json.loads(os.environ["CLI_JSON"])
assert payload["error"]["code"] == "invalid_argument", payload
assert "finite" in payload["error"]["message"], payload
print("CLI non-finite threshold: invalid_argument")
PY

biomcp_bin="${BIOMCP_BIN:-../../target/spec/biomcp}"
python3 - "$biomcp_bin" <<'PY'
import json
import subprocess
import sys

process = subprocess.Popen(
    [sys.argv[1], "serve"],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    text=True,
)

def request(payload):
    process.stdin.write(json.dumps(payload) + "\n")
    process.stdin.flush()
    return json.loads(process.stdout.readline())

try:
    request({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {"name": "study-spec", "version": "1"},
        },
    })
    process.stdin.write(json.dumps({
        "jsonrpc": "2.0",
        "method": "notifications/initialized",
        "params": {},
    }) + "\n")
    process.stdin.flush()
    response = request({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "biomcp",
            "arguments": {
                "command": "biomcp study filter --study definitely-not-a-local-study --expression-below MYC:inf",
                "json": True,
            },
        },
    })
    result = response["result"]
    text = result["content"][0]["text"]
    assert result["isError"] is True, response
    assert "Invalid argument" in text and "finite" in text, response
    print("Raw MCP non-finite threshold: invalid_argument tool error")
finally:
    process.terminate()
    process.wait(timeout=5)
PY
```

Finite thresholds retain strict boundary comparisons for both flags.

```bash
../../tools/biomcp-ci study filter --study brca_tcga_pan_can_atlas_2018 --expression-above ERBB2:1 --expression-below TP53:3 | mustmatch like '# Study Filter: brca_tcga_pan_can_atlas_2018
| expression > 1 for ERBB2 | 2 |
| expression < 3 for TP53 | 2 |
| Intersection | 1 |
S3'
```

## Survival Validation

Survival analysis should stay typed: unknown endpoint names must fail fast and
tell the operator which endpoint vocabulary is valid.

```bash
../../tools/biomcp-ci study cohort --study msk_impact_2017 --gene TP53 | mustmatch like '# Study Cohort: TP53 (msk_impact_2017)
| Group | Samples | Patients |'
../../tools/biomcp-ci study survival --study msk_impact_2017 --gene TP53 --endpoint foo 2>&1 | mustmatch like "Unknown survival endpoint 'foo'.
Expected: os, dfs, pfs, dss."
```

## Typed Comparison Validation

Comparison and co-occurrence analytics should reject malformed inputs before
running local cohort work.

```bash
../../tools/biomcp-ci study compare --study msk_impact_2017 --gene TP53 --type foo --target ERBB2 2>&1 | mustmatch like "Unknown comparison type 'foo'. Expected: expression, mutations."
../../tools/biomcp-ci study co-occurrence --study msk_impact_2017 --genes TP53 2>&1 | mustmatch like '--genes must contain 2 to 10 comma-separated symbols'
```

## Undefined Statistics

Study analytics retain raw cohort counts while representing statistics that cannot
be computed as JSON `null`. When neither gene occurs, the complete contingency
table remains available without implying an association.

```bash
../../tools/biomcp-ci --json study co-occurrence --study brca_tcga_pan_can_atlas_2018 --genes ZZQQXX,NOTAGENE | mustmatch like '{"pairs":[{"gene_a":"ZZQQXX","gene_b":"NOTAGENE","both_mutated":0,"a_only":0,"b_only":0,"neither":3,"log_odds_ratio":null,"p_value":null}]}'
```

The same rule applies when only one gene is absent: a zero required marginal
makes both inferential values undefined while preserving every raw cell.

```bash
../../tools/biomcp-ci --json study co-occurrence --study brca_tcga_pan_can_atlas_2018 --genes TP53,NOTAGENE | mustmatch like '{"pairs":[{"gene_a":"TP53","gene_b":"NOTAGENE","both_mutated":0,"a_only":1,"b_only":0,"neither":2,"log_odds_ratio":null,"p_value":null}]}'
```

A zero cell alone does not make a table degenerate. If both genes have observed
mutations and all row and column marginals are non-zero, BioMCP still reports the
sparse table's calculated statistics.

```bash
../../tools/biomcp-ci --json study co-occurrence --study brca_tcga_pan_can_atlas_2018 --genes TP53,PIK3CA | mustmatch like '"both_mutated": 0,
"a_only": 1,
"b_only": 1,
"neither": 1,
"log_odds_ratio": -
"p_value": 1'
```

Likewise, an empty expression-comparison group has a real sample count of zero
but has no distribution to summarize. Every distribution statistic stays present
in structured output with a null value.

```bash
../../tools/biomcp-ci --json study compare --study brca_tcga_pan_can_atlas_2018 --gene ZZQQXX --type expression --target TP53 | mustmatch like '{"groups":[{"group_name":"ZZQQXX-mutant","sample_count":0,"mean":null,"median":null,"min":null,"max":null,"q1":null,"q3":null}]}'
```

The terminal table uses its standard missing-value marker for that same group.

```bash
../../tools/biomcp-ci study compare --study brca_tcga_pan_can_atlas_2018 --gene ZZQQXX --type expression --target TP53 | mustmatch like '| ZZQQXX-mutant | 0 | - | - | - | - | - | - |'
```

## Comparison & Chart Output

Study analytics should remain usable from the terminal: comparison summaries
stay tabular, and chart mode still exposes a visible title and axis label.

```bash
../../tools/biomcp-ci study compare --study msk_impact_2017 --gene TP53 --type mutations --target ERBB2 | mustmatch like '# Study Group Comparison: Mutation Rate
| Group | N | Mutated | Mutation Rate |'
../../tools/biomcp-ci study query --study msk_impact_2017 --gene TP53 --type mutations --chart bar | mustmatch like 'TP53 mutation classes
Variant class'
```

A title containing an accidental terminal control should remain readable without
breaking the chart. The escaped input fixture invokes this same public command;
it does not replace BioMCP output.

```bash
bash ../fixtures/run-study-control-title.sh ../.. | mustmatch like 'Control Title
Variant class'
```

# Optional Section Outcomes

Optional sections keep their biomedical collection fields stable for scripts,
while a typed outcome distinguishes a confirmed zero from missing evidence. The
same entity-owned outcome also controls source attribution in `_meta`.

The retained variant `population.status` field uses the same four-state outcome
vocabulary as `section_outcomes.population`: `data`, `empty`, `unavailable`,
and `inapplicable`. It is not a separate provider-status vocabulary.

## A healthy empty is not an unavailable section

A successful Drugs@FDA lookup can truthfully return no approvals. The empty
`approvals` array remains compatible, and both the entity and provenance record
that OpenFDA successfully established the empty result.

```bash
../../tools/biomcp-ci --json get drug fixture-drug approvals \
  | jq '(.approvals == []) and (.section_outcomes.approvals == {"outcome":"empty","sources":["OpenFDA Drugs@FDA"]}) and ([._meta.section_sources[] | select(.key == "approvals") | {key,outcome,sources}] == [{"key":"approvals","outcome":"empty","sources":["OpenFDA Drugs@FDA"]}])' \
  | mustmatch 'true'
```

## Healthy-empty Markdown remains a confirmed zero

Human-readable output keeps the source-scoped zero claim when Drugs@FDA
successfully returns no approvals. It must not present this result as source
unavailability.

```bash
../../tools/biomcp-ci get drug fixture-drug approvals \
  | mustmatch like '## Drugs@FDA Approvals
No approvals found in Drugs@FDA'
../../tools/biomcp-ci get drug fixture-drug approvals \
  | mustmatch not '/(?i)unavailable/'
```

## Typed MCP get preserves the outcome

Agents using the typed MCP `get` tool receive the same entity-owned outcome as
CLI JSON. Transport through MCP does not erase the healthy-empty distinction.

```bash
bash ../fixtures/run-section-outcome-mcp.sh ../.. \
  | jq '(.approvals == []) and (.section_outcomes.approvals == {"outcome":"empty","sources":["OpenFDA Drugs@FDA"]})' \
  | mustmatch 'true'
```

## Drug interaction failure preserves a multi-section card

When DDInter is unavailable, a true multi-section drug request keeps completed
label evidence. Label interaction text makes the additive interaction outcome
`degraded`; a healthy label with no interaction text makes it `unavailable`.
Both states use the canonical outcome projection and never credit failed
DDInter.

```bash
BIOMCP_DDINTER_DIR="$BIOMCP_DDINTER_UNAVAILABLE_DIR" ../../tools/biomcp-ci get drug fixture-drug-label label interactions \
  | mustmatch like '## FDA Label
Fixture indication survives.
**Interactions status (DDInter / DrugBank / OpenFDA label):** degraded (partial/incomplete)
Label interaction evidence survives DDInter failure.'
BIOMCP_DDINTER_DIR="$BIOMCP_DDINTER_UNAVAILABLE_DIR" ../../tools/biomcp-ci get drug fixture-drug-label label interactions \
  | mustmatch not like '## Interactions (DDInter)'
BIOMCP_DDINTER_DIR="$BIOMCP_DDINTER_UNAVAILABLE_DIR" ../../tools/biomcp-ci get drug fixture-drug-empty label interactions \
  | mustmatch like '## FDA Label
Fixture indication survives.
**Interactions status (DDInter / DrugBank / OpenFDA label):** unavailable; no conclusion can be drawn'
```

JSON remains one valid drug document and has exactly one interaction provenance
entry agreeing with the entity-owned outcome.

```bash
BIOMCP_DDINTER_DIR="$BIOMCP_DDINTER_UNAVAILABLE_DIR" ../../tools/biomcp-ci --json get drug fixture-drug-label label interactions \
  | jq '(.name == "fixture-drug-label") and (.interaction_text | contains("Label interaction evidence")) and (.section_outcomes.interactions.outcome == "degraded") and (.section_outcomes.interactions.sources == ["OpenFDA label"]) and ([._meta.section_sources[] | select(.key == "interactions") | {outcome,sources}] == [{"outcome":"degraded","sources":["OpenFDA label"]}]) and ((tostring | test("SENSITIVE-UPSTREAM-DETAIL|ddinter-unavailable"; "i")) | not)' \
  | mustmatch 'true'
BIOMCP_DDINTER_DIR="$BIOMCP_DDINTER_UNAVAILABLE_DIR" ../../tools/biomcp-ci --json get drug fixture-drug-empty label interactions \
  | jq '(.name == "fixture-drug-empty") and (has("interaction_text") | not) and (.section_outcomes.interactions.outcome == "unavailable") and (.section_outcomes.interactions.sources == []) and ([._meta.section_sources[] | select(.key == "interactions") | {outcome,sources}] == [{"outcome":"unavailable","sources":[]}]) and ((tostring | test("SENSITIVE-UPSTREAM-DETAIL|ddinter-unavailable"; "i")) | not)' \
  | mustmatch 'true'
```

The reported `all` command also survives the unavailable optional source and
retains other completed outcomes.

```bash
BIOMCP_DDINTER_DIR="$BIOMCP_DDINTER_UNAVAILABLE_DIR" ../../tools/biomcp-ci --json get drug fixture-drug-label all \
  | jq '(.name == "fixture-drug-label") and (.label.indications | contains("Fixture indication survives")) and (.approvals == []) and (.shortage == []) and (.section_outcomes.safety.outcome == "empty") and (.section_outcomes.targets.outcome == "empty") and (.section_outcomes.indications.outcome == "empty") and (.section_outcomes.civic.outcome == "empty") and (.section_outcomes.interactions.outcome == "degraded")' \
  | mustmatch 'true'
```

All six provider-failure settlements are cards in CLI Markdown and JSON. The
status, surviving payload, contributor order, and retry are identical to the
frozen reducer matrix, including the conditional DrugBank contributor.

```bash
BIOMCP_DDINTER_DIR="$BIOMCP_DDINTER_UNAVAILABLE_DIR" ../../tools/biomcp-ci get drug fixture-drug-label interactions \
  | mustmatch like '**Interactions status (DDInter / DrugBank / OpenFDA label):** degraded (partial/incomplete) — Drug interaction evidence is incomplete because a source was unavailable.
Retry: `biomcp get drug fixture-drug-label interactions`
Label interaction evidence survives DDInter failure.'
BIOMCP_DDINTER_DIR="$BIOMCP_DDINTER_UNAVAILABLE_DIR" ../../tools/biomcp-ci get drug fixture-drug-empty interactions \
  | mustmatch like '**Interactions status (DDInter / DrugBank / OpenFDA label):** unavailable; no conclusion can be drawn — Drug interaction evidence is temporarily unavailable.
Retry: `biomcp get drug fixture-drug-empty interactions`'
BIOMCP_DDINTER_DIR="$BIOMCP_DDINTER_UNAVAILABLE_DIR" ../../tools/biomcp-ci get drug fixture-drug-empty-openfda-fail interactions \
  | mustmatch like '**Interactions status (DDInter / DrugBank / OpenFDA label):** unavailable; no conclusion can be drawn — Drug interaction evidence is temporarily unavailable.
Retry: `biomcp get drug fixture-drug-empty-openfda-fail interactions`'
BIOMCP_DDINTER_DIR="$BIOMCP_DDINTER_AVAILABLE_DIR" ../../tools/biomcp-ci get drug fixture-drug-ddinter-openfda-fail interactions \
  | mustmatch like '## Interactions (DDInter)
**Interactions status (DDInter / DrugBank / OpenFDA label):** degraded (partial/incomplete) — Drug interaction evidence is incomplete because a source was unavailable.
Retry: `biomcp get drug fixture-drug-ddinter-openfda-fail interactions`'
BIOMCP_DDINTER_DIR="$BIOMCP_DDINTER_AVAILABLE_DIR" ../../tools/biomcp-ci get drug fixture-drug-drugbank-openfda-fail interactions \
  | mustmatch like '## Interactions (DDInter)
**Interactions status (DDInter / DrugBank / OpenFDA label):** degraded (partial/incomplete) — Drug interaction evidence is incomplete because a source was unavailable.
DrugBank narrative survives OpenFDA failure.
Retry: `biomcp get drug fixture-drug-drugbank-openfda-fail interactions`'
BIOMCP_DDINTER_DIR="$BIOMCP_DDINTER_AVAILABLE_DIR" ../../tools/biomcp-ci get drug fixture-drug-empty-openfda-fail interactions \
  | mustmatch like '**Interactions status (DDInter / DrugBank / OpenFDA label):** unavailable; no conclusion can be drawn — Drug interaction evidence is temporarily unavailable.
Retry: `biomcp get drug fixture-drug-empty-openfda-fail interactions`'
for fixture in \
  "$BIOMCP_DDINTER_UNAVAILABLE_DIR fixture-drug-label degraded OpenFDA_label" \
  "$BIOMCP_DDINTER_UNAVAILABLE_DIR fixture-drug-empty unavailable none" \
  "$BIOMCP_DDINTER_UNAVAILABLE_DIR fixture-drug-empty-openfda-fail unavailable none" \
  "$BIOMCP_DDINTER_AVAILABLE_DIR fixture-drug-ddinter-openfda-fail degraded DDInter" \
  "$BIOMCP_DDINTER_AVAILABLE_DIR fixture-drug-drugbank-openfda-fail degraded DDInter,DrugBank" \
  "$BIOMCP_DDINTER_AVAILABLE_DIR fixture-drug-empty-openfda-fail unavailable none"; do
  read -r directory identity outcome source_key <<<"$fixture"
  sources=$(case "$source_key" in none) printf '[]';; OpenFDA_label) printf '["OpenFDA label"]';; DDInter) printf '["DDInter"]';; *) printf '["DDInter","DrugBank"]';; esac)
  markdown=$(BIOMCP_DDINTER_DIR="$directory" ../../tools/biomcp-ci get drug "$identity" interactions)
  case "$markdown" in *SENSITIVE-UPSTREAM-DETAIL*|*ddinter-unavailable*) exit 1;; esac
  BIOMCP_DDINTER_DIR="$directory" ../../tools/biomcp-ci --json get drug "$identity" interactions \
    | jq -e --arg outcome "$outcome" --argjson sources "$sources" --arg command "biomcp get drug $identity interactions" \
      '(.section_outcomes.interactions.outcome == $outcome) and (.section_outcomes.interactions.sources == $sources) and ([._meta.section_sources[] | select(.key == "interactions") | {outcome,sources}] == [{outcome:$outcome,sources:$sources}]) and ([._meta.next_commands[] | select(. == $command)] | length == 1) and ((tostring | test("SENSITIVE-UPSTREAM-DETAIL|ddinter-unavailable"; "i")) | not)' >/dev/null
done
printf 'six CLI failure settlements passed\n' | mustmatch 'six CLI failure settlements passed'
```

Both MCP entry points transport the same six settlements in Markdown and JSON
rather than replacing any of them with an error result.

```bash
BIOMCP_DDINTER_DIR="$BIOMCP_DDINTER_UNAVAILABLE_DIR" bash ../fixtures/run-section-outcome-mcp.sh ../.. section-outcome-interactions \
  | jq '(length == 3) and all(.[]; . as $row | .typed_json.section_outcomes.interactions as $outcome | (.typed_json == .raw_json) and (.typed_markdown == .raw_markdown) and ([.typed_json_error,.typed_markdown_error,.raw_json_error,.raw_markdown_error] | all(. == false)) and ([.typed_json._meta.section_sources[] | select(.key == "interactions") | {outcome,sources}] == [{outcome:$outcome.outcome,sources:$outcome.sources}]) and ([.typed_json._meta.next_commands[] | select(. == ("biomcp get drug " + $row.id + " interactions"))] | length == 1) and (.typed_markdown | contains("Retry: `biomcp get drug " + $row.id + " interactions`"))) and (.[0].typed_json.section_outcomes.interactions == {"outcome":"degraded","sources":["OpenFDA label"],"message":"Drug interaction evidence is incomplete because a source was unavailable."}) and (.[0].typed_markdown | contains("Label interaction evidence survives DDInter failure.")) and (.[1].typed_json.section_outcomes.interactions == {"outcome":"unavailable","sources":[],"message":"Drug interaction evidence is temporarily unavailable."}) and (.[2].typed_json.section_outcomes.interactions == {"outcome":"unavailable","sources":[],"message":"Drug interaction evidence is temporarily unavailable."}) and ((tostring | test("SENSITIVE-UPSTREAM-DETAIL|ddinter-unavailable"; "i")) | not)' \
  | mustmatch 'true'
BIOMCP_DDINTER_DIR="$BIOMCP_DDINTER_AVAILABLE_DIR" bash ../fixtures/run-section-outcome-mcp.sh ../.. section-outcome-interactions \
  | jq '(length == 3) and all(.[]; . as $row | .typed_json.section_outcomes.interactions as $outcome | (.typed_json == .raw_json) and (.typed_markdown == .raw_markdown) and ([.typed_json_error,.typed_markdown_error,.raw_json_error,.raw_markdown_error] | all(. == false)) and ([.typed_json._meta.section_sources[] | select(.key == "interactions") | {outcome,sources}] == [{outcome:$outcome.outcome,sources:$outcome.sources}]) and ([.typed_json._meta.next_commands[] | select(. == ("biomcp get drug " + $row.id + " interactions"))] | length == 1) and (.typed_markdown | contains("Retry: `biomcp get drug " + $row.id + " interactions`"))) and (.[0].typed_json.section_outcomes.interactions == {"outcome":"degraded","sources":["DDInter"],"message":"Drug interaction evidence is incomplete because a source was unavailable."}) and (.[0].typed_json.interactions | length == 1) and (.[1].typed_json.section_outcomes.interactions == {"outcome":"degraded","sources":["DDInter","DrugBank"],"message":"Drug interaction evidence is incomplete because a source was unavailable."}) and (.[1].typed_markdown | contains("DrugBank narrative survives OpenFDA failure.")) and (.[2].typed_json.section_outcomes.interactions == {"outcome":"unavailable","sources":[],"message":"Drug interaction evidence is temporarily unavailable."}) and ((tostring | test("SENSITIVE-UPSTREAM-DETAIL"; "i")) | not)' \
  | mustmatch 'true'
```

A logically interaction-only request now settles the same typed partial result.
Repeating the selector is idempotent, and the printed recovery command is the
same command recorded in JSON metadata.

```bash
BIOMCP_DDINTER_DIR="$BIOMCP_DDINTER_UNAVAILABLE_DIR" ../../tools/biomcp-ci --json get drug fixture-drug-label interactions \
  | jq '(.section_outcomes.interactions == {"outcome":"degraded","sources":["OpenFDA label"],"message":"Drug interaction evidence is incomplete because a source was unavailable."}) and ([._meta.next_commands[] | select(. == "biomcp get drug fixture-drug-label interactions")] | length == 1)' \
  | mustmatch 'true'
diff -u \
  <(BIOMCP_DDINTER_DIR="$BIOMCP_DDINTER_UNAVAILABLE_DIR" ../../tools/biomcp-ci --json get drug fixture-drug-label interactions | jq '{outcome:.section_outcomes.interactions,commands:._meta.next_commands}') \
  <(BIOMCP_DDINTER_DIR="$BIOMCP_DDINTER_UNAVAILABLE_DIR" ../../tools/biomcp-ci --json get drug fixture-drug-label interactions interactions | jq '{outcome:.section_outcomes.interactions,commands:._meta.next_commands}')
retry=$(BIOMCP_DDINTER_DIR="$BIOMCP_DDINTER_UNAVAILABLE_DIR" ../../tools/biomcp-ci get drug fixture-drug-label interactions | sed -n 's/^Retry: `\(biomcp .*\)`$/\1/p')
test "$retry" = "biomcp get drug fixture-drug-label interactions"
BIOMCP_DDINTER_DIR="$BIOMCP_DDINTER_UNAVAILABLE_DIR" ../../tools/biomcp-ci --json ${retry#biomcp } \
  | jq '(.section_outcomes.interactions.outcome == "degraded") and (.section_outcomes.interactions.sources == ["OpenFDA label"])' \
  | mustmatch 'true'
retry=$(BIOMCP_DDINTER_DIR="$BIOMCP_DDINTER_UNAVAILABLE_DIR" ../../tools/biomcp-ci get drug fixture-drug-empty interactions | sed -n 's/^Retry: `\(biomcp .*\)`$/\1/p')
test "$retry" = "biomcp get drug fixture-drug-empty interactions"
BIOMCP_DDINTER_DIR="$BIOMCP_DDINTER_UNAVAILABLE_DIR" ../../tools/biomcp-ci --json ${retry#biomcp } \
  | jq '(.section_outcomes.interactions.outcome == "unavailable") and (.section_outcomes.interactions.sources == [])' \
  | mustmatch 'true'
```

The pageable interaction report keeps DDInter as its required owner and still
fails in both Markdown and JSON when the bundle is unavailable.

```bash
bash -c 'set +e; output=$(BIOMCP_DDINTER_DIR="$BIOMCP_DDINTER_UNAVAILABLE_DIR" ../../tools/biomcp-ci drug interactions fixture-drug-label 2>&1); status=$?; set -e; test "$status" -eq 1; test "$output" = "Error: Source unavailable: DDInter is not available. Review source configuration and retry."; printf "pageable-markdown=error\n"' \
  | mustmatch 'pageable-markdown=error'
bash -c 'set +e; output=$(BIOMCP_DDINTER_DIR="$BIOMCP_DDINTER_UNAVAILABLE_DIR" ../../tools/biomcp-ci --json drug interactions fixture-drug-label); status=$?; set -e; printf "%s\n" "$output" | jq '\''(.error == {"code":"source_unavailable","message":"Source unavailable: DDInter is not available.","source":"DDInter","recovery":"Review source configuration and retry."}) and (.name == null) and ((tostring | test("ddinter-unavailable"; "i")) | not)'\''; printf "exit=%s\n" "$status"' \
  | mustmatch like 'true
exit=1'
```

When label acquisition fails, a sole interaction card keeps DDInter evidence
and credits DrugBank only when the retained row has its narrative. The required
label shapes still fail instead of starting interaction settlement. The native
`required_label_failures_make_zero_ddinter_ready_calls` test uses a direct
test-only DDInter observer, proves the observer with one positive control, then
asserts exactly zero DDInter ready calls for both required-label shapes.

```bash
BIOMCP_DDINTER_DIR="$BIOMCP_DDINTER_AVAILABLE_DIR" ../../tools/biomcp-ci get drug fixture-drug-ddinter-openfda-fail interactions \
  | mustmatch like '**Interactions status (DDInter / DrugBank / OpenFDA label):** degraded (partial/incomplete) — Drug interaction evidence is incomplete because a source was unavailable.
Retry: `biomcp get drug fixture-drug-ddinter-openfda-fail interactions`'
BIOMCP_DDINTER_DIR="$BIOMCP_DDINTER_AVAILABLE_DIR" ../../tools/biomcp-ci --json get drug fixture-drug-ddinter-openfda-fail interactions \
  | jq '(.interactions | length == 1) and (.section_outcomes.interactions.outcome == "degraded") and (.section_outcomes.interactions.sources == ["DDInter"]) and ((tostring | test("SENSITIVE-UPSTREAM-DETAIL"; "i")) | not)' \
  | mustmatch 'true'
BIOMCP_DDINTER_DIR="$BIOMCP_DDINTER_AVAILABLE_DIR" ../../tools/biomcp-ci --json get drug fixture-drug-drugbank-openfda-fail interactions \
  | jq '(.interactions[0].description == "DrugBank narrative survives OpenFDA failure.") and (.section_outcomes.interactions.outcome == "degraded") and (.section_outcomes.interactions.sources == ["DDInter","DrugBank"]) and ((tostring | test("SENSITIVE-UPSTREAM-DETAIL"; "i")) | not)' \
  | mustmatch 'true'
assert_required_label_failure() {
  mode=$1; shift
  set +e
  if test "$mode" = json; then
    output=$(BIOMCP_DDINTER_DIR="$BIOMCP_DDINTER_AVAILABLE_DIR" ../../tools/biomcp-ci --json get drug fixture-drug-empty-openfda-fail "$@")
  else
    output=$(BIOMCP_DDINTER_DIR="$BIOMCP_DDINTER_AVAILABLE_DIR" ../../tools/biomcp-ci get drug fixture-drug-empty-openfda-fail "$@" 2>&1)
  fi
  status=$?
  set -e
  test "$status" -eq 1
  case "$output" in *SENSITIVE-UPSTREAM-DETAIL*) return 1;; esac
  if test "$mode" = json; then
    printf '%s\n' "$output" | jq -e '.error == {"code":"api","message":"API request to OpenFDA failed.","source":"OpenFDA","recovery":"Retry the remote source."}' >/dev/null
  else
    test "$output" = 'Error: API request to OpenFDA failed. Retry the remote source.'
  fi
}
assert_required_label_failure markdown label interactions
assert_required_label_failure json label interactions
assert_required_label_failure markdown all
assert_required_label_failure json all
printf 'required-label failures abort before DDInter reads\n' \
  | mustmatch 'required-label failures abort before DDInter reads'
```

## Unrequested sections stay distinguishable

A base drug card does not call Drugs@FDA. Its registry records that omission as
`not_requested`, and provenance does not imply that approval evidence was queried.

```bash
../../tools/biomcp-ci --json get drug fixture-drug \
  | jq '(.section_outcomes.approvals == {"outcome":"not_requested","sources":[]}) and (._meta.section_sources | any(.key == "approvals") | not)' \
  | mustmatch 'true'
```

## Local outcomes do not credit providers

A requested enrichment cannot run when the resolved variant lacks that source's
required identifier. BioMCP records that local decision without claiming that
an uncontacted provider returned an empty result.

The separate feature-off AlphaGenome outcome is build-specific: a binary that
cannot predict says so before it reads credentials or considers coordinates.
That dual-build property is proven natively by
`list_variant_explains_alphagenome_availability_for_this_build`, rather than
by this profile-independent CLI page, which also runs against release builds.
The coordinate preflight itself still ships in every release binary and is
proven by a feature-on unit test.

| input | section | expected outcome | uncontacted provider | str:label |
|---|---|---|---|---|
| rs589001 | cbioportal | inapplicable | cBioPortal | cBioPortal needs a gene |
| rs589001 | civic | inapplicable | CIViC | CIViC needs a molecular profile |
| chr7:g.140453136A>T | gwas | inapplicable | GWAS Catalog | GWAS needs an rsID |

```bash each_row="Local outcomes do not credit providers"
biomcp --json --no-cache get variant '{{input}}' {{section}} \
  | jq '(.section_outcomes["{{section}}"].outcome == "{{expected_outcome}}") and (.section_outcomes["{{section}}"].sources == []) and (((.section_outcomes["{{section}}"].message // "") | length) > 0) and (._meta.section_sources | any(.key == "{{section}}" and .outcome == "{{expected_outcome}}" and .sources == [])) and (._meta.section_sources | all(.sources | index("{{uncontacted_provider}}") | not))' \
  | mustmatch 'true'
```

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

Typed MCP transports the same two drug documents rather than replacing either
with an error envelope.

```bash
BIOMCP_DDINTER_DIR="$BIOMCP_DDINTER_UNAVAILABLE_DIR" bash ../fixtures/run-section-outcome-mcp.sh ../.. section-outcome-interactions \
  | jq '(length == 2) and (.[0].section_outcomes.interactions.outcome == "degraded") and (.[0]._meta.section_sources | any(.key == "interactions" and .outcome == "degraded" and .sources == ["OpenFDA label"])) and (.[1].section_outcomes.interactions.outcome == "unavailable") and (.[1]._meta.section_sources | any(.key == "interactions" and .outcome == "unavailable" and .sources == []))' \
  | mustmatch 'true'
```

A logically interaction-only request remains a hard failure. Repeating the
same selector does not turn it into partial success.

```bash
bash -c 'set +e; output=$(BIOMCP_DDINTER_DIR="$BIOMCP_DDINTER_UNAVAILABLE_DIR" ../../tools/biomcp-ci --json get drug fixture-drug-label interactions); status=$?; set -e; printf "%s\n" "$output" | jq '\''(.error.code == "source_unavailable") and (.name == null)'\''; printf "exit=%s\n" "$status"' \
  | mustmatch like 'true
exit=1'
bash -c 'set +e; output=$(BIOMCP_DDINTER_DIR="$BIOMCP_DDINTER_UNAVAILABLE_DIR" ../../tools/biomcp-ci --json get drug fixture-drug-label interactions interactions); status=$?; set -e; printf "%s\n" "$output" | jq '\''(.error.code == "source_unavailable") and (.name == null)'\''; printf "exit=%s\n" "$status"' \
  | mustmatch like 'true
exit=1'
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

# Optional Section Outcomes

Optional sections keep their biomedical collection fields stable for scripts,
while a typed outcome distinguishes a confirmed zero from missing evidence. The
same entity-owned outcome also controls source attribution in `_meta`.

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

## Unrequested sections stay distinguishable

A base drug card does not call Drugs@FDA. Its registry records that omission as
`not_requested`, and provenance does not imply that approval evidence was queried.

```bash
../../tools/biomcp-ci --json get drug fixture-drug \
  | jq '(.section_outcomes.approvals == {"outcome":"not_requested","sources":[]}) and (._meta.section_sources | any(.key == "approvals") | not)' \
  | mustmatch 'true'
```

## An inapplicable lookup does not credit a provider

A requested prediction cannot run when the resolved variant lacks genomic HGVS
coordinates. BioMCP reports that local applicability decision without claiming
that AlphaGenome returned an empty result.

```bash
../../tools/biomcp-ci --json get variant rs589000 predict \
  | jq '(.section_outcomes.predict.outcome == "inapplicable") and (.section_outcomes.predict.sources == []) and (((.section_outcomes.predict.message // "") | length) > 0) and (._meta.section_sources | any(.key == "predict" and .outcome == "inapplicable" and .sources == [])) and (._meta.section_sources | all(.sources | index("AlphaGenome") | not))' \
  | mustmatch 'true'
```

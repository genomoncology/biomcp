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

## Inapplicable lookups do not credit providers

A requested enrichment cannot run when the resolved variant lacks that source's
required identifier. BioMCP reports the local applicability decision without
claiming that the uncontacted provider returned an empty result.

| input | section | uncontacted provider | str:label |
|---|---|---|---|
| rs589000 | predict | AlphaGenome | prediction needs genomic coordinates |
| rs589001 | cbioportal | cBioPortal | cBioPortal needs a gene |
| rs589001 | civic | CIViC | CIViC needs a molecular profile |
| chr7:g.140453136A>T | gwas | GWAS Catalog | GWAS needs an rsID |

```bash each_row="Inapplicable lookups do not credit providers"
biomcp --json --no-cache get variant '{{input}}' {{section}} \
  | jq '(.section_outcomes["{{section}}"].outcome == "inapplicable") and (.section_outcomes["{{section}}"].sources == []) and (((.section_outcomes["{{section}}"].message // "") | length) > 0) and (._meta.section_sources | any(.key == "{{section}}" and .outcome == "inapplicable" and .sources == [])) and (._meta.section_sources | all(.sources | index("{{uncontacted_provider}}") | not))' \
  | mustmatch 'true'
```

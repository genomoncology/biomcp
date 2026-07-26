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

## Local outcomes do not credit providers

A requested enrichment cannot run when the resolved variant lacks that source's
required identifier. Likewise, a binary compiled without AlphaGenome reports
that optional section as unavailable before it reads credentials or contacts a
provider. BioMCP reports either local decision without claiming that an
uncontacted provider returned an empty result.

Capability is decided before applicability. A binary that cannot predict says
so for every `predict` request, including inputs that would also have failed the
coordinate preflight. Reporting "needs genomic coordinates" from a build that
would refuse coordinates too invites a caller to retry an input that can never
succeed here. The coordinate preflight itself still ships in every release
binary and is proven by a feature-on unit test.

| input | section | expected outcome | uncontacted provider | str:label |
|---|---|---|---|---|
| rs589000 | predict | unavailable | AlphaGenome | feature-off prediction says it was not built even without coordinates |
| chr7:g.140453136A>T | predict | unavailable | AlphaGenome | feature-off prediction says it was not built |
| rs589001 | cbioportal | inapplicable | cBioPortal | cBioPortal needs a gene |
| rs589001 | civic | inapplicable | CIViC | CIViC needs a molecular profile |
| chr7:g.140453136A>T | gwas | inapplicable | GWAS Catalog | GWAS needs an rsID |

```bash each_row="Local outcomes do not credit providers"
biomcp --json --no-cache get variant '{{input}}' {{section}} \
  | jq '(.section_outcomes["{{section}}"].outcome == "{{expected_outcome}}") and (.section_outcomes["{{section}}"].sources == []) and (if "{{expected_outcome}}" == "unavailable" then ((.section_outcomes["{{section}}"].message // "") | test("not built"; "i")) else (((.section_outcomes["{{section}}"].message // "") | length) > 0) end) and (._meta.section_sources | any(.key == "{{section}}" and .outcome == "{{expected_outcome}}" and .sources == [])) and (._meta.section_sources | all(.sources | index("{{uncontacted_provider}}") | not))' \
  | mustmatch 'true'
```

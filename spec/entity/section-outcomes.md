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
  | jq '{approvals, section_outcome: .section_outcomes.approvals, section_source: (._meta.section_sources[] | select(.key == "approvals"))}' \
  | mustmatch like '{"approvals":[],"section_outcome":{"outcome":"empty","sources":["OpenFDA Drugs@FDA"]},"section_source":{"key":"approvals","outcome":"empty","sources":["OpenFDA Drugs@FDA"]}}'
```

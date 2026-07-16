# Protein Queries

Protein cards are the bridge between gene-scale identity and structure-scale
evidence. These batch-A canaries keep the reviewed search default, UniProt
identity card, deepen sections, and JSON follow-up contract stable.

## Positional Search & Table

Protein search should still echo the reviewed default and keep the result table
centered on accession, name, gene, and species.

```bash
../../tools/biomcp-ci search protein BRAF --limit 3 | mustmatch like '# Proteins: BRAF, reviewed=true
| Accession | Name | Gene | Species |
| P15056 | Serine/threonine-protein kinase B-raf | BRAF | Homo sapiens |'
```

## UniProt Identity Card

The default card should preserve the accession-level identity, provenance, and
gene follow-up that users need before diving into structures or complexes.

```bash
../../tools/biomcp-ci get protein P15056 | mustmatch like 'Accession: P15056
Source: UniProt
biomcp get gene BRAF'
```

## Protein Complexes

Complexes should stay readable as a bounded summary table plus compact member
bullets, rather than dumping an unstructured raw payload.

```bash
bash ../fixtures/setup-complexportal-spec-fixture.sh ../..
. ../../.cache/spec-complexportal-env
trap 'bash ../fixtures/cleanup-complexportal-spec-fixture.sh ../..' EXIT
../../tools/biomcp-ci get protein P15056 complexes | mustmatch like '## Complexes (ComplexPortal)
| ID | Name | Members | Curation |'
../../tools/biomcp-ci get protein P15056 complexes | mustmatch '/\| CPX-[0-9]+ \|/'
../../tools/biomcp-ci get protein P15056 complexes | mustmatch '/- `CPX-[0-9]+` members \([0-9]+\): [^\n]+/'
cat "$BIOMCP_COMPLEXPORTAL_FIXTURE_REQUEST_LOG" | mustmatch like 'GET /search/P15056 number=25 filters=species_f:("Homo sapiens")'
```

## Typed optional-section outcomes

Live protein enrichments retain one bounded state per requested section. The
same outcomes appear in provenance even when a provider has no rows to return.

```bash
../../tools/biomcp-ci --json get protein P15056 domains interactions complexes \
  | jq '[{"entity": .section_outcomes.domains.outcome, "provenance": (._meta.section_sources | map(select(.key == "domains")) | if length == 1 then .[0].outcome else null end)}, {"entity": .section_outcomes.interactions.outcome, "provenance": (._meta.section_sources | map(select(.key == "interactions")) | if length == 1 then .[0].outcome else null end)}, {"entity": .section_outcomes.complexes.outcome, "provenance": (._meta.section_sources | map(select(.key == "complexes")) | if length == 1 then .[0].outcome else null end)}] | all(.[]; (.entity | IN("data", "empty", "degraded", "unavailable")) and .entity == .provenance)' \
  | mustmatch 'true'
```

## Structures Follow-Up

Structure pagination should still render as a structures section with concrete
PDB-style rows rather than a generic blob of identifiers.

```bash
../../tools/biomcp-ci protein structures P15056 --limit 5 --offset 5 | mustmatch like '## Structures (PDB / AlphaFold via UniProt)'
../../tools/biomcp-ci protein structures P15056 --limit 5 --offset 5 | mustmatch '/\n- [0-9A-Z]{4} \(/'
```

## JSON Provenance & Next Commands

Structured output should keep the same deepen-path commands and also preserve
the evidence URLs that explain where the protein card came from.

```bash
../../tools/biomcp-ci --json get protein P15056 | mustmatch like '"next_commands": ['
../../tools/biomcp-ci --json get protein P15056 | jq -e '._meta.next_commands | index("biomcp get protein P15056 structures")' >/dev/null
../../tools/biomcp-ci --json get protein P15056 | jq -e '._provenance.evidence_urls | type == "array" and length > 0' >/dev/null
```

## JSON Search Next Commands

Protein search JSON should teach a script how to open the top UniProt result
instead of returning only a row array and pagination metadata.

```bash
../../tools/biomcp-ci --json search protein BRAF --limit 1 | mustmatch like '"next_commands":'
../../tools/biomcp-ci --json search protein BRAF --limit 1 | jq -e '._meta.next_commands[0] | test("^biomcp get protein .+$")' >/dev/null
../../tools/biomcp-ci --json search protein BRAF --limit 1 | jq -e '._meta.next_commands | any(. == "biomcp list protein")' >/dev/null
```

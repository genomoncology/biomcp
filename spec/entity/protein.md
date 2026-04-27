# Protein Queries

Protein cards are the bridge between gene-scale identity and structure-scale
evidence. These batch-A canaries keep the reviewed search default, UniProt
identity card, deepen sections, and JSON follow-up contract stable.

## Positional Search & Table

Protein search should still echo the reviewed default and keep the result table
centered on accession, name, gene, and species.

```bash
out="$(../../tools/biomcp-ci search protein BRAF --limit 3)"
echo "$out" | mustmatch like "# Proteins: BRAF, reviewed=true"
echo "$out" | mustmatch like "| Accession | Name | Gene | Species |"
echo "$out" | mustmatch like "| P15056 | Serine/threonine-protein kinase B-raf | BRAF | Homo sapiens |"
```

## UniProt Identity Card

The default card should preserve the accession-level identity, provenance, and
gene follow-up that users need before diving into structures or complexes.

```bash
out="$(../../tools/biomcp-ci get protein P15056)"
echo "$out" | mustmatch like "Accession: P15056"
echo "$out" | mustmatch like "Source: UniProt"
echo "$out" | mustmatch like "biomcp get gene BRAF"
```

## Protein Complexes

Complexes should stay readable as a bounded summary table plus compact member
bullets, rather than dumping an unstructured raw payload.

```bash
bash ../fixtures/setup-complexportal-spec-fixture.sh ../..
. ../../.cache/spec-complexportal-env
trap 'bash ../fixtures/cleanup-complexportal-spec-fixture.sh ../..' EXIT
out="$(../../tools/biomcp-ci get protein P15056 complexes)"
echo "$out" | mustmatch like "## Complexes (ComplexPortal)"
echo "$out" | mustmatch like "| ID | Name | Members | Curation |"
echo "$out" | mustmatch '/\| CPX-[0-9]+ \|/'
echo "$out" | mustmatch '/- `CPX-[0-9]+` members \([0-9]+\): [^\n]+/'
request_log="$(cat "$BIOMCP_COMPLEXPORTAL_FIXTURE_REQUEST_LOG")"
echo "$request_log" | mustmatch like 'GET /search/P15056 number=25 filters=species_f:("Homo sapiens")'
```

## Structures Follow-Up

Structure pagination should still render as a structures section with concrete
PDB-style rows rather than a generic blob of identifiers.

```bash
out="$(../../tools/biomcp-ci protein structures P15056 --limit 5 --offset 5)"
echo "$out" | mustmatch like "## Structures (PDB / AlphaFold via UniProt)"
echo "$out" | mustmatch '/\n- [0-9A-Z]{4} \(/'
```

## JSON Provenance & Next Commands

Structured output should keep the same deepen-path commands and also preserve
the evidence URLs that explain where the protein card came from.

```bash
json_out="$(../../tools/biomcp-ci --json get protein P15056)"
echo "$json_out" | mustmatch like '"next_commands": ['
echo "$json_out" | jq -e '._meta.next_commands | index("biomcp get protein P15056 structures")' >/dev/null
echo "$json_out" | jq -e '._provenance.evidence_urls | type == "array" and length > 0' >/dev/null
```

# Cell Line Queries

A cell line arrives under several spellings, and the same short name can belong
to two different lines. These routine contracts replay recorded Cellosaurus
responses through one fail-closed local server and pin the answers that keep a
join honest.

## Punctuation Does Not Change The Answer

`MV4;11` is the spelling a lab writes. Search should still land on the line
whose Cellosaurus identifier is `MV4-11`.

```bash
../../tools/biomcp-ci search cell-line 'MV4;11' | mustmatch like '# Cell lines: MV4;11
| Accession | Name | Species | Category | Disease | Match | Matched on |
| CVCL_0064 | MV4-11 | Homo sapiens (Human) | Cancer cell line | Childhood acute monocytic leukemia | exact | name |'
```

## Three Lines Share One Short Name

`KG1` matches three Cellosaurus records. All three are exact after
normalization, and the human identifier match leads.

```bash
../../tools/biomcp-ci search cell-line KG1 --json | jq -r '[.results[] | select(.match == "exact")] | length' | mustmatch '3'
../../tools/biomcp-ci search cell-line KG1 --json | jq -r '.results[0].accession' | mustmatch 'CVCL_0374'
```

## A Synonym Never Outranks An Identifier

`NB4` is the identifier of CVCL_0005 and a synonym of the neuroblastoma line
SJNB-4. Upstream order puts SJNB-4 first; ranking must not.

```bash
../../tools/biomcp-ci search cell-line NB4 --json | jq -r '.results[0].accession' | mustmatch 'CVCL_0005'
```

## A Source ID Resolves To The Accession

A DepMap ID is a valid way to name a line.

```bash
../../tools/biomcp-ci get cell-line ACH-000362 --json | jq -r '.accession' | mustmatch 'CVCL_2119'
```

## The Accession Opens Its Own Card

```bash
../../tools/biomcp-ci get cell-line CVCL_2119 --json | jq -r '.accession' | mustmatch 'CVCL_2119'
```

## Cross-References Are An Explicit Section

The base card never fetches cross-references, so the assertion needs the
section.

```bash
../../tools/biomcp-ci get cell-line CVCL_2119 xrefs --json | jq -c '.xrefs.depmap' | mustmatch '["ACH-000362"]'
```

## The Card Names Its Terms And Its Next Step

```bash
../../tools/biomcp-ci get cell-line CVCL_2119 | mustmatch like 'Cellosaurus 56.0 (2026-06-25), CC BY 4.0. Cite Bairoch A. J. Biomol. Tech. 29:25-38 (2018).
biomcp get cell-line CVCL_2119 xrefs'
```

## Curated Variants Are Published As Written

```bash
../../tools/biomcp-ci get cell-line CVCL_1844 variants | mustmatch like '## Variants (Cellosaurus)
| Gene | HGNC | Type | Description | Zygosity | PubMed |
| NPM1 | HGNC:7910 | Mutation | p.Trp288Cysfs*12 (c.860_863dupTCTG) | Heterozygous | 16079892 |'
```

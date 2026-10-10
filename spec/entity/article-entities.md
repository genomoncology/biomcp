# Article Entities

`article entities` turns one PMID's PubTator3 annotations into rows an agent
can follow. Every row carries the identifier PubTator3 assigned with its
namespace, so a mention can be opened by identifier instead of re-searched by
text. Rows that share a mention text but differ in identifier stay separate
rows, and rows that share an identifier but differ in text stay separate too:
the row is the mention group, not the registry entry.

The page runs against the provider-faithful PubTator3 captures for PMIDs
30738221 and 37887282, recorded from the public export endpoint and served
by the shared article fixture.

## Rows Carry Identifier and Namespace

```bash
../../tools/biomcp-ci --json article entities 30738221 \
  | jq '.annotations.genes == [
      {"text":"KRAS","count":12,"namespace":"NCBIGene","identifier":"3845"},
      {"text":"PD-L1","count":9,"namespace":"NCBIGene","identifier":"29126"},
      {"text":"programmed death ligand 1","count":1,"namespace":"NCBIGene","identifier":"29126"},
      {"text":"cytotoxic T-lymphocyte associated protein 4","count":1,"namespace":"NCBIGene","identifier":"1493"}
    ]' \
  | mustmatch 'true'
```

```bash
../../tools/biomcp-ci --json article entities 30738221 \
  | jq '.annotations.diseases == [
      {"text":"NSCLC","count":16,"namespace":"MESH","identifier":"MESH:D002289"},
      {"text":"tumor","count":3,"namespace":"MESH","identifier":"MESH:D009369"},
      {"text":"Non-Small Cell Lung Cancer","count":1,"namespace":"MESH","identifier":"MESH:D002289"}
    ] and (.annotations.mutations | map({text, namespace, identifier}) == [
      {"text":"G12A","namespace":"HGVS","identifier":"KRAS p.G12A"},
      {"text":"G12C","namespace":"HGVS","identifier":"KRAS p.G12C"},
      {"text":"G12D","namespace":"HGVS","identifier":"KRAS p.G12D"},
      {"text":"G12V","namespace":"HGVS","identifier":"KRAS p.G12V"},
      {"text":"G13C","namespace":"HGVS","identifier":"KRAS p.G13C"}
    ])' \
  | mustmatch 'true'
```

`PD-L1` and `programmed death ligand 1` are two mention groups for one NCBI
Gene identifier; both rows survive with `text` and `count` keeping their
meaning. The variant rows never carry the tmVar composite string. Every
variant row in this capture carries a gene annotation and a protein change,
so all five rows carry the gene-plus-HGVS form that names one allele: one
rsID can name several alleles (rs121913529 names three KRAS codon-12
alleles), the article may mention only one of them, and BioMCP has no
offline rsID-to-alleles table, so the row's own gene-plus-change pair is the
form that always names the row's allele. A row with no gene annotation and
no protein change keeps its rsID link, and one rsID can name several
alleles, so a gene-less single-mention row can print an rsID that opens a
different allele; BioMCP has no allele-specific form for that row.

## Compact JSON Omits Passage Positions

```bash
../../tools/biomcp-ci --json article entities 30738221 \
  | jq '[.annotations[] | .[] | has("positions")] | any' \
  | mustmatch 'false'
```

## Full JSON Adds Passage Positions

`--full` is the only view that carries passage positions. Positions are the
provider's document-global character offsets, one span per counted mention.

```bash
../../tools/biomcp-ci --json article entities 30738221 --full \
  | jq '(.annotations.genes[0].count == 12)
    and (.annotations.genes[0].positions | length == 12)
    and (.annotations.genes[0].positions[0] == {"offset":44,"length":4})
    and (.annotations.mutations[] | select(.text == "G12C") | .positions == [{"offset":1775,"length":4}])' \
  | mustmatch 'true'
```

## Rows Print a Get Command by Identifier Where BioMCP Accepts It

Disease MeSH identifiers resolve through the disease crosswalk and variant
rsIDs or gene-plus-HGVS forms parse as exact variant input, so those rows open
the record directly. Gene rows keep the text search because `get gene` takes
symbols, not NCBI Gene identifiers.

```bash
../../tools/biomcp-ci article entities 30738221 \
  | mustmatch '/\| NSCLC \| 16 \| `biomcp get disease MESH:D002289` \|/'
```

```bash
../../tools/biomcp-ci article entities 30738221 \
  | mustmatch '/\| tumor \| 3 \| `biomcp get disease MESH:D009369` \|/'
```

```bash
../../tools/biomcp-ci article entities 30738221 \
  | mustmatch '/\| KRAS \| 12 \| `biomcp search gene -q KRAS` \|/'
```

```bash
../../tools/biomcp-ci article entities 30738221 \
  | mustmatch not '/get gene 3845|get variant G12C|get variant p\.G12C|get variant rs121913530|get variant rs121913535/'
```

## Variant Rows Name Their Allele

rs121913529 names a KRAS codon-12 position, not one allele. The recorded
MyVariant response for that rsID carries three hits (G12A, G12D and G12V), and
`get variant rs121913529` opens G12D, so a row that mentions G12A or G12V
must not link through it. rs121913530 and rs121913535 also name several
alleles each — MyVariant lists G12S, G12R and G12C under the first and G13S,
G13R and G13C under the second — so their old links opened the row's allele
only when the provider's ranking picked it. Every variant row of this
capture carries a gene annotation and a protein change, so every variant row
prints the gene-plus-protein command, which names the row's own allele
whatever else the article mentions and whatever else its rsID names. A
document that mentions one allele of any of these rsIDs shows the same
single-change shape, and BioMCP has no offline rsID-to-alleles table, so the
row's own gene-plus-change form is the only link that names the row's allele.

```bash
../../tools/biomcp-ci article entities 30738221 \
  | mustmatch '/\| G12A \| 1 \| `biomcp get variant "KRAS p.G12A"` \|/'
```

```bash
../../tools/biomcp-ci article entities 30738221 \
  | mustmatch '/\| G12C \| 1 \| `biomcp get variant "KRAS p.G12C"` \|/'
```

```bash
../../tools/biomcp-ci article entities 30738221 \
  | mustmatch '/\| G12D \| 1 \| `biomcp get variant "KRAS p.G12D"` \|/'
```

```bash
../../tools/biomcp-ci article entities 30738221 \
  | mustmatch '/\| G12V \| 1 \| `biomcp get variant "KRAS p.G12V"` \|/'
```

```bash
../../tools/biomcp-ci article entities 30738221 \
  | mustmatch '/\| G13C \| 1 \| `biomcp get variant "KRAS p.G13C"` \|/'
```

```bash
../../tools/biomcp-ci article entities 30738221 \
  | mustmatch not '/get variant rs121913529/'
```

Each command opens its own variant through the recorded MyVariant gene+protein
responses of the routine variant-identity fixture. The rsID control row shows
what `rs121913529` alone opens: G12D, not the allele every other row names.

| query | id | protein |
|---|---|---|
| KRAS p.G12A | chr12:g.25398284C>G | p.Gly12Ala |
| KRAS p.G12C | chr12:g.25398285C>A | p.Gly12Cys |
| KRAS p.G12D | chr12:g.25398284C>T | p.Gly12Asp |
| KRAS p.G12V | chr12:g.25398284C>A | p.Gly12Val |
| KRAS p.G13C | chr12:g.25398282C>A | p.Gly13Cys |
| rs121913529 | chr12:g.25398284C>T | p.Gly12Asp |

```bash each_row="Variant Rows Name Their Allele"
biomcp --json --no-cache get variant '{{query}}' \
  | jq -c '{id: .id, protein: .hgvs_p}' \
  | mustmatch like '{"id":"{{id}}","protein":"{{protein}}"}'
```

## Spelled Gene Names Keep the Gene Form

PMID 37887282 spells the gene "K-Ras" and "K-RAS" in every human mention,
so no gene annotation text can head an exact gene-plus-change form. The
rows' own data still carries NCBI Gene 3845, and the recorded MyGene
batch-symbol response answers that identifier with the official symbol
KRAS, so every row prints the gene-plus-protein command. A spelled name must
not send the row to the rsID: rs121913530 names G12S, G12R and G12C, and
rs121913529 names G12A, G12D and G12V, so either link opens whichever allele
the provider ranks first, and the bare mention text is shorthand, not exact
input.

```bash
../../tools/biomcp-ci article entities 37887282 \
  | mustmatch '/\| G12C \| 1 \| `biomcp get variant "KRAS p.G12C"` \|/'
```

```bash
../../tools/biomcp-ci article entities 37887282 \
  | mustmatch '/\| G12V \| 1 \| `biomcp get variant "KRAS p.G12V"` \|/'
```

```bash
../../tools/biomcp-ci article entities 37887282 \
  | mustmatch '/\| G12D \| 1 \| `biomcp get variant "KRAS p.G12D"` \|/'
```

```bash
../../tools/biomcp-ci article entities 37887282 \
  | mustmatch not '/get variant rs121913529|get variant rs121913530|get variant G12[CVD]/'
```

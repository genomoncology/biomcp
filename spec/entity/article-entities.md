# Article Entities

`article entities` turns one PMID's PubTator3 annotations into rows an agent
can follow. Every row carries the identifier PubTator3 assigned with its
namespace, so a mention can be opened by identifier instead of re-searched by
text. Rows that share a mention text but differ in identifier stay separate
rows, and rows that share an identifier but differ in text stay separate too:
the row is the mention group, not the registry entry.

The page runs against the provider-faithful PubTator3 capture for PMID
30738221, recorded from the public export endpoint and served by the shared
article fixture.

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
      {"text":"G12A","namespace":"rsID","identifier":"rs121913529"},
      {"text":"G12C","namespace":"rsID","identifier":"rs121913530"},
      {"text":"G12D","namespace":"rsID","identifier":"rs121913529"},
      {"text":"G12V","namespace":"rsID","identifier":"rs121913529"},
      {"text":"G13C","namespace":"rsID","identifier":"rs121913535"}
    ])' \
  | mustmatch 'true'
```

`PD-L1` and `programmed death ligand 1` are two mention groups for one NCBI
Gene identifier; both rows survive with `text` and `count` keeping their
meaning. The variant rows carry the rsID PubTator3 normalizes to, never the
tmVar composite string.

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
rsIDs parse as exact variant input, so those rows open the record directly.
Gene rows keep the text search because `get gene` takes symbols, not NCBI
Gene identifiers.

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
  | mustmatch '/\| G12C \| 1 \| `biomcp get variant rs121913530` \|/'
```

```bash
../../tools/biomcp-ci article entities 30738221 \
  | mustmatch '/\| G13C \| 1 \| `biomcp get variant rs121913535` \|/'
```

```bash
../../tools/biomcp-ci article entities 30738221 \
  | mustmatch '/\| KRAS \| 12 \| `biomcp search gene -q KRAS` \|/'
```

```bash
../../tools/biomcp-ci article entities 30738221 \
  | mustmatch not '/get gene 3845|get variant G12C|get variant p\.G12C/'
```

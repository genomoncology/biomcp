# Disease Queries

Disease workflows are where BioMCP has to normalize human language onto stable
ontology IDs while still keeping treatment and diagnostic pivots close at hand.
These batch-A canaries focus on MONDO grounding, synonym rescue, section gating,
and executable follow-up guidance.

## Disease Normalization & Search

Direct disease search should still surface the canonical melanoma row with its
MONDO identifier visible in the result table.

```bash
out="$(../../tools/biomcp-ci search disease melanoma --limit 3)"
echo "$out" | mustmatch like "# Diseases: melanoma"
echo "$out" | mustmatch like "| MONDO:0005105 | melanoma |"
echo "$out" | mustmatch like "| ID | Name | Synonyms |"
```

## Synonym Rescue

Common synonym phrasing should resolve through the discover-plus-crosswalk path
instead of falling back to a dead-end “no diseases found” message.

```bash
out="$(../../tools/biomcp-ci search disease 'Arnold Chiari syndrome' --limit 3)"
echo "$out" | mustmatch like "Resolved via discover + crosswalk"
echo "$out" | mustmatch like "MONDO:0000115"
```

## Canonical Disease Card

The default card should expose the persistent ID, top cross-entity summaries,
and the executable next steps for trials, articles, diagnostics, and drugs.

```bash
out="$(../../tools/biomcp-ci get disease melanoma)"
echo "$out" | mustmatch like "ID: MONDO:0005105"
echo "$out" | mustmatch like "Recruiting Trials (ClinicalTrials.gov):"
echo "$out" | mustmatch like 'biomcp search trial -c "melanoma"'
echo "$out" | mustmatch like 'biomcp search drug --indication "melanoma"'
```

## Genes & Diagnostics

`genes` and `diagnostics` stay opt-in sections, but when requested they should
render as explicit tables and admit that the diagnostic list is truncated.

```bash
out="$(../../tools/biomcp-ci get disease 'Lynch syndrome' genes diagnostics)"
echo "$out" | mustmatch like "## Associated Genes"
echo "$out" | mustmatch like "| Gene | Relationship | Source | OpenTargets |"
echo "$out" | mustmatch like "## Diagnostics"
echo "$out" | mustmatch '/Showing [0-9]+ of [0-9]+ diagnostic matches/'
```

## Clinical Features

Clinical features are a separate opt-in MedlinePlus section for reviewed
configured diseases. Uterine leiomyoma should render source-native symptom rows
with reviewed HPO mappings instead of falling back to the broader phenotype
section or a blank table.

```bash
out="$(../../tools/biomcp-ci get disease "uterine leiomyoma" clinical_features)"
echo "$out" | mustmatch like "## Clinical Features (MedlinePlus)"
echo "$out" | mustmatch like "| Rank | Feature | HPO | Confidence | Evidence | Source |"
echo "$out" | mustmatch like "heavy menstrual bleeding"
echo "$out" | mustmatch like "HP:0000132 (Menorrhagia)"
echo "$out" | mustmatch like "[MedlinePlus](https://medlineplus.gov/uterinefibroids.html)"
```

## NIH Funding Context

Funding belongs in its own section. The card should keep that view truthful and
bounded instead of implying the first page is the whole research landscape.

```bash
out="$(../../tools/biomcp-ci get disease 'Marfan syndrome' funding)"
echo "$out" | mustmatch like "## Funding (NIH Reporter)"
echo "$out" | mustmatch like "| Project | PI | Organization | FY | Amount |"
echo "$out" | mustmatch '/Showing top [0-9]+ unique grants from [0-9]+ matching NIH project-year records/'
```

## JSON Pivots

The JSON card should keep the same executable disease follow-ups that the
markdown card teaches to humans.

```bash
json_out="$(../../tools/biomcp-ci --json get disease melanoma)"
echo "$json_out" | mustmatch like '"next_commands": ['
echo "$json_out" | jq -e '._meta.next_commands | index("biomcp search trial -c \"melanoma\"")' >/dev/null
echo "$json_out" | jq -e '._meta.suggestions | index("biomcp search diagnostic --disease \"melanoma\"")' >/dev/null
```

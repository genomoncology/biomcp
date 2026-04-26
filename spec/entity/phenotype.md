# Phenotype Queries

Phenotype search is where BioMCP turns symptom language or HPO IDs into a
ranked disease shortlist that a human can inspect further. These canaries keep
the input grammar, ranking table, and disease follow-up path visible.

## Symptom-Phrase Search

Free-text symptom phrases should still resolve into a ranked disease table
instead of an opaque backend-specific response.

```bash
out="$(../../tools/biomcp-ci search phenotype 'seizure, developmental delay' --limit 3)"
echo "$out" | mustmatch like "# Phenotype Search: seizure, developmental delay"
echo "$out" | mustmatch like "| Disease ID | Disease Name | Similarity Score |"
echo "$out" | mustmatch '/\| MONDO:[^|]+ \| .+ \| [0-9.]+ \|/'
```

## HPO ID Input

HPO IDs should use the same phenotype search surface, so operators do not have
to learn a second command for ontology-backed inputs.

```bash
out="$(../../tools/biomcp-ci search phenotype 'HP:0001250 HP:0001263' --limit 3)"
echo "$out" | mustmatch like "# Phenotype Search: HP:0001250 HP:0001263"
echo "$out" | mustmatch like "| Disease ID | Disease Name | Similarity Score |"
```

## Disease Follow-Up Guidance

The phenotype surface should still teach the next typed command so the user can
open the top disease hit with genes and phenotypes in one step.

```bash
out="$(../../tools/biomcp-ci search phenotype 'seizure, developmental delay' --limit 3)"
echo "$out" | mustmatch '/See also:[\s\S]*biomcp get disease ".+" genes phenotypes/'
echo "$out" | mustmatch '/biomcp get disease ".+" genes phenotypes/'
```

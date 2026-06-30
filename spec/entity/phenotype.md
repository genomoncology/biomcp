# Phenotype Queries

Phenotype search is where BioMCP turns symptom language or HPO IDs into a
ranked disease shortlist that a human can inspect further. These canaries keep
the input grammar, ranking table, and disease follow-up path visible.

## Symptom-Phrase Search

Free-text symptom phrases should still resolve into a ranked disease table
instead of an opaque backend-specific response.

```bash
../../tools/biomcp-ci search phenotype 'seizure, developmental delay' --limit 3 | mustmatch like '# Phenotype Search: seizure, developmental delay
| Disease ID | Disease Name | Similarity Score |'
../../tools/biomcp-ci search phenotype 'seizure, developmental delay' --limit 3 | mustmatch '/\| MONDO:[^|]+ \| .+ \| [0-9.]+ \|/'
```

## HPO ID Input

HPO IDs should use the same phenotype search surface, so operators do not have
to learn a second command for ontology-backed inputs.

```bash
../../tools/biomcp-ci search phenotype 'HP:0001250 HP:0001263' --limit 3 | mustmatch like '# Phenotype Search: HP:0001250 HP:0001263
| Disease ID | Disease Name | Similarity Score |'
```

## Disease Follow-Up Guidance

The phenotype surface should still teach the next typed command so the user can
open the top disease hit with genes and phenotypes in one step.

```bash
../../tools/biomcp-ci search phenotype 'seizure, developmental delay' --limit 3 | mustmatch '/See also:[\s\S]*biomcp get disease ".+" genes phenotypes/'
../../tools/biomcp-ci search phenotype 'seizure, developmental delay' --limit 3 | mustmatch '/biomcp get disease ".+" genes phenotypes/'
```

## JSON Search Next Commands

Phenotype search JSON should teach the same disease follow-up as markdown, so
script callers can open the top disease match without guessing a nonexistent
`get phenotype` command.

```bash
../../tools/biomcp-ci --json search phenotype 'HP:0001250 HP:0001263' --limit 1 | mustmatch like '"next_commands":'
../../tools/biomcp-ci --json search phenotype 'HP:0001250 HP:0001263' --limit 1 | jq -e '._meta.next_commands[0] | test("^biomcp get disease .+ genes phenotypes$")' >/dev/null
../../tools/biomcp-ci --json search phenotype 'HP:0001250 HP:0001263' --limit 1 | jq -e '._meta.next_commands | any(. == "biomcp list phenotype")' >/dev/null
```

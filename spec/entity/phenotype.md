# Phenotype Queries

Phenotype search turns symptom language or HPO IDs into a ranked disease shortlist. These captured contracts use the shipped CLI against fresh HPO and Monarch responses served by the supervised routine fixture.

## Captured Symptom-Phrase Route

The routine fixture resolves the symptom phrase through HPO search and then replays the exact Monarch similarity request produced by those identifiers.

```bash
../../tools/biomcp-ci search phenotype 'seizure, developmental delay' --limit 3 | mustmatch like '# Phenotype Search: seizure, developmental delay
| Disease ID | Disease Name | Similarity Score |
MONDO:0007367
febrile seizures, familial, 1'
```

## Captured HPO-ID Route

Direct HPO IDs skip phrase resolution and use the same similarity and rendering path.

```bash
../../tools/biomcp-ci search phenotype 'HP:0001250 HP:0001263' --limit 3 | mustmatch like '# Phenotype Search: HP:0001250 HP:0001263
| Disease ID | Disease Name | Similarity Score |
MONDO:0010450
intellectual disability, X-linked 89'
```

## Disease Follow-Up Guidance

The captured phrase result teaches the typed disease command for its top match.

```bash
../../tools/biomcp-ci search phenotype 'seizure, developmental delay' --limit 3 | mustmatch like 'See also:
biomcp get disease "febrile seizures, familial, 1" genes phenotypes'
```

## Captured JSON Follow-Up Envelope

JSON callers receive the same typed disease follow-up without an unsupported phenotype-get command.

```bash
../../tools/biomcp-ci --json search phenotype 'HP:0001250 HP:0001263' --limit 1 | jq '._meta.next_commands | any(startswith("biomcp get disease ") and endswith(" genes phenotypes"))' | mustmatch 'true'
```

## Observed Phenotype Provider Requests

The fixture fails closed outside the recorded HPO query and the exact Monarch term sets and limits.

```bash
grep -F 'GET /hpo/search?q=seizure' "$BIOMCP_DISEASE_SURVIVAL_REQUEST_LOG" | mustmatch like 'search?q=seizure'
grep -F 'GET /monarch/v3/api/semsim/search/HP:0001250,HP:0001263/Human%20Diseases?limit=3' "$BIOMCP_DISEASE_SURVIVAL_REQUEST_LOG" | mustmatch like 'Diseases?limit=3'
grep -F 'GET /monarch/v3/api/semsim/search/HP:0001250,HP:0001263/Human%20Diseases?limit=1' "$BIOMCP_DISEASE_SURVIVAL_REQUEST_LOG" | mustmatch like 'Diseases?limit=1'
```

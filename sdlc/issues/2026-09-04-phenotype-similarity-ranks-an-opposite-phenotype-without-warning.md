# Phenotype similarity ranks an opposite phenotype without warning

BioMCP 0.9.0-dev.6 returned isolated microcephaly as the first candidate disease for an exact macrocephaly HPO query:

```text
$ biomcp --no-cache search phenotype "HP:0000256" --limit 5
| MONDO:0043137 | isolated microcephaly | 14.772 |
| MONDO:0019419 | X-linked intellectual disability-macrocephaly-macroorchidism syndrome | 12.707 |
```

The result survives a no-cache request. `biomcp get disease MONDO:0043137 clinical_features` reports only `HP:0000252 Microcephaly` for the first disease. A control query for `HP:0001250` returns seizure disorders, so the command does use the supplied term.

Term normalization works. `biomcp discover "macrocephaly"` returns `HP:0000256 Macrocephaly` as an exact match and labels related HPO candidates separately. The mismatch appears in the Monarch semantic-similarity result or BioMCP's presentation of that result. Nearby ontology concepts can receive a high similarity score even when their clinical meanings conflict.

The CLI currently calls every row a candidate disease and recommends opening the first row. The JSON omits the resolved HPO terms and does not distinguish a direct phenotype association from ontology proximity. An agent can turn the top row into a false phenotype match.

## Desired behavior

Keep semantic similarity available, but prevent an opposite phenotype from looking like a direct match. Show the resolved HPO identifiers and labels in Markdown and JSON. Mark results as semantic neighbors when the disease lacks the submitted phenotype. Add a clear warning when a top result contains a known opposite term such as macrocephaly versus microcephaly. If reliable detection would require an unsafe heuristic, stop recommending the top row as a match and explain that a reviewer must inspect its phenotype annotations.

## Success criteria

- `search phenotype "HP:0000256"` does not recommend isolated microcephaly as an unqualified top disease match.
- Structured output includes the submitted or resolved HPO identifiers and labels.
- Markdown and JSON distinguish semantic similarity from a confirmed phenotype association.
- Free-text `search phenotype "macrocephaly"` follows the same contract after term resolution.
- A regression test fixes the provider response and proves the warning or qualification without calling a live service.

Found on 2026-09-04 during a rare disease case research exercise.

---
flow: build
priority: 11
deps: ["1022", "1035"]
---
# Use one vocabulary for a section's outcome

In a single response on the development build, the `population` object reports `"status": "missing"` while `section_outcomes.population` reports `"outcome": "inapplicable"` for the same condition. Two keys, two vocabularies, one state.

The four-state contract of `data`, `empty`, `unavailable`, and `inapplicable` is the valuable thing in this design, and it is undermined by a second word appearing in the same payload for the same fact. A caller writing against the contract has to learn which key to trust, and a caller who guesses wrong gets a state that is not in the documented set.

## Done when

- A given section's outcome is reported with one vocabulary across the whole response.
- Any key that survives uses a value from the documented four-state set, or is documented as a distinct concept with its reason for existing stated.
- The documented contract and the emitted payload agree, and a test pins that agreement so a third vocabulary cannot appear later.

## Existing tests that pin this

The `missing` vocabulary is asserted in shipped tests. Restatement is authorized in these files, for these tests by name, only to the extent they assert the string `missing` as a population status:

- `src/render/markdown/variant/tests.rs` — `variant_population_markdown_keeps_missing_status_compact`
- `src/entities/variant/get/tests.rs` — `population_status_json_keeps_explicit_null_exome_and_genome_results`

The producing code is `GnomadPopulationStatus` in `src/entities/variant/get.rs`, and the documented contract is `spec/entity/section-outcomes.md`, which may be updated to match whichever vocabulary is chosen.

No other test file is authorized. If the design stage finds a further shipped assertion naming a fifth state and it is not listed above, say so in the design output rather than restating it.

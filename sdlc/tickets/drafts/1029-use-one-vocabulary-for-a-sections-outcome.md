---
flow: quickfix
priority: 3
hold: draft for review; do not promote until Ian releases this
---
# Use one vocabulary for a section's outcome

In a single response on the development build, the `population` object reports `"status": "missing"` while `section_outcomes.population` reports `"outcome": "inapplicable"` for the same condition. Two keys, two vocabularies, one state.

The four-state contract of `data`, `empty`, `unavailable`, and `inapplicable` is the valuable thing in this design, and it is undermined by a second word appearing in the same payload for the same fact. A caller writing against the contract has to learn which key to trust, and a caller who guesses wrong gets a state that is not in the documented set.

## Done when

- A given section's outcome is reported with one vocabulary across the whole response.
- Any key that survives uses a value from the documented four-state set, or is documented as a distinct concept with its reason for existing stated.
- The documented contract and the emitted payload agree, and a test pins that agreement so a third vocabulary cannot appear later.

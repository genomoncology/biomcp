---
flow: build
priority: 8
---

# Lowercase "not", "and" and "or" in a criteria phrase invert the query and disable verification

The boolean-operator regex is case-insensitive:

```rust
Regex::new(r"(?i)\b(OR|AND|NOT)\b")
```

So `--criteria "patients not previously treated"` parses as `"patients" NOT "previously treated"`. The user asks for untreated patients and the query excludes the thing they asked about.

The same phrase then does something worse. `has_boolean_operators` answers true for it, and `src/entities/trial/search/eligibility.rs:255` and `:265` skip client-side eligibility verification whenever it does. So the inverted query runs and its results come back unverified, presented as matches.

Ordinary clinical English carries these words constantly. "Patients not previously treated", "measurable disease and adequate organ function", "chemotherapy or radiotherapy" all read as operator expressions today.

Verified in `src/entities/trial/search/essie.rs` and `eligibility.rs` on 2026-09-02 against `0.9.0-dev.6`.

## Required behavior

A criteria phrase written in ordinary English is searched as the user wrote it.

A user who intends a boolean expression can still write one, and the way to do so is unambiguous.

Eligibility verification is skipped only when the query genuinely carries an operator the user meant, never because a phrase happened to contain an English word.

## Done, observably

- `--criteria "patients not previously treated"` searches for that phrase and does not exclude "previously treated".
- The same phrase does not disable client-side eligibility verification.
- A deliberate boolean expression still parses as one.

## Correct behavior

Only uppercase OR, AND, and NOT are operators. A lowercase word is a search term. The operator test used for the query and the one used for eligibility verification are the same test.

Write that as a failing test, then fix. Red before green.

The assertion to write: A lowercase phrase containing and, or, not produces a query with no operator. The query builder and the eligibility check agree on whether a given criteria string contains an operator, for every string in the corpus.

This behavior is held against both 0.9 and 1.0 alike. It is recorded as case 15 of the clinical-trial conformance cases in the sibling BioData repository. **An attempt cannot read that repository**, so the statement above is this ticket's own authoritative copy, reconciled against the case when the ticket was filed. If it looks wrong, stop and say so rather than implementing something different.

## Boundary

Do not change the essie escaping of a phrase that carries no operator. Do not change what eligibility verification checks once it runs; this ticket is about when it is skipped. The article-search stopword lists are unrelated and stay as they are.

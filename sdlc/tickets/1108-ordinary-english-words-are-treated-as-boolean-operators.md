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

## Where correct behavior is written

`repos/biodata/sdlc/planning/clinical-trial-conformance/cases.json`, case 15. That file is the shared statement of correct behavior for this defect, held against both 0.9 and 1.0 so the two cannot drift into disagreeing about what correct means.

Take the assertion from that case, write it as a failing test, then fix. Red before green.

**Do not copy the expected behavior into this repository as a second statement of it.** Reference the case. If the case's expected behavior looks wrong, stop and say so rather than implementing something different; that disagreement gets settled in the case file, not in this codebase.

Reported by the BioData lead in `notes/biomcp/feedback/2026-09-02-seventeen-trial-defects-to-fix-in-0-9.md`, defect 15 of seventeen, verified against BioMCP `60f5377e` and re-verified here against `0.9.0-dev.6` on 2026-09-02.
## Boundary

Do not change the essie escaping of a phrase that carries no operator. Do not change what eligibility verification checks once it runs; this ticket is about when it is skipped. The article-search stopword lists are unrelated and stay as they are.

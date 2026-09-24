# Carry DDInter coverage and fix the drug card

Split from ticket 1238. Source issue:
`sdlc/issues/2026-09-23-drug-card-reports-ddinter-not-covered-as-no-interactions.md`.

## Problem

`get drug <name> interactions` prints "no matching rows… 0 of 0" for a
drug the DDInter bundle does not cover, dropping the coverage status
that `drug interactions` reports correctly. Name matching misses
synonyms (a row filed under acetylsalicylic acid is not found from
aspirin) and can resolve a combination product. The freshness label is
re-read from file times while the index is cached for the process life,
so a long-running server can label old rows fresh. A corrupt bundle
surfaces as a generic "API request failed" without naming the file.

## Design

1. Carry the coverage status onto the drug card so an uncovered drug
   reads as not covered, not as zero interactions.
2. Match on synonyms alongside the query, resolved name, and brands;
   add a test pairing aspirin with acetylsalicylic acid.
3. Tie the freshness label to the loaded index instead of re-reading
   file times per render.
4. Name the bundle file in the corrupt-bundle error.
5. Check the 8 MB per-file download cap against the real bundle's file
   sizes and raise it if the real data exceeds it (record the number).
6. The real-bundle run stays with the M5 verification leg and is
   recorded here when it runs.

## Acceptance

- An uncovered drug on the card reports not covered.
- The aspirin/acetylsalicylic-acid synonym test passes.
- Freshness comes from the loaded index; the corrupt error names the
  file; the cap decision is recorded.

## Review

- Design review: pending
- Code review: pending

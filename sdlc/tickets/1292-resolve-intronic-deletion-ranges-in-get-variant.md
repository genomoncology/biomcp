# 1292 — resolve intronic deletion ranges in get variant

Proposed 2026-10-03 by the BioMCP owner for the next 0.9 release. Status: accepted, ready to build.

## Evidence

Experiment 432 question clinvar-09 names `NM_000249.4(MLH1):c.678-14_678-3del`. Two agent legs could not reach it: `get variant` did not parse it and it did not appear in MLH1 search listings.

## Change

1. Record the exact failing commands and errors on 0.9.1.
2. Accept transcript-qualified deletion ranges with intronic offsets, or refuse with a message that prints a working form, such as a ClinVar VariationID or rsID lookup.
3. Accept a ClinVar VariationID as a `get variant` input if it is not accepted today.

## Retained behavior

Every input form that works today keeps working.

## Proof

A table of input forms (substitution, deletion range, intronic offsets, VariationID, rsID) with expected results, as a spec page.

## Deferred

HGVS parsing moves to a shared parser in 1.0. This ticket fixes the 0.9 behavior only.

## Review

- Design review: ACCEPT 2026-10-03 on the first pass, dispatch 26de9b8f (fresh SWE-2 researcher, read-only).

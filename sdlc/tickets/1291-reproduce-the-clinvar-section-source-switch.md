# 1291 — reproduce the ClinVar section's source switch

Proposed 2026-10-03 by the BioMCP owner for the next 0.9 release.

Status: OPEN.

## Outcome

The ClinVar section's fallback to MyVariant.info is reproduced and explained: the degraded message names why NCBI ClinVar did not answer and how old the fallback copy is.

## Evidence

- Starts from: Experiment 432's diagnosis reports that the same `get variant ... clinvar` command served the ClinVar section from NCBI ClinVar in some runs and from MyVariant.info in others, and that the MyVariant.info copy carried years-old Uncertain significance records (cells c/13, d/04, d/15, d/16). The diagnosis reran each of four variants once, and all four returned NCBI ClinVar. The owner reran one variant twice with the same result. The switch is not yet reproduced. Code reading by the ticket reviewer shows the mechanism: `add_clinvar` wraps the NCBI call in an 8 s timeout (`src/entities/variant/get.rs:61`, `src/entities/variant/mod.rs:435-438`), and on any error or timeout it serves the MyVariant.info record as a `degraded` section (`mod.rs:401-413`).
- Keeps: The fallback itself stays. Agents still get an answer when NCBI is slow.
- Changes: See Change detail.
- Proof: A spec case with a recorded NCBI timeout that shows the fallback label. The reproduction log goes in the record.
- Defers: Changing which source is primary.

## Change detail

1. Reproduce: rerun the four cells' exact commands 10 times each, and record which source answered and why (timeout, rate limit, fallback rule).
2. The fallback already carries `source: "MyVariant.info"` and a `degraded` outcome. Keep the error instead of collapsing it to `Err(())` (`mod.rs:435-438`), and add the reason to the degraded message: timeout, rate limit or HTTP error.
3. Report the fallback copy's newest evaluation date, so a reader can see how old it may be.

## Review

- Design review: ACCEPT 2026-10-03 on the second pass, dispatch 26de9b8f (fresh SWE-2 researcher, read-only). The first pass returned FIX; the revision addressed every finding.

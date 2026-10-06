# 1291 — reproduce the ClinVar section's source switch

Proposed 2026-10-03 by the BioMCP owner for the next 0.9 release.

Status: OPEN.
Milestone: 0.9.2

## Outcome

The ClinVar section's fallback to MyVariant.info is reproduced and explained: the degraded message names why NCBI ClinVar did not answer and how old the fallback copy is.

## Evidence

- Starts from: Experiment 432's diagnosis reports that the same `get variant ... clinvar` command served the ClinVar section from NCBI ClinVar in some runs and from MyVariant.info in others, and that the MyVariant.info copy carried years-old Uncertain significance records (cells c/13, d/04, d/15, d/16). The diagnosis reran each of four variants once, and all four returned NCBI ClinVar. The owner reran one variant twice with the same result. The switch is not yet reproduced. Code reading by the ticket reviewer shows the mechanism: `add_clinvar` wraps the NCBI call in an 8 s timeout (`src/entities/variant/get.rs:61`, `src/entities/variant/mod.rs:435-438`), and on any error or timeout it serves the MyVariant.info record as a `degraded` section (`mod.rs:401-413`).
- Keeps: The fallback itself stays. Agents still get an answer when NCBI is slow.
- Changes: See Change detail.
- Proof: A spec case with a recorded NCBI timeout that shows the fallback label. The reproduction log goes in the record.
- Defers: Changing which source is primary.

## Reproduced 2026-10-05

Experiment 439 (`~/workspace/experiments/439-reproduce-1293-p2-and-1291-switch/`), binary at main a877443f, live sources through a logging ClinVar proxy. The four exact commands from experiment 432, 10 runs each. The switch did not reproduce: d/04 (DICER1 577152), d/15 (RAD50 230605) and d/16 (TP53 428884) answered NCBI ClinVar 10 of 10 each; 31 efetch calls took 266–1,394 ms (median 358 ms), far inside the 8 s timeout. c/13's form (`get variant 1463720 clinvar -j`) exits 2 `invalid_argument` on main: a bare VariationID is refused input, ticket 1292's gap confirmed live. The fallback path was validated under forced NCBI errors: source MyVariant.info, outcome degraded, message "Direct ClinVar retrieval is unavailable; showing MyVariant.info fallback data." — the message cannot distinguish timeout from rate limit from HTTP error, the `Err(())` collapse at `src/entities/variant/mod.rs:455-457`. The trigger window stands as an intermittent NCBI answer slower than 8 s (`OPTIONAL_ENRICHMENT_TIMEOUT`, `src/entities/variant/get.rs:59-63`), which today's runs never approached.

## Change detail

1. Reproduce: rerun the four cells' exact commands 10 times each, and record which source answered and why (timeout, rate limit, fallback rule).
2. The fallback already carries `source: "MyVariant.info"` and a `degraded` outcome. Keep the error instead of collapsing it to `Err(())` (`mod.rs:435-438`), and add the reason to the degraded message: timeout, rate limit or HTTP error.
3. Report the fallback copy's newest evaluation date, so a reader can see how old it may be.

## Review

- Design review: ACCEPT 2026-10-03 on the second pass, dispatch 26de9b8f (fresh SWE-2 researcher, read-only). The first pass returned FIX; the revision addressed every finding.

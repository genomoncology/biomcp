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
- Proof: A spec case with a held-open NCBI efetch answer that shows the fallback label. The fixture is synthetic; see the 2026-10-07 note for why a live recording is not reproducible. The reproduction log goes in the record.
- Defers: Changing which source is primary.

## Reproduced 2026-10-05

Experiment 439 (`~/workspace/experiments/439-reproduce-1293-p2-and-1291-switch/`), binary at main a877443f, live sources through a logging ClinVar proxy. The four exact commands from experiment 432, 10 runs each. The switch did not reproduce: d/04 (DICER1 577152), d/15 (RAD50 230605) and d/16 (TP53 428884) answered NCBI ClinVar 10 of 10 each; 31 efetch calls took 266–1,394 ms (median 358 ms), far inside the 8 s timeout. c/13's form (`get variant 1463720 clinvar -j`) exits 2 `invalid_argument` on main: a bare VariationID is refused input, ticket 1292's gap confirmed live. The fallback path was validated under forced NCBI errors: source MyVariant.info, outcome degraded, message "Direct ClinVar retrieval is unavailable; showing MyVariant.info fallback data." — the message cannot distinguish timeout from rate limit from HTTP error, the `Err(())` collapse at `src/entities/variant/mod.rs:455-457`. The trigger window stands as an intermittent NCBI answer slower than 8 s (`OPTIONAL_ENRICHMENT_TIMEOUT`, `src/entities/variant/get.rs:59-63`), which today's runs never approached.

## Change detail

1. Reproduce: rerun the four cells' exact commands 10 times each, and record which source answered and why (timeout, rate limit, fallback rule).
2. The fallback already carries `source: "MyVariant.info"` and a `degraded` outcome. Keep the error instead of collapsing it to `Err(())` (`mod.rs:435-438`), and add the reason to the degraded message: timeout, rate limit or HTTP error.
3. Report the fallback copy's newest evaluation date, so a reader can see how old it may be.

## Review fold 2026-10-07

Finding 9 of the 2026-10-07 review of the work since 0.9.1 landed on this ticket: `src/entities/variant/mod.rs` grew past its 1000-line floor behind a hand-added inventory entry, the mutation attacks (`.max()` to `.min()`, mapping the timeout to an HTTP error) passed every test, a sustained 429 could read "timed out", non-429 failures all read "HTTP error", and 2021-02-30 passed the date gate. The fold:

1. Extract, not grow: the ClinVar section owner (direct retrieval, `ClinvarDirectFailure` and its wording, the fallback age gate, the degraded labeling, and their tests) moved to `src/entities/variant/clinvar.rs`; `variant/mod.rs` returned to 738 lines, and `tools/update-rust-source-size-inventory` regenerated the inventory so the hand-added entry for `mod.rs` is gone and `ncbi_efetch.rs` keeps its exact existing pin. The new module is 807 lines, under the floor, so it needs no entry. The packaged source grows by that one file (1399 to 1400).
2. The timeout wording is honestly merged: "NCBI ClinVar timed out (slow answer or sustained rate limiting)". A sustained 429 whose Retry-After hold outlives the deadline classifies as a deadline miss, and the label admits it, because the rate-limit waits (the Retry-After retry sleep and the in-process rate limiter) run inside the dropped send future and leave no signal at the `add_clinvar` seam. Pretending a pure timeout, or inventing a RateLimited label the seam cannot know, would both be wrong; the fast-429 refusal still reads "rate limited" because the response arrives before the deadline.
3. Non-429 failures read "NCBI ClinVar request failed": transport errors, non-success HTTP statuses, and decode or parse failures share the class, so no non-429 failure claims HTTP error when the failure was not one. The class renamed from `HttpError` to `ProviderError` to match what it holds.
4. Tests now fail the review's attacks: a multi-row fallback picks its true newest dated row (`.max()` to `.min()` fails), `add_clinvar` drives the deadline miss end to end against a held-open fixture server through `BIOMCP_CLINVAR_BASE` (mapping the miss to another class fails), a 429 with Retry-After outliving the deadline pins the merged wording, a fast 429 and a 500 each pin their distinct labels end to end, and the day gate requires a real calendar day (2021-02-30, 2021-02-29, 2021-04-31 refused; 2020-02-29 accepted).
5. The spec fixture is synthetic and says so on the page: experiment 439's 31 recorded efetch calls all answered within 1.4 s, no provider control forces an eight-second server hold, and the trigger never reproduced live, so a recorded exchange of the timeout path is not reproducible on demand. The recorded evidence for the seam is experiment 439's fast-answer exchanges; the fixture holds the NCBI answer open to exercise the code-confirmed path.

## Review

- Design review: ACCEPT 2026-10-03 on the second pass, dispatch 26de9b8f (fresh SWE-2 researcher, read-only). The first pass returned FIX; the revision addressed every finding.

## Build status

- Built on branch `tickets/1291-clinvar-source-switch-labeling`
  through ef33a1473 plus the fold-record count fix, 2026-10-07, across
  two timeout revivals with checkpoints (nothing lost).
- Code review: ACCEPT 2026-10-07
  The full post-fix-round review (the one the adversarial review said
  was missing; recorded on main at 990cba43b) covered the branch
  through 7026e9a77, the fold-record count fix. Verified: the extraction
  is complete and mechanical with the inventory regenerated by the tool
  (mod.rs at 738, no hand entry, ncbi_efetch.rs at the 1322 this ticket's
  own authorization raised); every mutation attack genuinely encoded
  (max-to-min, deadline-to-error, the calendar gate with the Gregorian
  leap rule); the sustained-429 merged wording reasoned and documented at
  the enum, spec page, and ticket; the synthetic fixture stated plainly
  and fail-closed; the package count 1400 measured on that tree. One P2
  folded: a stale module line count in the review fold record, corrected
  on the branch.
- Wait-ratchet delta re-review (bb4d52250 on the pre-rebase branch):
  PENDING, needs a fresh reviewer. The ACCEPT above predates the delta;
  it must cover the watchdog comments on the deadline test clock checks,
  the clinvar.rs wait-inventory entry, and the 43→46 marker-ceiling
  raise.

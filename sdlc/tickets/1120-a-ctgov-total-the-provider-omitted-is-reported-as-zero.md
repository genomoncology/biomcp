---
flow: build
priority: 5
---

# A ClinicalTrials.gov total the provider did not supply is reported as exactly zero

`src/entities/trial/search/ctgov.rs:956`:

```rust
let total = resp.total_count.unwrap_or(0) as usize;
return Ok(ctgov_count_from_native_total(total, filters.age.is_some()));
```

`total_count` is optional on the wire. When the provider does not supply it, the caller is told there are exactly zero matching trials. `ctgov_count_from_native_total` at `:639` returns `Exact` or `Approximate` and has no way to say the provider did not answer.

Verified against `0.9.0-dev.6` on 2026-09-03.

## Required behavior

A result total ClinicalTrials.gov did not state is reported as unknown. A total it did state is reported as that number. This holds whether or not an age filter is set: an absent total cannot be approximated any more than it can be counted.

Write that as a failing test, then fix. Red before green.

This behavior is held against both 0.9 and 1.0 alike. It is recorded as case 9 of the clinical-trial conformance cases in the sibling BioData repository. **An attempt cannot read that repository**, so the statement above is this ticket's own authoritative copy. If it looks wrong, stop and say so rather than implementing something different.

## What already exists, so it is not built twice

`TrialCount::Unknown` is not dead code and this ticket does not introduce the variant. It is constructed at `src/entities/trial/search/ctgov.rs:667`, `:871` and `:966`, and rendered at `src/cli/trial/dispatch.rs:223` as `total: null` in JSON and at `:232` as `Total: unknown (traversal limit reached)` in text. A test pins it at `src/entities/trial/search/ctgov/tests.rs:840`.

The JSON surface is complete and needs nothing. The text surface does not say what this ticket needs it to say; see the boundary.

## The evidence, and its limit

`testdata/sources/ctgov/search_phelan_next_20260811.json` carries only `studies` and `nextPageToken`, with no `totalCount`. Its receipt at entry 219 of `testdata/sources/capture-receipts.json` is classified `real_and_receipted`, its recorded request URL contains `countTotal=true`, and its minimization note reads "preserving the omitted later-page total." The provider omitted a total on a request that asked for one, and someone deliberately preserved that.

The limit, stated plainly: that is a **later-page** response, and the line above reads a **first-page** response. No recorded first-page response omits `totalCount`.

That provenance is why we believe the provider can omit the field. Whether the code can be shown mishandling it is a separate question, and the answer is yes, today.

## The proof, and what constrains it

Two rules bind the proof. Design chooses how to satisfy them.

**The red must be an assertion failure, not a compile error.** Unit-testing `ctgov_count_from_native_total` directly would require widening its first parameter before the test compiles, which proves a signature is missing rather than a behavior wrong.

**Recorded bytes must not be served in answer to a request that never produced them.** Returning the later-page capture to a first-page count request would pair real bytes with a request that did not yield them, which is worse than an honestly synthetic body. A composed first-page body with the field omitted is a synthetic body reproducing a real optionality, and that is an acceptable answer. Neither is a recorded first-page omission, and this ticket does not pretend otherwise.

Two routes already exist, and design may use either or another that satisfies the rules. `src/sources/clinicaltrials.rs:11` defines `BIOMCP_CTGOV_BASE`, and `src/entities/disease/get/tests.rs` already drives the whole CLI against a local fixture server through it, writing CTGov bytes back — including a literal `{"studies":[],"totalCount":0}` at line 71 — and calling `crate::cli::execute`. Separately, `spec/entity/trial.md:134` already pins the `--count-only` text output as a routine contract, exercised by `make spec`; that page is this repository's owned contract surface for this command.

## Done, observably

- A count request whose response carries no `totalCount` renders `total: null` in JSON and a text total that does not state a number.
- The same holds when an age filter is set. Today that path renders an approximate zero.
- A response carrying a total still renders that number, and the approximate rendering under an age filter is unchanged when a total is present.

## Boundary

A count that stopped at the traversal cap must still tell the caller that is why. That outcome is protected.

The string at `src/cli/trial/dispatch.rs:232` reads `Total: unknown (traversal limit reached)`. It names a cause, and today `Unknown` has exactly one cause, so it is true. This ticket gives it a second cause with a different reason: the provider did not supply a total and no traversal limit was reached. Left as it is, the tool would state a reason that did not happen.

So that line may change, and the variant may gain a way to carry its reason if design chooses that. What may not happen is a rendering that asserts a cause which does not apply, or one that stops telling a capped count why it was capped.

Do not change paging or how many rows a search requests.

This ticket is ClinicalTrials.gov only. The NCI half of case 9 is retired; see below.

## The NCI half is retired, not deferred

The original ticket also claimed the NCI branch reports a total of one because `src/entities/trial/search/mod.rs:372` falls back to `page.results.len()` on a single-row request.

That claim is withdrawn. Case 9 in the sibling BioData repository carries its own correction of 2026-09-02: measured against the live provider, every NCI response carries `total` at the top level, in both fifty-record and one-record samples, and the case concludes the NCI half may not be producible. Both recorded NCI payloads in this repository agree, carrying totals of 2094 and 2112.

No NCI response observed so far omits the total, so there is no failing test to write. Making that branch return unknown would be a guard against a shape nothing has produced.

The fallback at `mod.rs:372` stays as written. Nothing is filed for it and nothing is waiting on it.

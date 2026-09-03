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

A total ClinicalTrials.gov did not supply is reported as unknown, not as the number zero.

## Correct behavior

A result total the provider did not state is reported as unknown. A total the provider did state is reported as that number.

Write that as a failing test, then fix. Red before green.

This behavior is held against both 0.9 and 1.0 alike. It is recorded as case 9 of the clinical-trial conformance cases in the sibling BioData repository. **An attempt cannot read that repository**, so the statement above is this ticket's own authoritative copy. If it looks wrong, stop and say so rather than implementing something different.

## What already exists, so it is not built twice

`TrialCount::Unknown` is not dead code and this ticket does not introduce it. It is constructed at `src/entities/trial/search/ctgov.rs:667`, `:871` and `:966`, and it is rendered at `src/cli/trial/dispatch.rs:223` as `total: null` in JSON and at `:232` as `Total: unknown` in text. Tests pin it at `src/entities/trial/search/ctgov/tests.rs:840`.

The reporting half of this fix is already built and working. What is missing is that the one path above never reaches it.

## The evidence, and its limit

`testdata/sources/ctgov/search_phelan_next_20260811.json` carries only `studies` and `nextPageToken`, with no `totalCount`. Its receipt at entry 219 of `testdata/sources/capture-receipts.json` is classified `real_and_receipted`, its recorded request URL contains `countTotal=true`, and its minimization note reads "preserving the omitted later-page total." The provider omitted a total on a request that asked for one, and someone deliberately preserved that.

The limit, stated plainly: that is a **later-page** response, and the line above reads a **first-page** response. No recorded first-page response omits `totalCount`.

So this fix is defensive. It rests on two things: the field is `Option` on the wire, and the provider is on record omitting it when asked on a neighbouring request. That is the ticket's position, and it is not a claim that a first-page omission has been observed. Do not treat the absence of a first-page example as a reason to refuse; treat it as the reason the ticket says "defensive" rather than "reproduced in production."

## Done, observably

- Converting a response with no `totalCount` yields an unknown total rather than `Exact(0)`.
- A response carrying a total still reports that number, and the `Approximate` behavior under an age filter is unchanged.

## Boundary

Do not change paging or how many rows a search requests.

Do not change the rendering of an unknown total. It already works and this ticket does not touch it.

This ticket is ClinicalTrials.gov only. The NCI half of case 9 is retired; see below.

## The NCI half is retired, not deferred

The original ticket also claimed the NCI branch reports a total of one because `src/entities/trial/search/mod.rs:372` falls back to `page.results.len()` on a single-row request.

That claim is withdrawn. Case 9 in the sibling BioData repository carries its own correction of 2026-09-02: measured against the live provider, every NCI response carries `total` at the top level, in both fifty-record and one-record samples. Both recorded NCI payloads in this repository agree, carrying totals of 2094 and 2112. A live request on 2026-09-03 agreed again.

No NCI response omits the total, so there is no failing test to write and no payload that could ever be recorded to produce one. Making that branch return unknown would be a guard against a shape the provider does not produce.

The fallback at `mod.rs:372` stays as written. Nothing is filed for it and nothing is waiting on it.

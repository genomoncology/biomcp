---
flow: build
priority: 8
---

# A degraded interactions section prints a retry command that fails

## Goal

`get drug <identity> interactions` renders the existing typed interaction outcome whenever base drug identity resolves. That makes the canonical recovery command executable without inventing an unrelated second section or discarding surviving evidence.

Observed at commit `f68d8832` with the release binary `0.9.0-dev.6+gf68d8832` on 2026-09-05, using an empty directory as `BIOMCP_DDINTER_DIR` to make DDInter unavailable:

```text
$ biomcp get drug imatinib label interactions
## Interactions

**Interactions status (DDInter / DrugBank / OpenFDA label):** degraded (partial/incomplete) — Drug interaction evidence is incomplete because a source was unavailable.
Retry: `biomcp get drug "imatinib mesylate" interactions`
## Additive Label Text (OpenFDA)
...

$ biomcp get drug "imatinib mesylate" interactions
Error: Source unavailable: DDInter is not available. Review source configuration and retry.
$ echo $?
1
```

The same `Retry:` line is printed from `biomcp get drug imatinib all`. The JSON form of the printed command returns the `source_unavailable` error envelope instead of a card, also with exit 1.

The gene sections behave the way the affordance intends. Under forced source failures, the printed recovery command for `pathways`, `ontology`, `diseases`, `interactions`, and `expression` was run for `TP53`; each rendered its section with the degraded or unavailable status and exited 0.

## Why the two contracts collide

Record 1098 requires that "a logically sole explicit `interactions` request must keep its current `source_unavailable` error and exit 1", and lists retries as out of scope. Record 1103, landed later the same day, prints a sole-section retry for every degraded or unavailable registered section. Neither record cites the other. The recovery affordance therefore points at the one request shape that is contractually required to refuse.

## The contract to settle

Do not change the retry to `label interactions`, `all`, or an arbitrary second section. `label interactions` makes OpenFDA mandatory and hard-fails before DDInter is evaluated when the label request fails. `all` has the same required-label failure and adds unrelated providers. A second selector such as `targets interactions` happens to enable partial mode, but performs unrelated work solely to change error handling.

Instead, reverse the narrow record 1098 boundary for the `get drug` card only. `populate_card_interactions` already computes the truthful typed outcome matrix through `apply_interactions_result`; a logically sole selector must render that result instead of propagating the DDInter report error. The separate `biomcp drug interactions <name>` report helper retains its hard-failure contract.

The resulting card distinguishes:

- DDInter failure with surviving label evidence: `degraded`, attributed to the OpenFDA label.
- DDInter evidence with label failure: `degraded`, attributed to DDInter / DrugBank.
- One source healthy-empty while the other fails: `unavailable`.
- Both sources fail: `unavailable`.
- Both sources are healthy-empty: `empty`.
- Evidence with no source failure: `data`.

## Done, observably

- Deterministic cases prove: DDInter failure plus surviving label evidence yields `degraded` with `OpenFDA label` attribution; surviving DDInter evidence plus OpenFDA label-acquisition failure yields `degraded` with DDInter / DrugBank attribution; and both failures yield `unavailable` with no sources. Each sole `get drug <identity> interactions` request renders a card and exits 0 once base identity resolves.
- Markdown asserts the rendered state and provider attribution. CLI JSON, raw MCP, and equivalent typed-MCP requests assert that `section_outcomes` agrees with `_meta.section_sources` and that upstream failure details do not leak.
- The command printed beside a degraded or unavailable interaction status, and the same command in `_meta.next_commands`, are executed under the same injected failure rather than merely parsed or retried against a healthy fixture.
- A repeated `interactions interactions` selector has the same semantics as one selector; duplication does not accidentally toggle partial handling.
- Every other entity's and every other drug section's printed recovery command keeps the behavior it has today.

## Boundary

This ticket changes the sole-section outcome contract for the `get drug` card and its recovery affordance. It does not change the separate `biomcp drug interactions` report helper, DDInter coverage or synchronization, timeouts, retry middleware, or any other section's recovery route.

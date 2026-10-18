---
priority: 2
---

# 2204 — Review every disabled-source surface

Status: OPEN.

Milestone: 0.9.4

Reviews: revision a03d835, accept

## Outcome

After 2196 lands, an agent test review checks every entity, section, typed MCP tool and health row with sources disabled for any call or wrong reason, and audits every source's commercial terms record.

- Evidence: the 0.9.3 release review sampled these surfaces and found the defects 2196 fixes.
- Changes: `sdlc/records/2204-disabled-source-surface-review.md` `sdlc/tickets/2204-disable-sources-surface-review.md`. This ticket records the bounded review; product/terms fixes are separate tickets.
- Proof: recorded traced-network results; file defects as tickets.

## Progress

- 2026-10-18 started

Coordinator pre-execution scope: claim `sdlc/records/2204-disabled-source-surface-review.md` and this ticket only. Review all current source registry ids, discovery-derived entities/sections/selectors, typed MCP tools and health rows with traced local replies; distinguish reached, carried, blocked and fixture-missing cases rather than reporting early all-off exits as full coverage. Reuse existing qualified2196/2207 cases for unchanged inputs and use current replay/environment isolation without adding a fixture framework. For strict blocked-egress proof, an occasional Linux build job is authorized for this ticket after the initial2191 baseline; ordinary builds remain on the designated build host. Verify every commercial-terms entry against official terms and cite precise sources; null is not proof of open terms. Report evidence gaps and concrete defects for new tickets, do not expand into product fixes or infer clinical rules. Before execution give the bounded matrix and stop rule; preserve raw traces in a numbered owned experiment until the short report is accepted.

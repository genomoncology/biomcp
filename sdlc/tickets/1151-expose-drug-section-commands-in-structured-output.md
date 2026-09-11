---
flow: build
priority: 7
deps: [1161]
---

# Expose drug section commands in every card surface

## Outcome

Drug cards expose the same useful, executable follow-up commands in single and
batch JSON and Markdown. Today JSON includes only related-entity pivots, while
Markdown independently suggests some unloaded sections. A default eflornithine
card therefore omits accepted commands for approvals, label, and regulatory
detail.

## Current facts

- Single JSON builds `_meta.next_commands` in `src/cli/drug/render.rs` from
  recovery plus `related_drug`; it never adds unloaded drug sections.
- Single Markdown separately calls `sections_drug` and `related_drug`.
- Generic drug batches independently use `related_drug` for JSON and
  `drug_markdown` for Markdown, so those surfaces can disagree too.
- Ticket 1161 is complete. Its sole-interactions card now makes the existing
  recovery command executable.
- The current candidate families have disjoint rendered prefixes, so a natural
  duplicate cannot be forced through `drug_command_discovery` today. Exact
  owner projections therefore prove category wiring; the private flattening
  boundary has a direct mutation-sensitive first-wins/cap test.

## Scope

Add one internal drug command-discovery projection used by all four rendering
paths. It returns categorized recovery, section, aggregate, and related
commands plus one flattened `next_commands` list.

Loaded means selected by the request, not incidentally populated by a
provider. The default request has `targets` loaded. Explicit sections load
themselves. `all` uses the current parser expansion; legacy US-only
`approvals` remains discoverable unless explicitly requested.

Candidate order is:

1. recovery for requested degraded or unavailable sections, using registry
   order;
2. up to three unloaded sections in `approvals`, `label`, `regulatory`,
   `safety`, `shortage`, `interactions`, `indications`, `targets`, `civic`
   order;
3. `all` when it would load something new; and
4. the existing related commands in their existing order.

Deduplicate exact rendered command strings, first occurrence wins, then cap
the flattened list at ten. Markdown renders only surviving categorized
commands. Build every command through `NextCommand` quoting. Regional section,
recovery, and aggregate commands carry the resolved `--region`; WHO excludes
safety and shortage.
Generic batches retain their current US acquisition behavior.

Do not change drug parsing, `all` expansion, default acquisition, provider
requests, outcome classification, evidence, related-command generation, MCP
schemas or tool count, search output, or the separate pageable interaction
report.

## Acceptance

- Pure projection tests assert exact categorized and flattened results for
  default, explicit sections, `all`, recovery, US/EU/WHO/all regions,
  deduplication, blank identity, and the ten-command cap. Because current
  candidate families are disjoint, deduplication is proven by the exact owner
  projections plus a direct private-boundary test that forces duplicate exact
  bytes and verifies first-wins behavior before the cap.
- Production-path tests prove single and two-item batch JSON/Markdown agree,
  preserve item order and resolved identities, and make no extra provider
  request for discovery.
- Parse every emitted command with the real CLI parser; execute representative
  section, aggregate, recovery, and related commands against the existing
  provider fixture.
- Raw MCP and typed single-card MCP inherit the CLI projection. Raw MCP batch
  inherits both item projections. Typed MCP schema and tool inventory remain
  unchanged.
- Update the drug schema example, user guide, CLI reference, and executable
  drug spec. Run focused tests, `make lint`, `make test`, `make spec`, package
  the exact 1,300-file source archive, and finish with `git diff --check`.

## Complexity

- Contract score: 1
- State and timing score: 0
- Reach score: 1
- Proof score: 1
- Cost of error score: 1
- Total: 4
- Minimum level floor: none
- Final level: 2
- Reasons: one existing public metadata field must agree across several
  renderers; the behavior is local and stateless, but wrong commands are a
  user-visible break and need several surface checks.
- Selected model: GPT-5.6 Luna High for implementation.

## Review

- Design review: ACCEPT on current main; no material findings. The reviewer
  confirmed the rendering owner, dependency, scope, acceptance, and level 2
  Luna High route.
- Code review: REJECT on `cd3fef3b`. The projection logic is focused, but the
  candidate lacks production-path batch/MCP/command-execution evidence and
  several exact pure cases required above. Remediation pending.

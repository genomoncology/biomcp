---
flow: build
priority: 6
---

# One unavailable source discards four working sections

`biomcp get drug mercaptopurine all` exits 1 and prints nothing at all when DDInter is unavailable. Verified against the repository build `biomcp 0.9.0-dev.6` (`./target/debug/biomcp`) on 2026-09-01:

```
$ biomcp get drug mercaptopurine all
Error: Source unavailable: DDInter is not available. Review source configuration and retry.
exit 1, 0 bytes on stdout
```

`label`, `safety`, `targets` and `approvals` all succeed when requested individually. Asking for all of them returns none of them.

The mechanism to do better already exists in the codebase and this path does not use it. The pharmacogenomics JSON envelope carries `section_outcomes` with an outcome vocabulary built for exactly this situation.

The cost is measurable. In a 31-task study driving BioMCP through its MCP interface on 2026-09-01, an agent hit this failure, then recovered by dropping the failing section and re-requesting the four that work. It reached the right answer and spent extra calls doing by hand what the tool could have done in one.

Mercaptopurine is the thiopurine used in paediatric acute lymphoblastic leukaemia maintenance, so this is not an obscure entry point.

## Required behavior

A request for several sections returns the sections that succeeded and names the ones that did not, together with why.

A caller can tell from the output which sections it is holding and which are missing, without running each section separately to find out.

The exit status distinguishes a request that returned nothing from one that returned part of what was asked.

## Done, observably

- `get drug mercaptopurine all` prints the sections that resolve while DDInter is unavailable, and marks the interactions section unavailable.
- The same holds in JSON: a consumer can read which sections carry data and which failed.
- A request in which every section fails is still reported as a failure.
- The behaviour is not specific to drugs or to DDInter.

## Boundary

This ticket does not add retries, does not change any source's timeout, and does not change what a single-section request does when that one section fails. The related question of printing the exact command that recovers a degraded or capped section is filed separately; see `sdlc/issues/2026-08-27-degraded-and-capped-sections-should-print-their-recovery-command.md`.

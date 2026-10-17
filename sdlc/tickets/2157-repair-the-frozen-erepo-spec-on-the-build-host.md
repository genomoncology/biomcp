# 2157 — Repair the frozen ERepo executable specification

Status: OPEN.

Milestone: 0.9.3

## Outcome

The frozen ERepo executable specification passes through the routine build job on the supported build host, preserving its summary, detail and typed MCP source-fact assertions.

- Starts from: `spec/entity/clingen-erepo.md` and its frozen runner. Job `1792238883286-80627` on the reviewed autocomplete repair passed lint and the routine tests, then failed both ERepo blocks at lines 24 and 28 with runner `clingen-erepo-frozen` exiting 1. The autocomplete repair does not modify this page or runner.
- Keeps: frozen provider facts, explicit failure propagation, typed MCP behavior, and all other routine specifications. Do not weaken assertions to manufacture a pass.
- Changes: claims `spec/entity/clingen-erepo.md` `CHANGELOG.md`. Investigate the actual runner and report exact additional files before editing implementation; the coordinator narrows this claim before code changes.
- Proof: reproduce the two existing executable-spec failures on current main through the build job before editing; record the exact command and output. Repair their identified cause, run the affected spec page, then the routine merged-candidate checks. Existing failed executable contracts provide the regression coverage; add a focused contract only if needed to preserve the fix.
- Defers: historical failure analysis and broad fixture restructuring. This ticket only clears the concrete main check failure.

PM decision: 2026-10-17. Ian approved main repairs before trigger shutdown. This is a reviewed recovery lane while ordinary landings are held. No new clinical or biological interpretation is authorized; if the failure requires a source-fact decision, report it for clinical review.

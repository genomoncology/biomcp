# 2157 — Repair the frozen ERepo executable specification

Status: OPEN.

Milestone: 0.9.3

## Outcome

The frozen ERepo executable specification passes through the routine build job on the supported build host, preserving its summary, detail and typed MCP source-fact assertions.

- Starts from: `spec/entity/clingen-erepo.md` and its frozen runner. Job `1792238883286-80627` on the reviewed autocomplete repair passed lint and the routine tests, then failed both ERepo blocks at lines 24 and 28 with runner `clingen-erepo-frozen` exiting 1. The autocomplete repair does not modify this page or runner.
- Keeps: frozen provider facts, explicit failure propagation, typed MCP behavior, and all other routine specifications. Do not weaken assertions to manufacture a pass.
- Changes: claims `spec/fixtures/run-clingen-erepo-fixture.sh` `CHANGELOG.md`. The runner contains three GNU-only `head -c -1` calls; inventory and replace that class with portable behavior while preserving every emitted JSON array. The executable spec's assertions stay unchanged.
- Proof: reproduce the two existing executable-spec failures on current main through the build job before editing; record the exact command and output. Repair their identified cause, run the affected spec page, then the routine merged-candidate checks. Existing failed executable contracts provide the regression coverage; add a focused contract only if needed to preserve the fix.
- Defers: historical failure analysis and broad fixture restructuring. This ticket only clears the concrete main check failure.

PM decision: 2026-10-17. Ian approved main repairs before trigger shutdown. This is a reviewed recovery lane while ordinary landings are held. No new clinical or biological interpretation is authorized; if the failure requires a source-fact decision, report it for clinical review.

## Reproduction

On current main `67c42424e`, job `1792239239603-16935` ran the following and exited 1 after successful artifact preparation:

```sh
pm job run 2157 67c42424ea7df9e42e3631e212be8e4eefe437b1 -- bash -c 'make prepare-spec && source .cache/spec-artifacts.env && export BIOMCP_BIN BIOMCP_TEST_RETRY_SLEEP_OFF=1 && "$HOME/.local/bin/mustmatch" test spec/entity/clingen-erepo.md --lang bash --timeout 180'
```

Both existing ERepo blocks failed because `clingen-erepo-frozen` exited 1. Diagnostic job `1792239294142-21205` ran the prepared runner with `bash -x` and confirmed `head: illegal byte count -- -1` while reading `missing_gene.stdout`, before either source-fact assertion. Replace the three negative-byte-count calls with portable byte handling; retain the exact JSON and trailing-newline checks.

---
priority: 1
---

# 2188 — Keep changelog contract history local during offline tests

Status: OPEN.

Milestone: 0.9.3

Reviews: revision 6b818bb, accept

## Outcome

The changelog contract proves ticket ownership across development-cycle sections using locally available controlled history, including when the source checkout is partial and tests run offline. Preserve the real changelog behavior assertions.

- Evidence: test_dev_cycle_section_carries_its_own_tickets failed in second rehearsal39078684323 at7e4ede39a6fd54815a9a6dd0db67bcd8a554db86. The offline job tried to fetch missing promisor Git object8003da3ef6f9dee29c38ba780fe91c0b6c7f2267 from the remote and could not resolve the host. Python reported1259passed,8skipped,3failed total; the other two failures belong to2187.
- Changes: claims `tests/test_docs_changelog_refresh.py` `CHANGELOG.md`. Start from the existing contract and fixture ownership. Do not permit network access, skip the failing case or weaken ticket-section assertions. Report any additional claimed file before editing; keep this separate from the measurement fixture lane.
- Proof: retain the actual Linux rehearsal RED, reproduce the named isolation boundary on current main through a build job, correct it with the smallest existing outside-in contract, and run affected checks. Fresh review, a combined landing gate and actual-main full check precede a new complete nonpublishing rehearsal.
- Keeps: offline isolation, dependency pinning, existing accepted product fixes and release-only triggers. No broad fixture framework or unrelated refactor.

## Reproduction

- Command: `make test-python` in qualification/python-lane, job117531654838 of https://github.com/genomoncology/biomcp/actions/runs/39078684323 .
- The coordinator retains its complete raw log in temporary scratch. Main remains7e4ede39a6fd54815a9a6dd0db67bcd8a554db86; ordinary landings stay held except these reviewed repairs.

## Progress

- 2026-10-17 started

## Repair and checks

Restated outcome: retain the real changelog and current-ticket ownership checks while supplying the historical ownership input locally, so the test does not depend on a partial checkout fetching old blobs. Only the existing contract and changelog changed. The local fixture records the historical milestone of batch 2079 from commit a1d49933525bada2ff1bb1c0de4e760e035256f2; it changes no production exception or gate behavior.

- RED job1792280095003-98320 at main7e4ede39a6fd54815a9a6dd0db67bcd8a554db86 ran the named existing contract in a temporary partial checkout with historical blob8003da3ef6f9dee29c38ba780fe91c0b6c7f2267 absent. GIT_NO_LAZY_FETCH=1 and protocol.allow=never prohibited Git transport. The historical2079lookup failed, matching the Linux boundary.
- GREEN job1792280173274-8171 at branch base863bb1e03e822c476a87a0d8b65a7aabc368ca87 plus final inline patch retained that missing-history and no-transport boundary. `python -m pytest tests/test_docs_changelog_refresh.py tests/test_changelog_coverage_check.py -q -n 0`:95passed, including existing wrong-milestone and unlanded-future rejection cases. Ruff0.13.2 and `git diff --check` passed.
- Tested patch SHA2561d9fba70ecfe4f085001a686ac15c93f2cbbbf8580ab24b905b4860c6efbc00a equals the committed diff863bb1e03..bb3e691590184e8b0b33900043781949ab2bba94; coordinator checked the equality and job result.
- One intermediate GREEN setup used main before this ticket existed and correctly failed its real ownership assertion. The same patch passed from the ticket branch base without a bypass or code change.

Root cause: the contract read ambient historical repository objects, which a partial clone may not hold offline. The local historical fixture is an input, not an allowed-ticket bypass. Another batch moving milestones requires its historical ownership input. The local operating system does not provide network namespace isolation; the reproduction specifically prohibited Git transport. A focused offline Linux check is required before the next full rehearsal. Actual product source, measurement harness and recorded measurements are unchanged.

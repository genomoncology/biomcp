---
priority: 1
---

# 2188 — Keep changelog contract history local during offline tests

Status: OPEN.

Milestone: 0.9.3

## Outcome

The changelog contract proves ticket ownership across development-cycle sections using locally available controlled history, including when the source checkout is partial and tests run offline. Preserve the real changelog behavior assertions.

- Evidence: test_dev_cycle_section_carries_its_own_tickets failed in second rehearsal39078684323 at7e4ede39a6fd54815a9a6dd0db67bcd8a554db86. The offline job tried to fetch missing promisor Git object8003da3ef6f9dee29c38ba780fe91c0b6c7f2267 from the remote and could not resolve the host. Python reported1259passed,8skipped,3failed total; the other two failures belong to2187.
- Changes: claims `tests/test_docs_changelog_refresh.py` `CHANGELOG.md`. Start from the existing contract and fixture ownership. Do not permit network access, skip the failing case or weaken ticket-section assertions. Report any additional claimed file before editing; keep this separate from the measurement fixture lane.
- Proof: retain the actual Linux rehearsal RED, reproduce the named isolation boundary on current main through a build job, correct it with the smallest existing outside-in contract, and run affected checks. Fresh review, a combined landing gate and actual-main full check precede a new complete nonpublishing rehearsal.
- Keeps: offline isolation, dependency pinning, existing accepted product fixes and release-only triggers. No broad fixture framework or unrelated refactor.

## Reproduction

- Command: `make test-python` in qualification/python-lane, job117531654838 of https://github.com/genomoncology/biomcp/actions/runs/39078684323 .
- The coordinator retains its complete raw log in temporary scratch. Main remains7e4ede39a6fd54815a9a6dd0db67bcd8a554db86; ordinary landings stay held except these reviewed repairs.

---
flow: build
priority: 2
deps: [2002]
---

# 2003: Search patients by typed facts

## Outcome

An agent finds patients on the configured FHIR server by gender, birth date range, and coded condition, and can ask for the server-reported count. Decision record: `sdlc/planning/adr/0002-read-one-patient-fhir-record-as-the-patient-entity.md`.

```
biomcp search patient --gender female --born-after 1950-01-01 --condition "http://snomed.info/sct|44054006" --limit 10
biomcp search patient --condition "http://snomed.info/sct|44054006" --count
```

## Current Facts

- Ticket 2002 supplies the FHIR request function, `BIOMCP_FHIR_BASE`, value encoding, and the serve-http refusal. This ticket adds no transport code.
- Other `search` commands take `-l/--limit` with a default of 10 (`src/cli/disease/mod.rs:29`).
- FHIR date search uses prefixes. `gt` means strictly after and `lt` means strictly before.
- `_has` reverse chaining is optional in FHIR R4. A server declares it in its CapabilityStatement.

## Scope

- `search patient` takes `--gender`, `--born-after`, `--born-before`, `--condition <system|code>`, `--limit`, and `--count`. `--limit` runs 1 to 50 with a default of 10, as other search commands do. It rejects free text and requires at least one filter.
- `--born-after D` sends `birthdate=gtD`. `--born-before D` sends `birthdate=ltD`. Both bounds exclude the given date, as the flag names say.
- Before a condition search, read `metadata`. `_has` counts as declared only when the `rest` entry with mode `server` has a `resource` of type `Patient` whose `searchParam` list holds an entry named `_has`. Otherwise refuse the filter with a clear message and send no search.
- List query: `Patient?gender=..&birthdate=gt..&birthdate=lt..&_has:Condition:patient:code=..&_elements=id,gender,birthDate&_count=<limit>`. It reads one page only.
- `--count` sends the same filters with `_summary=count` and prints `Bundle.total` labeled server-reported. When the server sends no `total`, the output says the server reported no count. It never counts returned entries. `--limit` does not apply with `--count`.
- Search results list id, gender, and birth date, each with a `get patient <id>` next command.
- `docs/user-guide/patient.md` and `CHANGELOG.md` cover search.

## Exclusions

No free text, no name or identifier search, no paging past the first page, no other `_has` targets, and no cohort entity.

## Acceptance

Synthetic bundles only, served by the existing spec fixture runner:

1. Each filter combination maps to the exact query string above. A test pins the strings, including encoded `|` in the condition value.
2. A capability statement without `_has` makes `--condition` fail with the refusal message. The fixture request log shows no Patient search.
3. `--count` prints the `Bundle.total` value labeled server-reported. A bundle with no `total` prints the no-count message.
4. `--limit 3` sends `_count=3`. `--limit 0` and `--limit 51` fail before any request. A search with no filter fails before any request.
5. Over `serve-http`, typed `search patient` is refused by the 2002 check.
6. `spec/entity/patient.md` gains search cases.

`make lint`, `make test`, and `make spec` pass on the gate host at the pushed SHA. The manual smoke run against the live HAPI server confirms that its CapabilityStatement declares `_has` in the same place the refusal test checks and that a condition search returns results.

## Dependencies

2002.

## Complexity

- Contract score: 2 (new search grammar, date semantics, count contract)
- State and timing score: 0 (one capability read and one page)
- Reach score: 1 (entity, CLI, docs, spec)
- Proof score: 1 (pinned query strings and fixtures)
- Cost of error score: 1 (a wrong count or date bound misstates a cohort)
- Total: 5
- Minimum level floor: none
- Final level: 2
- Reasons: new query grammar and count labeling on top of the 2002 transport
- Selected model: claude-opus

## Review

- Design review: split from 2002 (2026-09-23). Added the no-total count case, `--limit`, strict `gt` and `lt` date bounds, and the live `_has` check in the smoke run.
- Design re-review: accepted (2026-09-23). Stated the `--limit` range and where `_has` must be declared.
- Code review: pending

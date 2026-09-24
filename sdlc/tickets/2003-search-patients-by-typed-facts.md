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

- Ticket 2002 supplies `BIOMCP_FHIR_BASE`, URL encoding, the 2002 patient ID rule, and the serve-http refusal. Its one private request function in `src/sources/fhir.rs` sets every header.
- Other `search` commands take `-l/--limit` with a default of 10 (`src/cli/disease/mod.rs:29`).
- FHIR date search uses prefixes. `gt` means strictly after and `lt` means strictly before. In a FHIR search value, a comma means OR.
- A FHIR server may ignore a search parameter it does not support, and may ignore `_elements`. Under `Prefer: handling=strict` it must fail instead of ignoring a parameter. The CapabilityStatement at `metadata` lists the search parameters the server supports.
- MCP typed search for `patient` takes no filters today (`src/mcp/shell.rs:237`).

## Scope

- `search patient` takes `--gender`, `--born-after`, `--born-before`, `--condition <system|code>`, `--limit`, and `--count`. It rejects free text and requires at least one filter. `--limit` runs 1 to 50 with a default of 10.
- Values are checked before any request. `--gender` is one of `male`, `female`, `other`, or `unknown`. Each date is a FHIR date: `YYYY`, `YYYY-MM`, or `YYYY-MM-DD`, with no prefix of its own. The check validates a real calendar date, so `1950-13-01` and `1950-02-30` fail. `--condition` holds one `|` with a non-empty system on the left and a non-empty code on the right. A comma in any value fails.
- `--born-after D` sends `birthdate=gtD`. `--born-before D` sends `birthdate=ltD`. The command sends no other prefix.
- Transport: two new client methods. One reads `metadata`. One runs a single-page Patient search. Both build on the 2002 private request function. The search adds a `Prefer: handling=strict` header. The `--count` search sends it too. Neither uses the multi-page walk.
- Before any search, read `metadata`. Find the `rest` entry with mode `server` and its `resource` of type `Patient`. Every search parameter the command will send (`gender`, `birthdate`, `_has`) must appear by name in that resource's `searchParam` list. If any is missing, refuse, name the missing parameter, and send no search. The result controls `_count`, `_summary`, and `_elements` are not search parameters and are not checked. The output rules below cover a server that ignores them.
- List query: `Patient?gender=..&birthdate=gt..&birthdate=lt..&_has:Condition:patient:code=..&_elements=id,gender,birthDate&_count=<limit>`. It reads one page.
- Output keeps only `id`, `gender`, and `birthDate`, in text and in JSON. It drops every other field the server returns. It skips any entry that is not a Patient and prints at most `<limit>` entries. It prints `get patient <id>` only for an id that passes the 2002 ID rule. An entry with a bad id prints no id and no next command.
- `--count` sends the same filters with `_summary=count` and prints `Bundle.total` labeled server-reported. With no `total`, it says the server reported no count. It never counts entries. `--limit` does not apply with `--count`.
- MCP typed search for `patient` stays filterless. A filterless call reaches the CLI and fails the no-filter check with no request. MCP stdio callers search with filters through the shell tool, which runs the same CLI checks. Reason: typed filters would copy the value rules into a JSON schema and a second test surface. Ian can overturn this in a follow-up ticket.
- `docs/user-guide/patient.md` and `CHANGELOG.md` cover search and name the shell tool as the MCP route.

## Exclusions

No free text, no name or identifier search, no paging past the first page, no other `_has` targets, no typed MCP filters, and no cohort entity.

## Acceptance

Synthetic bundles only, served by the existing spec fixture runner. "Sends no request" means the fixture request log is empty.

1. Each filter combination maps to the exact query string above. A test pins the strings, including encoded `|`, and checks `Prefer: handling=strict` on the list search and on the `--count` search.
2. A capability statement without `_has` refuses `--condition`. One without `birthdate` refuses `--born-after`. One without `gender` refuses `--gender`. Each run logs the `metadata` read and no Patient search.
3. Each bad value fails and sends no request: `--gender F`, `--gender male,female`, `--born-after 1950-13-01`, `--born-after ge1950`, `--born-before 1950,1960`, `--condition 44054006`, `--condition "|44054006"`, `--condition "http://snomed.info/sct|"`, and `--condition "http://snomed.info/sct|1,2"`.
4. `--count` prints `Bundle.total` labeled server-reported. A bundle with no `total` prints the no-count message.
5. `--limit 3` sends `_count=3`. `--limit 0` and `--limit 51` fail and send no request. A search with no filter fails and sends no request.
6. A fixture bundle ignores `_elements`. It holds full Patients with names, addresses, and telecoms, one Condition entry, one Patient with id `a/b`, and more entries than `--limit`. Text and JSON output hold none of the names, addresses, or telecoms. They hold no Condition, no `get patient a/b`, and no more than `--limit` entries.
7. Over `serve-http`, the shell tool running `search patient --gender female --condition "http://snomed.info/sct|44054006" --count` is refused by the 2002 check and sends no request. Over stdio, typed search with `gender` fails as an unknown field and sends no request.
8. `spec/entity/patient.md` gains search cases.

`make lint`, `make test`, and `make spec` pass on the gate host at the pushed SHA. The manual smoke run against the live HAPI server confirms that its CapabilityStatement declares `gender`, `birthdate`, and `_has` where the refusal test checks, that it accepts `Prefer: handling=strict`, and that a `--condition` search under strict handling returns results. If the server does not declare one of the three, the ticket stops before landing and reports it. The check is not loosened to fit the server.

## Dependencies

2002.

## Complexity

- Contract score: 2 (new search grammar, value rules, date semantics, count contract)
- State and timing score: 0 (one capability read and one page)
- Reach score: 1 (entity, CLI, request header, docs, spec)
- Proof score: 2 (capability matrix, bad-value cases, and a leaky fixture bundle, each with an empty request log)
- Cost of error score: 2 (an ignored filter misstates a cohort count; an ignored `_elements` prints names and addresses)
- Total: 7
- Minimum level floor: none
- Final level: 3
- Reasons: the server can ignore filters and field limits, so correctness rests on the capability check, strict handling, value checks, and local output trimming
- Selected model: claude-opus

## Review

- Design review: split from 2002 (2026-09-23). Added the no-total count case, `--limit`, strict `gt` and `lt` date bounds, and the live `_has` check in the smoke run.
- Design re-review: accepted (2026-09-23). Stated the `--limit` range and where `_has` must be declared.
- Design review after 2002 landed: rejected (2026-09-23). Revised to check every search parameter against `metadata`, send `Prefer: handling=strict`, check value forms and commas, keep only id, gender, and birth date in output, state the transport work, keep typed MCP search filterless, and run the serve-http case with filters.
- Design review after revision: accepted with four low findings (2026-09-23), folded in above: strict handling on `--count` with its own test, a strict `--condition` case in the smoke run, a stop before landing if the smoke server lacks a parameter, and real calendar dates.
- Code review: pending

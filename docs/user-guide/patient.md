# Patient

Use patient commands to read one patient's record from your own FHIR R4 server, demographics and the Condition list, and to find patients by typed facts.

Key boundaries:

- The server is the one named in `BIOMCP_FHIR_BASE`. No command, section, or MCP argument takes a URL.
- Patient commands run on the CLI and stdio MCP only. `serve-http` refuses them until the HTTP transport has authenticated per-user sessions.
- Every FHIR request is sent with no-store. BioMCP caches no patient data.
- Errors name no URL and no patient ID.

## Configure the FHIR server

```bash
export BIOMCP_FHIR_BASE=https://fhir.example.org/fhir
```

The value must be an `http` or `https` URL with no credentials, query, or fragment. With the variable unset, patient commands fail and name it, and `biomcp health` reports the FHIR row as not configured. Health sends no request to the FHIR server.

## Get a patient record

```bash
biomcp get patient <id>
```

The card reads one Patient resource and shows its ID, gender, and birth date.

A patient ID is 1-64 letters, digits, hyphens, or periods, the FHIR `id` rule, and not only periods. Any other ID is refused before a request is sent.

## Search patients

```bash
biomcp search patient --gender female --born-after 1950-01-01 --condition "http://snomed.info/sct|44054006" --limit 10
biomcp search patient --condition "http://snomed.info/sct|44054006" --count
```

Search takes typed facts only, and at least one of them:

- `--gender`: `male`, `female`, `other`, or `unknown`
- `--born-after` and `--born-before`: a FHIR date, `YYYY`, `YYYY-MM`, or `YYYY-MM-DD`, with no prefix. A full date must be a real calendar date, so `1950-13-01` fails. Both bounds are strict. `--born-after D` sends `birthdate=gtD`.
- `--condition`: one `system|code` pair, such as `http://snomed.info/sct|44054006`. It finds patients with a Condition carrying that code.

A comma in any value is refused. In a FHIR search a comma means OR. Every value is checked before any request.

Before it searches, BioMCP reads the server's `metadata`. Each search parameter the command would send (`gender`, `birthdate`, `_has`) must appear under the server's Patient resource. If one is missing, the command names it and sends no search. The search itself sends `Prefer: handling=strict`, so a server that would ignore a parameter must fail instead.

The search reads one page of at most `--limit` patients, 1 to 50, default 10. Output keeps only each patient's id, gender, and birth date, whatever else the server returns. It skips any entry that is not a Patient. A patient whose id breaks the FHIR id rule shows no id and no `get patient` command.

`--count` prints the server's `Bundle.total`, labeled server-reported. When the server reports no total, the command says so. It never counts entries itself. `--limit` does not apply with `--count`.

## Request patient sections

```bash
biomcp get patient <id> conditions
biomcp get patient <id> all
```

`conditions` searches `Condition?patient=<id>` at 100 per page and follows the server's `next` links. Each row shows the condition text, its codes, the clinical and verification status, the onset, and the recorded date.

The walk follows a `next` link or a redirect only when it stays on the configured origin and base path, and it reads at most 20 pages. The section settles as `degraded` and keeps the rows it read when any of these happen:

- a `next` link repeats
- a `next` link or a redirect leaves the configured server
- the walk reaches the 20-page cap
- a later page fails
- the server reports an error in an OperationOutcome
- a condition has no clinical status

A failed first page makes the section `unavailable`. A search with no matches is `empty`.

`all` currently means `conditions`.

## Helper commands

Each search row with a valid id suggests `biomcp get patient <id>`.

## JSON mode

```bash
biomcp get patient <id> conditions --json
```

JSON carries `section_outcomes.conditions` with its `outcome` and, when degraded or unavailable, a `message`. `_meta.section_sources` names FHIR for the identity and the conditions.

`search patient --json` returns `results` with `id`, `gender`, and `birth_date`, and `_meta.next_commands`. `search patient --count --json` returns `server_reported_total`, which is `null` with a `message` when the server reports none.

## Practical tips

The `degraded` message tells you why the list may be incomplete. It names no server. You can show it to the clinician as written.

Over MCP, run BioMCP on stdio for patient work. The typed `get` tool accepts `{"entity": "patient", "id": "<id>", "sections": ["conditions"]}`. The typed `search` tool takes no patient filters. Search with filters through the `biomcp` shell tool, for example `biomcp search patient --gender female --count`. It runs the same checks as the CLI.

## Related guides

- [CLI Reference](cli-reference.md)
- [Disease](disease.md)

# Patient

Use patient commands to read one patient's record from your own FHIR R4 server: demographics and the Condition list.

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

## Search patients

```bash
biomcp search patient --gender female --born-after 1950-01-01 --condition "http://snomed.info/sct|44054006" --limit 10
biomcp search patient --condition "http://snomed.info/sct|44054006" --count
```

`search patient` finds people by typed facts, never free text: `--gender`
(male, female, other, unknown), `--born-after` and `--born-before` (FHIR dates
`YYYY`, `YYYY-MM`, or `YYYY-MM-DD`, exclusive bounds), and `--condition` as
`system|code`. At least one filter is required. `--limit` runs 1-50 with a
default of 10. No comma is allowed in any value.

Every value is checked before any request. The command then reads the server's
`metadata` and refuses a filter whose search parameter (`gender`, `birthdate`,
or `_has`) the server does not declare for Patient, naming the parameter. The
search itself is sent with `Prefer: handling=strict`, so a server that would
silently ignore a parameter fails instead. One page is read; there is no
paging.

Output keeps only the id, gender, and birth date of each Patient entry. A
Patient whose id breaks the FHIR id rule still lists its row but prints no id
and no `get patient` command.

`--count` sends the same filters with `_summary=count` and prints `Bundle.total`
labeled as server-reported. When the server sends no `total`, the command says
the server reported no count; it never counts entries itself. `--limit` does
not apply with `--count`.

Over MCP, run searches through the shell tool, for example
`biomcp search patient --gender female --count`; the typed `search` tool for
patient takes no filters on purpose, so the value rules live in one place.

## Get a patient record

```bash
biomcp get patient <id>
```

The card reads one Patient resource and shows its ID, gender, and birth date.

A patient ID is 1-64 letters, digits, hyphens, or periods, the FHIR `id` rule, and not only periods. Any other ID is refused before a request is sent.

## Request patient sections

```bash
biomcp get patient <id> conditions
biomcp get patient <id> all
```

`conditions` searches `Condition?patient=<id>` at 100 per page and follows the server's `next` links. Each row shows the condition text, its codes, the clinical and verification status, the onset, and the recorded date. A row whose source omits the clinical status shows it as absent.

The walk follows a `next` link or a redirect only when it stays on the configured origin and base path, and it reads at most 20 pages. The section settles as `degraded` and keeps the rows it read when any of these happen:

- a `next` link repeats
- a `next` link or a redirect leaves the configured server
- the walk reaches the 20-page cap
- a later page fails
- the server reports an error in an OperationOutcome
- a `problem-list-item` condition has no clinical status and its verification
  status is not `entered-in-error` (FHIR R4 invariant con-3)

A failed first page makes the section `unavailable`. A search with no matches is `empty`.

`all` currently means `conditions`.

## Helper commands

`search patient` covers finding patients by typed facts; `get patient <id>`
reads one known patient's record. There is no other helper family for
patient.

## JSON mode

```bash
biomcp get patient <id> conditions --json
```

JSON carries `section_outcomes.conditions` with its `outcome` and, when degraded or unavailable, a `message`. `_meta.section_sources` names FHIR for the identity and the conditions.

## Practical tips

The `degraded` message tells you why the list may be incomplete. It names no server. You can show it to the clinician as written.

Over MCP, run BioMCP on stdio for patient work. The typed `get` tool accepts `{"entity": "patient", "id": "<id>", "sections": ["conditions"]}`.

## Related guides

- [CLI Reference](cli-reference.md)
- [Disease](disease.md)

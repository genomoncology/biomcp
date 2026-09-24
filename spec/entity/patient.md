# Patient Record

A clinician's agent reads one patient from the operator's FHIR server. These
routine contracts run against synthetic Patient and Condition resources served
by the provider contract fixture. The agent never passes a URL. The server
comes from `BIOMCP_FHIR_BASE` alone, and each block points it at the fixture.

## The Card Shows Demographics

The base card reads one Patient resource and no conditions.

```bash
export BIOMCP_FHIR_BASE="${BIOMCP_PROVIDER_CONTRACT_BASE:?provider fixture base is not configured}/fhir"
../../tools/biomcp-ci get patient SYNTH-PT-1 | mustmatch like '# Patient SYNTH-PT-1

Source: FHIR
Gender: female
Birth date: 1970-01-01'
../../tools/biomcp-ci --json get patient SYNTH-PT-1 | jq -c '[.id, .gender, has("conditions")]' | mustmatch '["SYNTH-PT-1","female",false]'
```

## Conditions Follow Every Page

The fixture splits the condition list across two pages. Both rows appear, and
the section settles as `data`.

```bash
export BIOMCP_FHIR_BASE="${BIOMCP_PROVIDER_CONTRACT_BASE:?provider fixture base is not configured}/fhir"
../../tools/biomcp-ci get patient SYNTH-PT-1 conditions | mustmatch like '| Synthetic condition one | - | active | - | 2020-01-01 | - |
| Synthetic condition two | - | resolved | - | 2020-01-01 | - |'
../../tools/biomcp-ci --json get patient SYNTH-PT-1 all | jq -c '[(.conditions | length), .section_outcomes.conditions.outcome]' | mustmatch '[2,"data"]'
```

## No Conditions Is Empty

```bash
export BIOMCP_FHIR_BASE="${BIOMCP_PROVIDER_CONTRACT_BASE:?provider fixture base is not configured}/fhir"
../../tools/biomcp-ci get patient SYNTH-PT-EMPTY conditions | mustmatch like 'No conditions found on the FHIR server.'
../../tools/biomcp-ci --json get patient SYNTH-PT-EMPTY conditions | jq -r '.section_outcomes.conditions.outcome' | mustmatch 'empty'
```

## An Off-Server Next Link Is Degraded

The fixture's second page points at another host. BioMCP does not follow it.
The section keeps the first page and says it may be incomplete, and the
message names no URL.

```bash
export BIOMCP_FHIR_BASE="${BIOMCP_PROVIDER_CONTRACT_BASE:?provider fixture base is not configured}/fhir"
../../tools/biomcp-ci --json get patient SYNTH-PT-OFFSITE conditions | jq -c '[(.conditions | length), .section_outcomes.conditions.outcome, .section_outcomes.conditions.message]' | mustmatch '[1,"degraded","The condition list stopped at a link or redirect off the configured server and may be incomplete."]'
```

## Errors Name No Patient Or Server

A missing patient fails without naming the ID. A malformed ID is refused before
any request.

```bash
export BIOMCP_FHIR_BASE="${BIOMCP_PROVIDER_CONTRACT_BASE:?provider fixture base is not configured}/fhir"
../../tools/biomcp-ci get patient SYNTH-PT-MISSING >/dev/null 2>&1 && exit 1
(../../tools/biomcp-ci get patient SYNTH-PT-MISSING 2>&1 || true) | mustmatch 'Error: the FHIR server has no record with that patient ID.'
../../tools/biomcp-ci get patient 'a/b' >/dev/null 2>&1 && exit 1
(../../tools/biomcp-ci get patient 'a/b' 2>&1 || true) | mustmatch like 'a patient ID must be 1-64 characters of letters, digits, hyphens, or periods'
```

## An Unset Base Names The Variable

```bash
env -u BIOMCP_FHIR_BASE ../../tools/biomcp-ci get patient SYNTH-PT-1 >/dev/null 2>&1 && exit 1
(env -u BIOMCP_FHIR_BASE ../../tools/biomcp-ci get patient SYNTH-PT-1 2>&1 || true) | mustmatch like 'set BIOMCP_FHIR_BASE to its base URL'
```

## Patient Search Keeps Only Typed Facts

The fixture's search page ignores `_elements` and `_count`. It sends full
Patients with names, addresses, and telecoms, a Condition, a Patient whose id
breaks the FHIR id rule, and more entries than `--limit`. The output keeps
only id, gender, and birth date for at most `--limit` Patients. Each block
points the base at the page's own request log when the runner provides one.

```bash
log="${BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG:?provider request log is not configured}"
base="${BIOMCP_PROVIDER_CONTRACT_BASE:?provider fixture base is not configured}"
case "${log##*/}" in request-log.*) base="$base/__biomcp_provider_worker/${log##*/}" ;; esac
export BIOMCP_FHIR_BASE="$base/fhir"
../../tools/biomcp-ci search patient --gender female --limit 3 | mustmatch like '| - | female | 1970-01-01 |
| SYNTH-PT-S1 | female | 1970-01-01 |
| SYNTH-PT-S2 | female | 1970-01-01 |'
../../tools/biomcp-ci search patient --gender female --limit 3 | mustmatch not like 'leak'
../../tools/biomcp-ci search patient --gender female --limit 3 | mustmatch not like 'SYNTH-PT-S3'
../../tools/biomcp-ci search patient --gender female --limit 3 | mustmatch not like 'get patient a/b'
../../tools/biomcp-ci --json search patient --gender female --limit 3 | jq -c '[.results[].id, (.results[0] | keys), ._meta.next_commands[0]]' | mustmatch '[null,"SYNTH-PT-S1","SYNTH-PT-S2",["birth_date","gender","id"],"biomcp get patient SYNTH-PT-S1"]'
../../tools/biomcp-ci --json search patient --gender female --limit 3 | mustmatch not like 'leak'
```

## Patient Search Sends The Exact Query

Metadata is read first. The search then carries every filter, the encoded
`|`, `_elements`, and `_count`.

```bash
log="${BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG:?provider request log is not configured}"
base="${BIOMCP_PROVIDER_CONTRACT_BASE:?provider fixture base is not configured}"
case "${log##*/}" in request-log.*) base="$base/__biomcp_provider_worker/${log##*/}" ;; esac
export BIOMCP_FHIR_BASE="$base/fhir"
../../tools/biomcp-ci search patient --gender female --born-after 1950-01-01 --born-before 1960 --condition "http://snomed.info/sct|44054006" --limit 3 >/dev/null
grep -F 'GET /fhir/' "$log" | tail -n 2 | mustmatch 'GET /fhir/metadata
GET /fhir/Patient?gender=female&birthdate=gt1950-01-01&birthdate=lt1960&_has%3ACondition%3Apatient%3Acode=http%3A%2F%2Fsnomed.info%2Fsct%7C44054006&_elements=id%2Cgender%2CbirthDate&_count=3'
../../tools/biomcp-ci search patient --condition "http://snomed.info/sct|44054006" --count >/dev/null
grep -F 'GET /fhir/' "$log" | tail -n 1 | mustmatch 'GET /fhir/Patient?_has%3ACondition%3Apatient%3Acode=http%3A%2F%2Fsnomed.info%2Fsct%7C44054006&_summary=count'
```

## Patient Count Is Server-Reported

```bash
log="${BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG:?provider request log is not configured}"
base="${BIOMCP_PROVIDER_CONTRACT_BASE:?provider fixture base is not configured}"
case "${log##*/}" in request-log.*) base="$base/__biomcp_provider_worker/${log##*/}" ;; esac
export BIOMCP_FHIR_BASE="$base/fhir"
../../tools/biomcp-ci search patient --condition "http://snomed.info/sct|44054006" --count | mustmatch like 'Server-reported total: 7'
../../tools/biomcp-ci --json search patient --condition "http://snomed.info/sct|44054006" --count | jq -c '[.source, .server_reported_total]' | mustmatch '["FHIR",7]'
../../tools/biomcp-ci search patient --condition "http://example.org/synthetic|no-total" --count | mustmatch like 'The FHIR server reported no count.'
../../tools/biomcp-ci --json search patient --condition "http://example.org/synthetic|no-total" --count | jq -c '.server_reported_total' | mustmatch 'null'
```

## Patient Search Refuses What It Cannot Send Safely

A bad value, a missing filter, or an out-of-range limit fails before any
request. A server whose metadata does not list `_has` gets the metadata read
and no search.

```bash
log="${BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG:?provider request log is not configured}"
base="${BIOMCP_PROVIDER_CONTRACT_BASE:?provider fixture base is not configured}"
case "${log##*/}" in request-log.*) base="$base/__biomcp_provider_worker/${log##*/}" ;; esac
export BIOMCP_FHIR_BASE="$base/fhir"
before="$(grep -c -F 'GET /fhir' "$log" || true)"
../../tools/biomcp-ci search patient --gender F >/dev/null 2>&1 && exit 1
../../tools/biomcp-ci search patient --born-after 1950-13-01 >/dev/null 2>&1 && exit 1
../../tools/biomcp-ci search patient --condition "http://snomed.info/sct|1,2" >/dev/null 2>&1 && exit 1
../../tools/biomcp-ci search patient --gender female --limit 51 >/dev/null 2>&1 && exit 1
(../../tools/biomcp-ci search patient 2>&1 || true) | mustmatch like 'search patient needs at least one of --gender, --born-after, --born-before, or --condition'
(../../tools/biomcp-ci search patient --born-before 1950,1960 2>&1 || true) | mustmatch like 'In a FHIR search a comma means OR.'
grep -c -F 'GET /fhir' "$log" | mustmatch "$before"
export BIOMCP_FHIR_BASE="$base/fhir-lacks-has"
(../../tools/biomcp-ci search patient --condition "http://snomed.info/sct|44054006" 2>&1 || true) | mustmatch like 'does not list the Patient search parameter _has, so no search was sent'
grep -F 'GET /fhir-lacks-has/' "$log" | mustmatch 'GET /fhir-lacks-has/metadata'
```

## Discovery Names The Command And Its Limits

```bash
../../tools/biomcp-ci list patient | mustmatch like '- `get patient <id> conditions` - this patient'"'"'s FHIR Condition records'
../../tools/biomcp-ci list patient | mustmatch like '`serve-http` refuses them'
../../tools/biomcp-ci get patient --help | mustmatch like 'Reads one patient from the FHIR server named in BIOMCP_FHIR_BASE.'
../../tools/biomcp-ci list patient | mustmatch like '- `search patient --condition <system|code> --count` - the patient count the server reports'
../../tools/biomcp-ci search patient --help | mustmatch like 'Finds patients on the FHIR server named in BIOMCP_FHIR_BASE by gender, birth'
```

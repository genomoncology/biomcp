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

## Conditions Degrade Only On A Con-3 Break

FHIR R4 invariant con-3 requires a clinical status only on a `problem-list-item`
Condition that is not `entered-in-error`. Encounter-diagnosis conditions
satisfy it without one, the section settles `data`, and each row shows the
clinical status as absent when the source omits it.

```bash
export BIOMCP_FHIR_BASE="${BIOMCP_PROVIDER_CONTRACT_BASE:?provider fixture base is not configured}/fhir"
../../tools/biomcp-ci get patient SYNTH-PT-ENCOUNTER conditions | mustmatch like '| Synthetic encounter one | - | - | - | 2020-01-01 | - |
| Synthetic encounter two | - | resolved | - | 2020-01-01 | - |'
../../tools/biomcp-ci --json get patient SYNTH-PT-ENCOUNTER conditions | jq -c '[.section_outcomes.conditions.outcome, [.conditions[].clinical_status]]' | mustmatch '["data",[null,"resolved"]]'
../../tools/biomcp-ci get patient SYNTH-PT-PROBLEM conditions | mustmatch like '| Synthetic problem one | - | - | - | 2020-01-01 | - |'
../../tools/biomcp-ci --json get patient SYNTH-PT-PROBLEM conditions | jq -c '[.section_outcomes.conditions.outcome, .section_outcomes.conditions.message]' | mustmatch '["degraded","At least one problem-list-item condition has no clinical status."]'
../../tools/biomcp-ci --json get patient SYNTH-PT-EXEMPT conditions | jq -c '[.section_outcomes.conditions.outcome, [.conditions[] | [.clinical_status, .verification_status]]]' | mustmatch '["data",[[null,"entered-in-error"]]]'
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

## Search Matches Typed Facts

The fixture ignores `_elements` and returns full patients, one with an id
outside the FHIR id rule, and one Condition entry. The output keeps only id,
gender, and birth date, drops the Condition, prints a next command only for a
usable id, and stops at `--limit`.

```bash
export BIOMCP_FHIR_BASE="${BIOMCP_PROVIDER_CONTRACT_BASE:?provider fixture base is not configured}/fhir"
../../tools/biomcp-ci search patient --gender female --born-after 1950-01-01 --condition "http://snomed.info/sct|44054006" --limit 2 | mustmatch like '| SYNTH-PT-1 | female | 1970-01-01 | `biomcp get patient SYNTH-PT-1` |
| - | male | 1980-02-02 | - |'
../../tools/biomcp-ci search patient --gender female --limit 2 | mustmatch not like 'Synthia'
../../tools/biomcp-ci search patient --gender female --limit 2 | mustmatch not like 'Synthville'
../../tools/biomcp-ci search patient --gender female --limit 2 | mustmatch not like '555-0100'
../../tools/biomcp-ci search patient --gender female --limit 2 | mustmatch not like 'Synthetic search condition'
../../tools/biomcp-ci --json search patient --gender female --limit 2 | jq -c '[.count, .results[0].id, .results[1]]' | mustmatch '[2,"SYNTH-PT-1",{"gender":"male","birth_date":"1980-02-02"}]'
```

## The Count Is The Server's

`--count` prints `Bundle.total` labeled server-reported and never counts
entries.

```bash
export BIOMCP_FHIR_BASE="${BIOMCP_PROVIDER_CONTRACT_BASE:?provider fixture base is not configured}/fhir"
../../tools/biomcp-ci search patient --condition "http://snomed.info/sct|44054006" --count | mustmatch 'Server-reported patient count: 7'
../../tools/biomcp-ci --json search patient --condition "http://snomed.info/sct|44054006" --count | jq -c '.server_reported_count' | mustmatch '7'
```

## An Undeclared Search Parameter Refuses

The fixture serves three more bases, each with a CapabilityStatement missing
one parameter the command would send. Each refusal names the parameter. The
`metadata` read happens; no Patient search follows it.

```bash
export BIOMCP_FHIR_BASE="${BIOMCP_PROVIDER_CONTRACT_BASE:?provider fixture base is not configured}/fhir-nohas"
../../tools/biomcp-ci search patient --condition "http://snomed.info/sct|44054006" >/dev/null 2>&1 && exit 1
(../../tools/biomcp-ci search patient --condition "http://snomed.info/sct|44054006" 2>&1 || true) | mustmatch like 'does not support the `_has` search parameter on Patient'
export BIOMCP_FHIR_BASE="${BIOMCP_PROVIDER_CONTRACT_BASE:?provider fixture base is not configured}/fhir-nobirthdate"
../../tools/biomcp-ci search patient --born-after 1950-01-01 >/dev/null 2>&1 && exit 1
(../../tools/biomcp-ci search patient --born-after 1950-01-01 2>&1 || true) | mustmatch like 'does not support the `birthdate` search parameter on Patient'
export BIOMCP_FHIR_BASE="${BIOMCP_PROVIDER_CONTRACT_BASE:?provider fixture base is not configured}/fhir-nogender"
../../tools/biomcp-ci search patient --gender female >/dev/null 2>&1 && exit 1
(../../tools/biomcp-ci search patient --gender female 2>&1 || true) | mustmatch like 'does not support the `gender` search parameter on Patient'
```

## Filters Are Checked Before Any Request

One bad date and one filterless run fail locally; neither reaches the server.

```bash
export BIOMCP_FHIR_BASE="${BIOMCP_PROVIDER_CONTRACT_BASE:?provider fixture base is not configured}/fhir"
../../tools/biomcp-ci search patient --gender female --born-after 1950-13-01 >/dev/null 2>&1 && exit 1
(../../tools/biomcp-ci search patient --gender female --born-after 1950-13-01 2>&1 || true) | mustmatch like '--born-after takes a FHIR date: YYYY, YYYY-MM, or YYYY-MM-DD'
../../tools/biomcp-ci search patient >/dev/null 2>&1 && exit 1
(../../tools/biomcp-ci search patient 2>&1 || true) | mustmatch like 'needs at least one of --gender, --born-after, --born-before, or --condition'
```

## Discovery Names The Command And Its Limits

```bash
../../tools/biomcp-ci list patient | mustmatch like '- `get patient <id> conditions` - this patient'"'"'s FHIR Condition records'
../../tools/biomcp-ci list patient | mustmatch like '`serve-http` refuses them'
../../tools/biomcp-ci get patient --help | mustmatch like 'Reads one patient from the FHIR server named in BIOMCP_FHIR_BASE.'
```

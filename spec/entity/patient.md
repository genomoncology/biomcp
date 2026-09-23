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
out="$(../../tools/biomcp-ci --json get patient SYNTH-PT-OFFSITE conditions)"
echo "$out" | jq -c '[(.conditions | length), .section_outcomes.conditions.outcome]' | mustmatch '[1,"degraded"]'
echo "$out" | jq -r '.section_outcomes.conditions.message' | mustmatch 'The condition list stopped at a link or redirect off the configured server and may be incomplete.'
```

## Errors Name No Patient Or Server

A missing patient fails without naming the ID. A malformed ID is refused before
any request.

```bash
export BIOMCP_FHIR_BASE="${BIOMCP_PROVIDER_CONTRACT_BASE:?provider fixture base is not configured}/fhir"
set +e
out="$(../../tools/biomcp-ci get patient SYNTH-PT-MISSING 2>&1)"; status=$?
set -e
test "$status" -ne 0
echo "$out" | mustmatch 'Error: the FHIR server has no record with that patient ID.'
set +e
out="$(../../tools/biomcp-ci get patient 'a/b' 2>&1)"; status=$?
set -e
test "$status" -ne 0
echo "$out" | mustmatch like 'a patient ID must be 1-64 characters of letters, digits, hyphens, or periods'
```

## An Unset Base Names The Variable

```bash
set +e
out="$(env -u BIOMCP_FHIR_BASE ../../tools/biomcp-ci get patient SYNTH-PT-1 2>&1)"; status=$?
set -e
test "$status" -ne 0
echo "$out" | mustmatch like 'set BIOMCP_FHIR_BASE to its base URL'
```

## Patient Search Waits For Its Ticket

```bash
set +e
out="$(../../tools/biomcp-ci search patient 2>&1)"; status=$?
set -e
test "$status" -ne 0
echo "$out" | mustmatch like 'search patient is not yet available.'
```

## Discovery Names The Command And Its Limits

```bash
../../tools/biomcp-ci list patient | mustmatch like '- `get patient <id> conditions` - this patient'"'"'s FHIR Condition records'
../../tools/biomcp-ci list patient | mustmatch like '`serve-http` refuses them'
../../tools/biomcp-ci get patient --help | mustmatch like 'Reads one patient from the FHIR server named in BIOMCP_FHIR_BASE.'
```

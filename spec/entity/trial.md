# Trial Queries

Trial search is where BioMCP turns disease, intervention, and eligibility intent
into a shortlist a human can actually triage. These batch-A canaries keep the
search table, alias handling, count transparency, and detail-card sections honest.

## Condition-First Search

Condition search should still look like a trial table, not a blob of text, and
the visible query echo should confirm which narrowing path ran.

```bash
../../tools/biomcp-ci search trial -c melanoma -s recruiting --limit 3 | mustmatch like '# Trial Search Results
Query: condition=melanoma, status=recruiting
|NCT ID|Title|Status|Phase|Conditions|'
```

## Alias-Normalized Intervention Search

Brand-name intervention searches should normalize to the same shared drug
identity surface that trial help text documents, instead of hiding the alias
rewrite inside opaque result rows.

```bash
../../tools/biomcp-ci search trial -i Keytruda --limit 3 | mustmatch like '# Trial Search Results
Query: intervention=pembrolizumab
Matched Intervention'
```

## Empty Filtered Search Broadening Hint

A filtered trial search that returns zero rows should not silently imply that no
trials exist. BioMCP keeps the original filters honest but prints concrete
broadening guidance.

```bash
bash ../fixtures/setup-ctgov-intervention-alias-spec-fixture.sh ../..
. ../../.cache/spec-ctgov-intervention-alias-env
trap 'bash ../fixtures/cleanup-ctgov-intervention-alias-spec-fixture.sh ../..' EXIT
../../tools/biomcp-ci search trial -c melanoma --facility "University of Michigan" --mutation "EGFR L858R" --status recruiting --lat 42.36 --lon -71.06 --distance 100 --limit 3 | mustmatch like 'No trials found matching the filters.
Try broadening the filtered search:
- loosen or drop `--mutation`; it is an exact free-text boolean search
- widen `--distance` or remove the geo filter
- relax `--status` to include non-recruiting or not-yet-recruiting trials
- try `--biomarker <gene>`'
```

The same empty filtered search should give JSON callers machine-readable next
commands rather than a bare `results: []`.

```bash
bash ../fixtures/setup-ctgov-intervention-alias-spec-fixture.sh ../..
. ../../.cache/spec-ctgov-intervention-alias-env
trap 'bash ../fixtures/cleanup-ctgov-intervention-alias-spec-fixture.sh ../..' EXIT
../../tools/biomcp-ci --json search trial -c melanoma --facility "University of Michigan" --mutation "EGFR L858R" --status recruiting --lat 42.36 --lon -71.06 --distance 100 --limit 3 \
  | jq -r '.count, (.results | length), ._meta.next_commands[]?' \
  | mustmatch like '0
0
biomcp search trial -c melanoma --facility "University of Michigan" -s recruiting --lat 42.36 --lon -71.06 --distance 100
biomcp search trial -c melanoma --facility "University of Michigan" -s recruiting --mutation "EGFR L858R" --lat 42.36 --lon -71.06 --distance 200
biomcp search trial -c melanoma --facility "University of Michigan" --mutation "EGFR L858R" --lat 42.36 --lon -71.06 --distance 100
biomcp search trial -c melanoma --facility "University of Michigan" -s recruiting --biomarker EGFR --lat 42.36 --lon -71.06 --distance 100
biomcp list trial'
```

## Age-Only Count Transparency

The fast count path cannot fully apply age filtering upstream, so BioMCP should
stay explicit that the returned total is approximate.

```bash
../../tools/biomcp-ci search trial --age 0.5 --count-only | mustmatch '/^Total: .* [(]approximate, age post-filtered[)]$/'
```

## Trial Detail & Eligibility

When the user asks for eligibility and locations, the card should expose those
sections directly instead of forcing a second fetch or a hidden pagination path.

```bash
../../tools/biomcp-ci get trial NCT02576665 eligibility locations | mustmatch like '## Eligibility (ClinicalTrials.gov)
## Locations (ClinicalTrials.gov)
| Facility | City | Country | Status | Contact |'
```

## Trial Contacts Preserve Email and Structured Eligibility

When a user asks for contacts with eligibility and locations, the detail card
should show the action-critical central contact, site email, structured sex/age
eligibility, and full criteria without requiring raw ClinicalTrials.gov JSON.

```bash
bash ../fixtures/setup-ctgov-intervention-alias-spec-fixture.sh ../..
. ../../.cache/spec-ctgov-intervention-alias-env
trap 'bash ../fixtures/cleanup-ctgov-intervention-alias-spec-fixture.sh ../..' EXIT
../../tools/biomcp-ci get trial NCT41300001 contacts eligibility locations | mustmatch like '## Contacts (ClinicalTrials.gov)
Central Contact
Central Coordinator
central@example.test
site@example.test
## Eligibility (ClinicalTrials.gov)
Sex: Female
Eligible Ages: 2 Years to 18 Years
Key inclusion: confirmed SHANK3-related neurodevelopmental disorder.
## Locations (ClinicalTrials.gov)'
```

The `contacts` section needs site context to label site contacts, but JSON should
not expose the full locations section unless the user asks for it.

```bash
bash ../fixtures/setup-ctgov-intervention-alias-spec-fixture.sh ../..
. ../../.cache/spec-ctgov-intervention-alias-env
trap 'bash ../fixtures/cleanup-ctgov-intervention-alias-spec-fixture.sh ../..' EXIT
../../tools/biomcp-ci --json get trial NCT41300001 contacts \
  | jq -r '[.contacts[]? | select(.level == "site") | .facility][0], has("locations"), has("eligibility")' \
  | mustmatch like 'Rare Disease Center
false
false'
```

## Location Pagination Help Declares Its Flags

Location paging is part of the trial detail surface, so the paged locations
example must be discoverable from the same help page that teaches it. If the
example mentions a pagination flag, that flag belongs in `get trial` options.

```bash
../../tools/biomcp-ci get trial --help \
  | awk '/^EXAMPLES:/{capture=1; next} /^See also:/{capture=0} capture' \
  | mustmatch like 'biomcp get trial NCT02576665 --source ctgov eligibility'
../../tools/biomcp-ci get trial --help \
  | awk '/^EXAMPLES:/{capture=1; next} /^See also:/{capture=0} capture' \
  | mustmatch '/biomcp get trial NCT02576665 --offset [0-9]+ --limit [0-9]+ locations/'
```

## Source-Provided Intervention Aliases in JSON

ClinicalTrials.gov can attach alternate names directly to an intervention. BioMCP
should preserve that source evidence in JSON instead of leaving agents with only
the investigational code.

```bash
bash ../fixtures/setup-ctgov-intervention-alias-spec-fixture.sh ../..
. ../../.cache/spec-ctgov-intervention-alias-env
trap 'bash ../fixtures/cleanup-ctgov-intervention-alias-spec-fixture.sh ../..' EXIT
../../tools/biomcp-ci --json get trial NCT02136914 \
  | jq -r '.intervention_details[]? | select(.name == "ADS-5102") | .other_names[]?' \
  | mustmatch like "amantadine HCl extended release"
```

## Source-Provided Intervention Aliases in Markdown

The same alias belongs in the human-readable intervention card so a clinician or
agent can see the source-provided follow-up name without inspecting raw CTGov.

```bash
bash ../fixtures/setup-ctgov-intervention-alias-spec-fixture.sh ../..
. ../../.cache/spec-ctgov-intervention-alias-env
trap 'bash ../fixtures/cleanup-ctgov-intervention-alias-spec-fixture.sh ../..' EXIT
../../tools/biomcp-ci get trial NCT02136914 \
  | awk '/^## Interventions / {capture=1} capture && /^## / && !/^## Interventions / {exit} capture {print}' \
  | mustmatch like '## Interventions (ClinicalTrials.gov)'
../../tools/biomcp-ci get trial NCT02136914 \
  | awk '/^## Interventions / {capture=1} capture && /^## / && !/^## Interventions / {exit} capture {print}' \
  | grep -F "ADS-5102" \
  | mustmatch like "amantadine HCl extended release"
```

## Investigational Codes Avoid Brittle Drug Cards

If CTGov names an investigational intervention code and also supplies an
alternate name, BioMCP should not advertise a drug-card lookup for the raw code
unless that identity is known to resolve.

```bash
bash ../fixtures/setup-ctgov-intervention-alias-spec-fixture.sh ../..
. ../../.cache/spec-ctgov-intervention-alias-env
trap 'bash ../fixtures/cleanup-ctgov-intervention-alias-spec-fixture.sh ../..' EXIT
../../tools/biomcp-ci --json get trial NCT02136914 \
  | jq -r '._meta.next_commands[]?' \
  | mustmatch not like "biomcp get drug ADS-5102"
```

## Alias-Based Follow-Ups Stay Search-Safe

A safe next step can still use the intervention evidence, but it should stay in
a search or article context and carry the source-provided alias forward.

```bash
bash ../fixtures/setup-ctgov-intervention-alias-spec-fixture.sh ../..
. ../../.cache/spec-ctgov-intervention-alias-env
trap 'bash ../fixtures/cleanup-ctgov-intervention-alias-spec-fixture.sh ../..' EXIT
../../tools/biomcp-ci --json get trial NCT02136914 \
  | jq -r '._meta.next_commands[]? | select((startswith("biomcp search drug ") or startswith("biomcp search article ")) and contains("amantadine HCl extended release"))' \
  | mustmatch like "amantadine HCl extended release"
```

## CTGov Source Strings Stay Shell-Safe in Next Commands

ClinicalTrials.gov condition and alias values are untrusted source text, but
BioMCP presents them inside copy-pasteable next commands. Shell-active text must
be escaped in the emitted commands while preserving the visible source strings.

<!-- mustmatch-lint: skip -->

```bash run id=ctgov-shell-safe-next-commands
bash ../fixtures/setup-ctgov-intervention-alias-spec-fixture.sh ../..
. ../../.cache/spec-ctgov-intervention-alias-env
trap 'bash ../fixtures/cleanup-ctgov-intervention-alias-spec-fixture.sh ../..' EXIT
rm -f /tmp/biomcp-357-pwned
json_out="$(../../tools/biomcp-ci --json get trial NCT35700001)"
condition_cmd="$(printf '%s\n' "$json_out" | jq -r '._meta.next_commands[]? | select(startswith("biomcp search disease --query "))')"
alias_cmd="$(printf '%s\n' "$json_out" | jq -r '._meta.next_commands[]? | select(startswith("biomcp search drug -q "))')"
printf '%s\n' "$condition_cmd" "$alias_cmd"
bash -c 'condition_cmd="$1"; alias_cmd="$2"; eval "set -- $condition_cmd"; printf "condition=%s\n" "$5"; eval "set -- $alias_cmd"; printf "alias=%s\n" "$5"' _ "$condition_cmd" "$alias_cmd"
test ! -e /tmp/biomcp-357-pwned
rm -f /tmp/biomcp-357-pwned
```

```text expect=ctgov-shell-safe-next-commands contains
biomcp search disease --query "quoted \$(touch /tmp/biomcp-357-pwned) \"condition\""
biomcp search drug -q "alias \$(touch /tmp/biomcp-357-pwned) \"dose\""
condition=quoted $(touch /tmp/biomcp-357-pwned) "condition"
alias=alias $(touch /tmp/biomcp-357-pwned) "dose"
```

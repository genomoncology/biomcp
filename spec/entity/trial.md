# Trial Queries

Trial search is where BioMCP turns disease, intervention, and eligibility intent
into a shortlist a human can actually triage. These routine contracts replay
captured CT.gov responses and the established NCI source fixture through the
public CLI.

## Condition-First Search

Condition search should still look like a trial table, not a blob of text, and
the visible query echo should confirm which narrowing path ran.

```bash
../../tools/biomcp-ci search trial -c melanoma -s recruiting --limit 3 | mustmatch like '# Trial Search Results
Query: condition=melanoma, status=recruiting
|NCT ID|Title|Status|Phase|Conditions|'
```

## Terminal Pagination

A page that reaches the end of a small ClinicalTrials.gov result set must stop an
agent cleanly: a known total takes precedence over any stale provider cursor.
This live query deliberately requests 50 rows so its modest condition result set
is exhausted in one bounded request.

```bash
../../tools/biomcp-ci --json search trial -c "Phelan-McDermid Syndrome" --limit 50 \
  | jq -e '.pagination.total != null and .pagination.returned > 0 and .pagination.returned == .pagination.total and .pagination.has_more == false and .pagination.next_page_token == null' \
  | mustmatch 'true'
```

## Cursor Pagination Continues When the Registry Omits a Later Total

<!-- mustmatch-lint: skip -->

ClinicalTrials.gov reports the total with the initial page but can omit it on a
later cursor request. The opaque token must remain usable in that case rather
than treating the size of the returned page as a terminal total.

```bash run id=trial-cursor-first
../../tools/biomcp-ci --json search trial -c "Phelan-McDermid Syndrome" --limit 5
```

```json expect=trial-cursor-first contains
{
  "pagination": { "has_more": true }
}
```

```bash run id=trial-cursor-next uses=trial-cursor-first
../../tools/biomcp-ci --json search trial -c "Phelan-McDermid Syndrome" --limit 5 \
  --next-page '{{trial-cursor-first.pagination.next_page_token}}' \
  | jq '.pagination.has_more and (.pagination.next_page_token != null)'
```

```text expect=trial-cursor-next
true
```

## Simple mutation search verifies molecular inclusion

For simple molecular text, broad CTGov discovery is followed by a registry
eligibility check. A trial with a positive inclusion requirement remains, while
a recorded exclusion-only match is removed; the live check avoids pinning the
changing result total.

```bash
../../tools/biomcp-ci --json search trial -c "non-small cell lung cancer" --mutation "EGFR L858R" --limit 50 \
  | jq -r '([.results[].nct_id] | index("NCT06604689") != null), ([.results[].nct_id] | index("NCT06382129") == null)' \
  | mustmatch like 'true
true'
```

## NCI Condition Search

The NCI source keeps ordinary condition lookup available independently of CTGov.
A bounded melanoma search should return an NCI trial rather than entering any
condition-planning path.

```bash
"$BIOMCP_BIN" --json search trial -c melanoma --source nci --limit 1 \
  | jq -r '(.count >= 1) and (.results[0].nct_id | startswith("NCT")) and (.results[0].title | length > 0)' \
  | mustmatch 'true'
```

## Complete JSON Conditions and Disclosed Markdown Abbreviation

JSON search results retain every condition in the recorded provider response.

```bash
../../tools/biomcp-ci --json search trial -i Keytruda --no-alias-expand --limit 3 \
  | jq -e '[.results[] | select(.nct_id == "NCT03590054") | .conditions | length] == [21]' \
  | mustmatch 'true'
```

```bash
"$BIOMCP_BIN" --json search trial -c melanoma --source nci --limit 1 \
  | jq -e '[.results[] | select(.nct_id == "NCT05929768") | .conditions | length] == [26]' \
  | mustmatch 'true'
```

The bounded Markdown table names the complete count when it abbreviates the
same CTGov result.

```bash
../../tools/biomcp-ci search trial -i Keytruda --no-alias-expand --limit 3 \
  | awk -F'|' '$2 == "NCT03590054" {print}' \
  | mustmatch like 'abridged; 21 conditions total'
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

## Canonical Trial Age Bounds

ClinicalTrials.gov age strings are parsed once for filtering and public output.
The provider spelling remains visible, while JSON exposes the validated number
and unit object and retains the registry's no-limit sentinel.

```bash
../../tools/biomcp-ci --json get trial NCT60000001 eligibility \
  | jq -c '{minimum_age:.eligibility.minimum_age,maximum_age:.eligibility.maximum_age}' \
  | mustmatch '{"minimum_age":{"number":6.0,"unit":"months","original":"6 Months"},"maximum_age":{"number":null,"unit":null,"original":"N/A"}}'
../../tools/biomcp-ci get trial NCT60000001 \
  | grep -F 'Eligible Ages: 6 Months to Any age' \
  | mustmatch 'Eligible Ages: 6 Months to Any age'
../../tools/biomcp-ci get trial NCT60000001 eligibility \
  | grep -F 'Eligible Ages: 6 Months to Any age' \
  | mustmatch 'Eligible Ages: 6 Months to Any age'
```

Inclusive age filtering excludes the record immediately below six months and
retains it at exactly six months.

```bash
../../tools/biomcp-ci --json search trial -c 'Canonical Age Fixture' --age 0.49 --limit 5 \
  | jq -e '[.results[].nct_id] | index("NCT60000001") == null' \
  | mustmatch 'true'
../../tools/biomcp-ci --json search trial -c 'Canonical Age Fixture' --age 0.5 --limit 5 \
  | jq -e '[.results[].nct_id] | index("NCT60000001") != null' \
  | mustmatch 'true'
```

Batch detail returns the same exact objects as direct detail.

```bash
direct="$(../../tools/biomcp-ci --json get trial NCT60000001 eligibility)"
batch="$(../../tools/biomcp-ci --json batch trial NCT60000001 --sections eligibility)"
jq -n -e --argjson direct "$direct" --argjson batch "$batch" \
  '$direct.eligibility == $batch.items[0].result.eligibility' \
  | mustmatch 'true'
```

## Trial Detail & Eligibility

When the user asks for eligibility and locations, the card should expose those
sections directly instead of forcing a second fetch or a hidden pagination path.

```bash
../../tools/biomcp-ci get trial NCT02576665 eligibility locations | mustmatch like '## Eligibility (ClinicalTrials.gov)
## Locations (ClinicalTrials.gov)
| Facility | City | Postal code | Country | Status | Contact |
| Sarah Cannon Research Institute | Denver, Colorado | 80218 | United States | - | - |'
../../tools/biomcp-ci --json get trial NCT02576665 locations \
  | jq -r '.locations[] | select(.facility == "Sarah Cannon Research Institute") | .postal_code' \
  | mustmatch '80218'
```

## Partially Described Trial Sites Remain Visible

ClinicalTrials.gov may identify a site without naming its facility. BioMCP
should preserve all such sites, their location pagination, and the provider's
city and country while leaving the absent facility key absent.

```bash
../../tools/biomcp-ci --json get trial NCT00791778 --limit 59 locations \
  | jq -e '(.locations | length == 59) and (.location_pagination.total == 59) and ([.locations[] | (has("facility") | not) and (.city | length > 0) and (.country | length > 0)] | all)'
../../tools/biomcp-ci get trial NCT00791778 --limit 59 locations \
  | mustmatch like '| - | La Jolla, California | 92037 | United States |'
```

## Trial Contacts Preserve Email and Structured Eligibility

When a user asks for contacts with eligibility and locations, the detail card
should show the action-critical central contact, site email, structured sex/age
eligibility, and full criteria without requiring raw ClinicalTrials.gov JSON.

```bash
../../tools/biomcp-ci get trial NCT41300001 contacts eligibility locations \
  | awk '/^## Contacts/{inside=1} /^## Eligibility/{exit} inside' \
  | mustmatch like '### Central Contact
- Name: Central Coordinator
- Role: CONTACT
- Email: central@example.test
- Phone: 555-0100'
../../tools/biomcp-ci get trial NCT41300001 contacts eligibility locations \
  | awk '/^## Eligibility/{inside=1} /^## Locations/{exit} inside' \
  | mustmatch like 'Sex: Female
Eligible Ages: 2 Years to 18 Years
Key inclusion: confirmed SHANK3-related neurodevelopmental disorder.'
../../tools/biomcp-ci get trial NCT41300001 contacts eligibility locations \
  | awk '/^## Locations/{inside=1} inside && /^## / && !/^## Locations/{exit} inside' \
  | mustmatch like '| Rare Disease Center | Ann Arbor, Michigan | - | United States | RECRUITING | Site Coordinator (CONTACT) 555-0199 site@example.test |'
```

The `contacts` section needs site context to label site contacts, but JSON should
not expose the full locations section unless the user asks for it.

```bash
../../tools/biomcp-ci --json get trial NCT41300001 contacts \
  | jq -r '[.contacts[]? | select(.level == "site") | .facility][0], has("locations"), has("eligibility")' \
  | mustmatch like 'Rare Disease Center
false
false'
```

When contacts and locations are requested together, the top-level site
contacts follow the selected location page while central contacts remain.

```bash
../../tools/biomcp-ci --json get trial NCT41300001 --offset 20 --limit 3 contacts locations \
  | jq -e '(.locations | length == 3) and ([.locations[].facility] == ["Fixture Site 21", "Fixture Site 22", "Fixture Site 23"]) and ([.contacts[].name] == ["Central Coordinator", "Site Coordinator 21", "Site Coordinator 22", "Site Coordinator 23"]) and (.location_pagination == {"total": 25, "offset": 20, "limit": 3, "has_more": true})' \
  | mustmatch 'true'
../../tools/biomcp-ci get trial NCT41300001 --offset 20 --limit 3 contacts locations \
  | grep -E '^- Name: (Central Coordinator|Site Coordinator 2[1-3])$|^\*Locations:' \
  | mustmatch like '- Name: Central Coordinator
- Name: Site Coordinator 21
- Name: Site Coordinator 22
- Name: Site Coordinator 23
*Locations: showing 3 of 25 (offset 20, limit 3, more available)*'
../../tools/biomcp-ci get trial NCT41300001 --offset 20 --limit 3 contacts locations \
  | mustmatch not like 'site@example.test'
```

An explicit locations page is rendered in full, even above the default
20-site page size. Generic `all` Markdown keeps its disclosed display cap and
does not show a top-level contact for a hidden site.

```bash
../../tools/biomcp-ci get trial NCT41300001 --limit 25 contacts locations \
  | awk '/^## Locations/{inside=1} inside{print}' \
  | mustmatch like '| Fixture Site 25 | Fixture City 25, Michigan | - | United States | RECRUITING | Site Coordinator 25 (CONTACT) 555-0025 site-25@example.test |
*Locations: showing 25 of 25 (offset 0, limit 25)*'
../../tools/biomcp-ci get trial NCT41300001 all \
  | mustmatch like 'Locations: showing 20 of 25 (display cap 20).'
../../tools/biomcp-ci get trial NCT41300001 all \
  | awk '/^## Locations/{inside=1; next} inside && /^\| (Rare Disease Center|Fixture Site)/{count++} END{print count}' \
  | mustmatch '20'
../../tools/biomcp-ci get trial NCT41300001 all \
  | mustmatch not like 'site-21@example.test'
```

## Every Named Site Contact Reaches Its Location

Location JSON preserves every named site contact in provider order while the
legacy scalar aliases continue to describe the literal first source contact.

```bash
../../tools/biomcp-ci --json get trial NCT00000000 contacts locations \
  | jq -e '([.locations[0].contacts[] | [.name, .role]] == [["First Synthetic Contact", "CONTACT"], ["Second Synthetic Contact", "BACKUP"]]) and ([.contacts[] | select(.level == "site") | [.name, .role]] == [["First Synthetic Contact", "CONTACT"], ["Second Synthetic Contact", "BACKUP"]]) and (.locations[0].contacts | length == 2) and ([.contacts[] | select(.level == "site")] | length == 2) and (.locations[0].contact_name == "First Synthetic Contact")' \
  | mustmatch 'true'
../../tools/biomcp-ci get trial NCT00000000 contacts locations \
  | grep -F '| Synthetic Research Site | Example City | - | United States | RECRUITING | First Synthetic Contact (CONTACT)<br>Second Synthetic Contact (BACKUP) |' \
  | mustmatch like 'First Synthetic Contact (CONTACT)<br>Second Synthetic Contact (BACKUP)'
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
../../tools/biomcp-ci --json get trial NCT02136914 \
  | jq -r '.intervention_details[]? | select(.name == "ADS-5102") | .other_names[]?' \
  | mustmatch like "amantadine HCl extended release"
```

## Source-Provided Intervention Aliases in Markdown

The same alias belongs in the human-readable intervention card so a clinician or
agent can see the source-provided follow-up name without inspecting raw CTGov.

```bash
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
../../tools/biomcp-ci --json get trial NCT02136914 \
  | jq -r '._meta.next_commands[]?' \
  | mustmatch not like "biomcp get drug ADS-5102"
```

## Alias-Based Follow-Ups Stay Search-Safe

A safe next step can still use the intervention evidence, but it should stay in
a search or article context and carry the source-provided alias forward.

```bash
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
rm -f /tmp/biomcp-357-pwned
rm -f /tmp/biomcp-trial-title-expanded
trap 'rm -f /tmp/biomcp-trial-title-expanded' EXIT
markdown_out="$(../../tools/biomcp-ci get trial NCT35700001)"
json_out="$(../../tools/biomcp-ci --json get trial NCT35700001)"
condition_cmd="$(printf '%s\n' "$json_out" | jq -r '._meta.next_commands[]? | select(startswith("biomcp search disease --query "))')"
alias_cmd="$(printf '%s\n' "$json_out" | jq -r '._meta.next_commands[]? | select(startswith("biomcp search drug -q "))')"
markdown_publication_cmd="$(printf '%s\n' "$markdown_out" | sed -n 's/^  \(biomcp search article .* --limit 5\)   - find publications.*/\1/p')"
json_publication_cmd="$(printf '%s\n' "$json_out" | jq -r '._meta.next_commands[]? | select(startswith("biomcp search article ") and endswith(" --limit 5"))')"
test "$markdown_publication_cmd" = "$json_publication_cmd"
expected_query="NCT35700001 Alpha\\path's \$(touch /tmp/biomcp-trial-title-expanded) \"quoted\" \$HOME; \`uname\`"
printf '%s\n' "$condition_cmd" "$alias_cmd" "$markdown_publication_cmd" "$json_publication_cmd"
bash -c 'condition_cmd="$1"; alias_cmd="$2"; eval "set -- $condition_cmd"; printf "condition=%s\n" "$5"; eval "set -- $alias_cmd"; printf "alias=%s\n" "$5"' _ "$condition_cmd" "$alias_cmd"
bash -c 'set -eu; command="$1"; expected="$2"; HOME=/tmp/biomcp-fixed-home; export HOME; trap "rm -f /tmp/biomcp-trial-title-expanded" EXIT; eval "set -- $command"; test "$#" -eq 9; test "$7" = "$expected"; test ! -e /tmp/biomcp-trial-title-expanded; printf "publication=%s\n" "$7"' _ "$json_publication_cmd" "$expected_query"
test ! -e /tmp/biomcp-357-pwned
test ! -e /tmp/biomcp-trial-title-expanded
rm -f /tmp/biomcp-357-pwned
```

```text expect=ctgov-shell-safe-next-commands contains
biomcp search disease --query "quoted \$(touch /tmp/biomcp-357-pwned) \"condition\""
biomcp search drug -q "alias \$(touch /tmp/biomcp-357-pwned) \"dose\""
biomcp search article --drug SAFE-357 -q "NCT35700001 Alpha\\path's \$(touch /tmp/biomcp-trial-title-expanded) \"quoted\" \$HOME; \`uname\`" --limit 5
condition=quoted $(touch /tmp/biomcp-357-pwned) "condition"
alias=alias $(touch /tmp/biomcp-357-pwned) "dose"
publication=NCT35700001 Alpha\path's $(touch /tmp/biomcp-trial-title-expanded) "quoted" $HOME; `uname`
```

## Observed Trial Provider Requests

The shared fixtures record the production requests consumed by the cursor,
detail, mutation, and NCI routes.

```bash
grep -F 'query.cond=Phelan-McDermid+Syndrome&countTotal=true&pageSize=50' "$BIOMCP_CTGOV_INTERVENTION_ALIAS_REQUEST_LOG" | mustmatch like 'fields=NCTId%2CBriefTitle'
grep -F 'query.cond=non-small+cell+lung+cancer' "$BIOMCP_CTGOV_INTERVENTION_ALIAS_REQUEST_LOG" | grep -F 'pageSize=50' | mustmatch like 'EGFR+L858R'
grep -F '/api/v2/studies/NCT02576665?fields=' "$BIOMCP_CTGOV_INTERVENTION_ALIAS_REQUEST_LOG" | grep -F 'LocationFacility' | mustmatch like 'EligibilityCriteria'
grep -F '/api/v2/studies/NCT00791778?fields=BriefSummary%2CBriefTitle%2CCentralContactEMail%2CCentralContactName%2CCentralContactPhone%2CCentralContactRole%2CCompletionDate%2CCondition%2CEnrollmentCount%2CInterventionDescription%2CInterventionName%2CInterventionOtherName%2CInterventionType%2CLeadSponsorName%2CLocationCity%2CLocationContactEMail%2CLocationContactName%2CLocationContactPhone%2CLocationContactRole%2CLocationCountry%2CLocationFacility%2CLocationGeoPoint%2CLocationState%2CLocationStatus%2CLocationZip%2CMaximumAge%2CMinimumAge%2CNCTId%2COverallStatus%2CPhase%2CStartDate%2CStudyType%2CWhyStopped' "$BIOMCP_CTGOV_INTERVENTION_ALIAS_REQUEST_LOG" | mustmatch like '/api/v2/studies/NCT00791778?fields='
grep -F '/api/v2/studies/NCT00000000?fields=BriefSummary%2CBriefTitle%2CCentralContactEMail%2CCentralContactName%2CCentralContactPhone%2CCentralContactRole%2CCompletionDate%2CCondition%2CEnrollmentCount%2CInterventionDescription%2CInterventionName%2CInterventionOtherName%2CInterventionType%2CLeadSponsorName%2CLocationCity%2CLocationContactEMail%2CLocationContactName%2CLocationContactPhone%2CLocationContactRole%2CLocationCountry%2CLocationFacility%2CLocationGeoPoint%2CLocationState%2CLocationStatus%2CLocationZip%2CMaximumAge%2CMinimumAge%2CNCTId%2COverallStatus%2CPhase%2CStartDate%2CStudyType%2CWhyStopped' "$BIOMCP_CTGOV_INTERVENTION_ALIAS_REQUEST_LOG" | mustmatch like '/api/v2/studies/NCT00000000?fields='
grep -F 'GET /nci/api/v2/trials?keyword=melanoma&size=1&from=0' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" | mustmatch like 'keyword=melanoma'
```

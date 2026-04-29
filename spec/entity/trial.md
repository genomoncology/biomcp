# Trial Queries

Trial search is where BioMCP turns disease, intervention, and eligibility intent
into a shortlist a human can actually triage. These batch-A canaries keep the
search table, alias handling, count transparency, and detail-card sections honest.

## Condition-First Search

Condition search should still look like a trial table, not a blob of text, and
the visible query echo should confirm which narrowing path ran.

```bash
out="$(../../tools/biomcp-ci search trial -c melanoma -s recruiting --limit 3)"
echo "$out" | mustmatch like "# Trial Search Results"
echo "$out" | mustmatch like "Query: condition=melanoma, status=recruiting"
echo "$out" | mustmatch like "|NCT ID|Title|Status|Phase|Conditions|"
```

## Alias-Normalized Intervention Search

Brand-name intervention searches should normalize to the same shared drug
identity surface that trial help text documents, instead of hiding the alias
rewrite inside opaque result rows.

```bash
out="$(../../tools/biomcp-ci search trial -i Keytruda --limit 3)"
echo "$out" | mustmatch like "# Trial Search Results"
echo "$out" | mustmatch like "Query: intervention=pembrolizumab"
echo "$out" | mustmatch like "Matched Intervention"
```

## Age-Only Count Transparency

The fast count path cannot fully apply age filtering upstream, so BioMCP should
stay explicit that the returned total is approximate.

```bash
../../tools/biomcp-ci search trial --age 0.5 --count-only \
  | mustmatch '/^Total: [0-9]+ \(approximate, age post-filtered\)$/'
```

## Trial Detail & Eligibility

When the user asks for eligibility and locations, the card should expose those
sections directly instead of forcing a second fetch or a hidden pagination path.

```bash
out="$(../../tools/biomcp-ci get trial NCT02576665 eligibility locations)"
echo "$out" | mustmatch like "## Eligibility (ClinicalTrials.gov)"
echo "$out" | mustmatch like "## Locations (ClinicalTrials.gov)"
echo "$out" | mustmatch like "| Facility | City | Country | Status | Contact |"
```

## Source-Provided Intervention Aliases in JSON

ClinicalTrials.gov can attach alternate names directly to an intervention. BioMCP
should preserve that source evidence in JSON instead of leaving agents with only
the investigational code.

```bash
bash ../fixtures/setup-ctgov-intervention-alias-spec-fixture.sh ../..
. ../../.cache/spec-ctgov-intervention-alias-env
trap 'bash ../fixtures/cleanup-ctgov-intervention-alias-spec-fixture.sh ../..' EXIT
json_out="$(../../tools/biomcp-ci --json get trial NCT02136914)"
echo "$json_out" | jq -r '.intervention_details[]? | select(.name == "ADS-5102") | .other_names[]?' \
  | mustmatch like "amantadine HCl extended release"
```

## Source-Provided Intervention Aliases in Markdown

The same alias belongs in the human-readable intervention card so a clinician or
agent can see the source-provided follow-up name without inspecting raw CTGov.

```bash
bash ../fixtures/setup-ctgov-intervention-alias-spec-fixture.sh ../..
. ../../.cache/spec-ctgov-intervention-alias-env
trap 'bash ../fixtures/cleanup-ctgov-intervention-alias-spec-fixture.sh ../..' EXIT
out="$(../../tools/biomcp-ci get trial NCT02136914)"
echo "$out" | mustmatch like "## Interventions (ClinicalTrials.gov)"
echo "$out" | grep -F "ADS-5102" \
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
json_out="$(../../tools/biomcp-ci --json get trial NCT02136914)"
echo "$json_out" | jq -r '._meta.next_commands[]?' \
  | mustmatch not like "biomcp get drug ADS-5102"
```

## Alias-Based Follow-Ups Stay Search-Safe

A safe next step can still use the intervention evidence, but it should stay in
a search or article context and carry the source-provided alias forward.

```bash
bash ../fixtures/setup-ctgov-intervention-alias-spec-fixture.sh ../..
. ../../.cache/spec-ctgov-intervention-alias-env
trap 'bash ../fixtures/cleanup-ctgov-intervention-alias-spec-fixture.sh ../..' EXIT
json_out="$(../../tools/biomcp-ci --json get trial NCT02136914)"
echo "$json_out" | jq -r '._meta.next_commands[]? | select((startswith("biomcp search drug ") or startswith("biomcp search article ")) and contains("amantadine HCl extended release"))' \
  | mustmatch like "amantadine HCl extended release"
```

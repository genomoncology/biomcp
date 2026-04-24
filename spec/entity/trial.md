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

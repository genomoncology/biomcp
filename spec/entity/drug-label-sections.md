# Drug Label Sections

`get drug NAME label` must return whole label sections in JSON and say why a
label is missing, in the four section states. These rows run against recorded
openFDA responses: the osimertinib and mobocertinib full-text searches whose
top-ranked rows belong to other drugs, the gefitinib field-scoped match whose
sections run past the Markdown cap, and the pembrolizumab identity-only record
whose match carries no section text.

## Sparse-metadata labels resolve through the guarded full-text fallback

Osimertinib's current SPL record carries no populated `openfda.generic_name` or
`brand_name`, so the field-scoped lookup finds nothing. BioMCP falls back to a
full-text search and keeps only the record whose own identity matches the drug.

```bash
../../tools/biomcp-ci --json get drug osimertinib label --raw | jq -e '(.section_outcomes.label == {"outcome":"data","sources":["OpenFDA label"]}) and (.label.indications | test("TAGRISSO is a kinase inhibitor")) and ([._meta.section_sources[] | select(.key == "label")] == [{"key":"label","label":"FDA Label","outcome":"data","sources":["OpenFDA label"]}])' | mustmatch 'true'
../../tools/biomcp-ci --json get drug osimertinib label | jq -e '(.label | tostring | test("amivantamab|Rybrevant|Lazcluze|Datroway"; "i")) | not' | mustmatch 'true'
../../tools/biomcp-ci get drug osimertinib label | mustmatch like '## FDA Label'
../../tools/biomcp-ci get drug osimertinib label | mustmatch not like '(?i)amivantamab|Rybrevant|Lazcluze|Datroway'
```

The fallback fires only after the field-scoped miss, and the guarded record is
chosen from ranked full-text rows. Exactly two openFDA label request shapes
exist for this drug: the field-scoped narrow lookup and the quoted full-text
phrase.

```bash
label_requests=$(grep -F 'GET /openfda/drug/label.json?search=' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" | grep osimertinib)
narrow=$(printf '%s\n' "$label_requests" | grep -cF 'openfda.generic_name%3A%22osimertinib%22')
fulltext=$(printf '%s\n' "$label_requests" | grep -cF 'search=%22osimertinib%22&limit=100')
printf '%s\n' "$label_requests" | grep -v -F 'openfda.generic_name' | grep -v -F 'search=%22osimertinib%22' | wc -l | mustmatch '0'
test "$narrow" -gt 0 && test "$narrow" -eq "$fulltext"
printf 'each field-scoped miss is paired with one full-text fallback\n' \
  | mustmatch 'each field-scoped miss is paired with one full-text fallback'
```

Mobocertinib's record is even harder: fourteen itraconazole labels rank ahead of
the withdrawn EXKIVITY record in the full-text response, and none of those rows
is mobocertinib. The identity guard must still pick the drug's own label.

```bash
../../tools/biomcp-ci --json get drug mobocertinib label | jq -e '(.section_outcomes.label.outcome == "data") and (.label.indication_summary[0].name | test("exon 20 insertion")) and ((.label | tostring | test("Sporanox|ITRACONAZOLE"; "i")) | not)' | mustmatch 'true'
```

## A drug with no SPL record says so in the section outcome

Adavosertib has no openFDA SPL record at all: both searches answer with no
match, so the label section settles as a source-confirmed empty whose section
outcome carries the reason, instead of a silent `label: null`.

```bash
../../tools/biomcp-ci --json get drug adavosertib label | jq -e '(has("label") | not) and (.section_outcomes.label == {"outcome":"empty","sources":["OpenFDA label"],"message":"No openFDA SPL label record matched this drug."}) and ([._meta.section_sources[] | select(.key == "label")] == [{"key":"label","label":"FDA Label","outcome":"empty","sources":["OpenFDA label"]}])' | mustmatch 'true'
../../tools/biomcp-ci get drug adavosertib label | mustmatch like '## FDA Label
No openFDA SPL label record matched this drug.'
../../tools/biomcp-ci get drug adavosertib label | mustmatch not like '(?i)unavailable'
```

A match that carries no label section text is a different reason: the
pembrolizumab fixture record identifies the drug but has no section text, so the
outcome stays an empty that names the sparse record rather than a missing
record.

```bash
../../tools/biomcp-ci --json get drug pembrolizumab label | jq -e '(has("label") | not) and (.section_outcomes.label == {"outcome":"empty","sources":["OpenFDA label"],"message":"The matched openFDA SPL record carries no label section text."})' | mustmatch 'true'
```

## JSON carries whole sections; Markdown keeps the short form

Gefitinib's recorded warnings section runs past the 2,000-character Markdown
cap. JSON must carry the whole section with no truncation note, while Markdown
keeps the capped short form that points to the full DailyMed label.

```bash
../../tools/biomcp-ci --json get drug gefitinib label --raw | jq -e '(.section_outcomes.label.outcome == "data") and (.label.warnings | length > 2000) and ((.label | tostring | contains("truncated")) | not)' | mustmatch 'true'
../../tools/biomcp-ci get drug gefitinib label --raw | mustmatch like '(truncated, 4628 chars total; full label: https://dailymed.nlm.nih.gov/dailymed/drugInfo.cfm?setid=1a3c0ce7-06a1-4e04-9106-14ddb2a866a5)'
../../tools/biomcp-ci get drug gefitinib label --raw | mustmatch not like 'Use `--raw` for the full truncated FDA label text.'
../../tools/biomcp-ci get drug gefitinib label | mustmatch like 'Markdown caps label sections at a short form. Use `biomcp get drug gefitinib label --json` for whole sections, or `--raw` for the raw label text.'
```

Summary mode keeps its approved-indication rows, and the whole-section rule
never widens the safety section, which keeps its own capped warning text. The
short-form pointer only prints when the Markdown view actually cut a section,
so gefitinib's summary — nothing carried runs past the cap — points nowhere.

```bash
../../tools/biomcp-ci get drug gefitinib label | mustmatch like '## FDA Label
### Approved Indications'
../../tools/biomcp-ci get drug gefitinib label | mustmatch not like 'Markdown caps label sections'
```

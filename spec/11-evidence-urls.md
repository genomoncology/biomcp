# Evidence URLs and Citations

This spec verifies that `get` entity cards include evidence links and next-step
command hints in both markdown and JSON output. Assertions are structural and
avoid volatile upstream values.

| Section | Command focus | Why it matters |
|---|---|---|
| Markdown cards | `get <entity>` | Confirms evidence footer links + `See also:` block |
| JSON metadata | `get <entity> --json` | Confirms `_meta.evidence_urls` + `_meta.next_commands` |
| Trial locations JSON | `get trial ... locations --json` | Confirms `location_pagination` and `_meta` coexist |

## Markdown Evidence Links

Representative markdown cards should include evidence URLs and follow-up helper commands.

```bash
gene_out="$(biomcp get gene BRAF)"
echo "$gene_out" | mustmatch like "[NCBI Gene]("
echo "$gene_out" | mustmatch like "See also:"

variant_out="$(biomcp get variant "BRAF V600E")"
echo "$variant_out" | mustmatch like "[dbSNP]("
echo "$variant_out" | mustmatch like "[COSMIC]("
echo "$variant_out" | mustmatch like "See also:"

trial_out="$(biomcp get trial NCT02576665)"
echo "$trial_out" | mustmatch like "[ClinicalTrials.gov]("
echo "$trial_out" | mustmatch like "See also:"

pgx_out="$(biomcp get pgx CYP2D6)"
echo "$pgx_out" | mustmatch like "[CPIC](https://cpicpgx.org/genes/"
echo "$pgx_out" | mustmatch like "[PharmGKB](https://www.pharmgkb.org/"
echo "$pgx_out" | mustmatch like "See also:"

ae_out="$(biomcp get adverse-event 10222779)"
echo "$ae_out" | mustmatch like "[OpenFDA]("
echo "$ae_out" | mustmatch like "See also:"
```

## JSON Metadata Contract

JSON entity output should expose evidence links, next commands, and section
provenance under `_meta`. The `section_sources` field lists each rendered
section with its upstream source name(s) so callers can attribute data without
parsing markdown.

```bash
gene_json="$(biomcp get gene BRAF --json)"
echo "$gene_json" | mustmatch like '"_meta": {'
echo "$gene_json" | mustmatch like '"evidence_urls": ['
echo "$gene_json" | mustmatch like '"next_commands": ['
echo "$gene_json" | mustmatch like '"section_sources": ['

variant_json="$(biomcp get variant "BRAF V600E" --json)"
echo "$variant_json" | mustmatch like '"label": "dbSNP"'
echo "$variant_json" | mustmatch like '"label": "COSMIC"'
echo "$variant_json" | mustmatch like '"next_commands": ['

trial_json="$(biomcp get trial NCT02576665 --json)"
echo "$trial_json" | mustmatch like '"label": "ClinicalTrials.gov"'
echo "$trial_json" | mustmatch like '"next_commands": ['

pgx_json="$(biomcp get pgx CYP2D6 --json)"
echo "$pgx_json" | mustmatch like '"label": "CPIC"'
echo "$pgx_json" | mustmatch like '"next_commands": ['

ae_json="$(biomcp get adverse-event 10222779 --json)"
echo "$ae_json" | mustmatch like '"label": "OpenFDA"'
echo "$ae_json" | mustmatch like '"next_commands": ['
```

## Trial Locations JSON Shape

Trial locations pagination metadata should remain top-level while `_meta` is added.

```bash
trial_locations_json="$(biomcp get trial NCT02576665 locations --offset 20 --limit 10 --json)"
echo "$trial_locations_json" | mustmatch like '"nct_id": "NCT02576665"'
echo "$trial_locations_json" | mustmatch like '"location_pagination": {'
echo "$trial_locations_json" | mustmatch like '"offset": 20'
echo "$trial_locations_json" | mustmatch like '"limit": 10'
echo "$trial_locations_json" | mustmatch like '"_meta": {'
```

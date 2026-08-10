---
flow: build
priority: 7
---
# Report a complete gnomAD v4 population result

This ticket absorbs superseded 0879. Filtering allele frequency, raw allele
frequency, release identity, exome/genome separation, and quality flags are
one source response and one user decision; shipping only half would remain
misleading.

## Done when

biomcp get variant <id> population queries the gnomAD GraphQL dataset enum
`gnomad_r4` directly for the resolved GRCh38 coordinate. Do not silently move
to `gnomad_r3`, `gnomad_r2_1`, or a future dataset enum. The response returns:

- dataset `gnomad_r4` and the most precise release identity the provider
  actually supplies; when the API supplies no point-release identity, report
  `gnomAD v4` and do not invent a patch version;
- exome and genome results as separate objects;
- raw allele frequency and allele counts by ancestry;
- grpmax filtering allele frequency, including faf95 and the selected group;
- quality filter flags separately for exomes and genomes;
- explicit missing, absent, and provider-failure outcomes;
- the gnomAD FAF exclusion caveat for bottlenecked groups.

Keep the existing MyVariant/legacy gnomAD and ExAC data through the first
BioMCP release containing this ticket under a separate `legacy_population`
object with its own provider and release labels. That compatibility release is
v0.8.26 if this ticket lands before that cut; otherwise it is the first later
release containing the change. Mark the object deprecated in the schema and
documentation and name the exact removal release there. Draft ticket 0946 owns
removal in the following minor release. The legacy object must not be merged
into the v4 fields or used as a fallback when direct gnomAD data is absent or
unavailable.

If the resolved variant has no trustworthy GRCh38 coordinate, the section
states that requirement instead of querying gnomAD with a GRCh37 coordinate.

## Proof required

- A RequestPlan test pins the GraphQL operation, `dataset: gnomad_r4`,
  variables, coordinate, selected fields, and response-size limits.
- A real receipted v4 response exercises grpmax FAF and one response
  exercises discordant exome/genome filters.
- Production decoding keeps each data type separate.
- JSON preserves raw flag names and machine-readable numeric values.
- Markdown expands common flags in plain language and keeps the dataset next
  to every number.
- Missing/error/status and compact-output cases are covered locally.
- No routine test reaches gnomAD.

## Authorized test changes

Design commits may restate the population model, MyVariant legacy population
fixtures, gnomAD source tests, variant population JSON/Markdown tests, skill
schemas/examples, and related specs. Mechanical construction fixes may land
with implementation while unrelated assertions remain unchanged.

The src line ceiling may rise by at most 360 lines.

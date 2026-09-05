# Variant Queries

Variant workflows need to balance exact identity with search-time normalization.
These canaries keep the stable column contracts, normalization rules, and
opt-in clinical sections without depending on brittle row counts.

## Deterministic Source Contracts

Ticket 376 moves routine variant-source proof from live/cache-backed MyVariant
and normalization-service canaries to source-local request-plan and
fixture-backed contracts. Any irreducible public availability check belongs in
an explicit release/live-smoke lane; routine specs must instead prove MyVariant
search/get request shape, identifier normalization, and Mutalyzer/
VariantValidator status mapping locally.

## Deterministic Renderer Envelope Contracts

Ticket 377 moves routine variant renderer/envelope proof into fixture-result
contracts. The deterministic tests should cover variant search JSON
`_meta.next_commands`, markdown related anchors, and normalization JSON/markdown
per-service status, warnings, and genomic-description rendering without live
MyVariant, Mutalyzer, or VariantValidator calls.

Ticket 456 keeps the default variant card cheap while making CIViC actionability
discoverable: a pure renderer test proves the cached-CIViC pointer, the bare
fallback pointer, and the CIViC-section currency caveat without a live source
call.

```bash
grep -h -F \
  -e 'Therapeutic evidence: 1 CIViC predictive item(s) / 0 assertion(s)' \
  -e 'Therapeutic evidence: see `get variant \"chr1:g.101A>T\" civic`' \
  -e 'Caveat: CIViC evidence may lag current standard of care' \
  ../../src/render/markdown/variant/tests.rs ../../templates/variant.md.j2 \
  | mustmatch like 'Therapeutic evidence: 1 CIViC predictive item(s) / 0 assertion(s)
Therapeutic evidence: see `get variant \"chr1:g.101A>T\" civic`
Caveat: CIViC evidence may lag current standard of care'
```

## Finite score thresholds

<!-- mustmatch-lint: skip -->

GERP and CADD thresholds must be finite numbers. Non-finite values are rejected
as invalid arguments instead of being sent upstream and misreported as a
confident empty result.

| str:flag | str:value | str:label |
|---|---|---|
| --gerp-min | NaN | GERP NaN |
| --gerp-min | +inf | GERP positive infinity |
| --gerp-min | -inf | GERP negative infinity |
| --gerp-min | 1e309 | GERP overflow |
| --min-cadd | NaN | CADD NaN |
| --min-cadd | +inf | CADD positive infinity |
| --min-cadd | -inf | CADD negative infinity |
| --min-cadd | 1e309 | CADD overflow |

```bash run id=non-finite-threshold exit=2 each_row="Finite score thresholds"
biomcp --json search variant --gene BRAF {{flag}}={{value}} --limit 1
```

```json expect=non-finite-threshold contains each_row="Finite score thresholds"
{
  "error": {
    "code": "invalid_argument"
  }
}
```

## Variant filter vocabularies

<!-- mustmatch-lint: skip -->

Consequence, review-status, and field-presence filters use documented vocabularies. Unknown
values are rejected locally instead of being sent upstream and reported as a
successful empty search.

| str:flag | str:value | str:label |
|---|---|---|
| --consequence | protein_altering_variant | unsupported consequence |
| --consequence | missense_variant* | malformed consequence |
| --consequence | '' | empty consequence |
| --review-status | bogus | unknown review status |
| --review-status | 2* | malformed review status |
| --review-status | '' | empty review status |
| --has | not_a_real_field_zzz | unknown required field |
| --has | revel:* | malformed required field |
| --has | '' | empty required field |
| --missing | not_a_real_field_zzz | unknown missing field |
| --missing | revel:* | malformed missing field |
| --missing | '' | empty missing field |

```bash run id=invalid-variant-filter exit=2 each_row="Variant filter vocabularies"
biomcp --json search variant --gene BRAF {{flag}} {{value}} --limit 1
```

```json expect=invalid-variant-filter contains each_row="Variant filter vocabularies"
{
  "error": {
    "code": "invalid_argument"
  }
}
```

## Captured MyVariant Filters and Consequences

The routine fixture replays recorded MyVariant responses while preserving the
real CLI-to-source request path. A BRAF consequence filter and a field-presence
filter must each return decoded BRAF rows instead of reporting a successful
empty result when their provider query is accepted. The consequence receipt has
only independently ordered dbNSFP protein arrays, so those rows must omit
`hgvs_p` instead of inventing a transcript pairing.

```bash
biomcp --json --no-cache search variant -g BRAF --consequence missense_variant --limit 3 \
  | jq '(.results | length > 0) and all(.results[]; (.gene == "BRAF") and (.hgvs_p == null))' \
  | mustmatch 'true'
```

```bash
biomcp --json --no-cache search variant -g BRAF --has revel --limit 3 \
  | jq 'any(.results[]?; (.gene == "BRAF") and (.revel | type == "number"))' \
  | mustmatch 'true'
```

## Captured GRCh38 MyVariant lookup

Use `--assembly hg38` when supplying a GRCh38 coordinate. The command sends
that declared build to MyVariant and keeps the answering coordinate space in
its public provenance. The routine fixture replays the recorded BRAF V600E
response only for that request.

```bash
biomcp --json --no-cache get variant --assembly hg38 'chr7:g.140753336A>T' \
  | jq '{gene, id, genome_build}' \
  | mustmatch like '{"gene":"BRAF","id":"chr7:g.140753336A>T","genome_build":"GRCh38"}'
```

## One coordinate build per variant card

When population lookup resolves an rsID's GRCh37 source identity to a GRCh38
coordinate, the card uses that displayed GRCh38 spelling in every variant
follow-up command. It does not silently switch command coordinates back to the
source identity. The printed coordinate remains a fetchable `get variant`
input.

```bash run id=population-card-coordinate-consistency
card="$(biomcp --no-cache get variant rs334 population)"
commands="$(printf '%s\n' "$card" | grep -E 'biomcp (get variant|variant (trials|articles|oncokb))')"
printf '%s\n' "$card" | grep -Fq 'Resolved GRCh38 coordinate: chr11:g.5227002T>A (dbSNP)'
test -n "$commands"
while IFS= read -r command; do
  grep -Fq 'chr11:g.5227002T>A' <<<"$command"
  ! grep -Fq 'chr11:g.5248232T>A' <<<"$command"
done <<<"$commands"
printf 'all variant follow-ups use the displayed GRCh38 coordinate\n'
```

```text expect=population-card-coordinate-consistency
all variant follow-ups use the displayed GRCh38 coordinate
```

```bash
biomcp --json --no-cache get variant 'GRCh38:chr11:g.5227002T>A' \
  | jq '{id, rsid, genome_build}' \
  | mustmatch like '{"id":"chr11:g.5227002T>A","rsid":"rs334","genome_build":"GRCh38"}'
```

## Inferred genome builds for genomic variants

Genome-qualified coordinates do not require a separate `--assembly` flag. BioMCP
accepts the documented build aliases and versioned RefSeq forms, rewrites them to
the chromosome HGVS identifier MyVariant accepts, and reports the build that
answered. The routine fixture replays captured GRCh37 BRAF and GRCh38 PTEN
responses, so this remains a deterministic contract rather than a live-provider
canary.

### Explicit build prefixes

| str:input | str:id | str:rsid | str:build | str:label |
|---|---|---|---|---|
| GRCh37:chr7:g.140453136A>T | chr7:g.140453136A>T | rs113488022 | GRCh37 | GRCh37 prefix |
| hg19:chr7:g.140453136A>T | chr7:g.140453136A>T | rs113488022 | GRCh37 | hg19 prefix |
| GRCh38:chr7:g.140753336A>T | chr7:g.140753336A>T | rs113488022 | GRCh38 | GRCh38 prefix |
| hg38:chr7:g.140753336A>T | chr7:g.140753336A>T | rs113488022 | GRCh38 | hg38 prefix |

```bash each_row="Explicit build prefixes"
biomcp --json --no-cache get variant '{{input}}' \
  | jq '{id, rsid, genome_build}' \
  | mustmatch like '{"id":"{{id}}","rsid":"{{rsid}}","genome_build":"{{build}}"}'
```

Versioned RefSeq accessions identify their build from the accession version.
VCF-like and SPDI forms name the same single-nucleotide substitution. A
versioned RefSeq deletion remains an accepted deletion spelling and is rewritten
to the captured provider identity.

### Versioned and alternate coordinate grammars

| str:input | str:id | str:gene | str:label |
|---|---|---|---|
| NC_000010.11:g.87925512G>A | chr10:g.87925512G>A | PTEN | versioned RefSeq HGVS |
| chr10:87925512:G:A | chr10:g.87925512G>A | PTEN | VCF-like |
| NC_000010.11:87925511:G:A | chr10:g.87925512G>A | PTEN | SPDI |
| NC_000010.11:g.87925512del | chr10:g.87925512del | PTEN | versioned RefSeq deletion |

```bash each_row="Versioned and alternate coordinate grammars"
biomcp --json --no-cache get variant '{{input}}' \
  | jq '{id, gene, genome_build}' \
  | mustmatch like '{"id":"{{id}}","gene":"{{gene}}","genome_build":"GRCh38"}'
```

### Bare-coordinate build disambiguation

A bare coordinate is probed in both builds only when needed. A GRCh38-only
coordinate resolves as GRCh38. When both builds have different records,
BioMCP prefers GRCh38 and makes the competing GRCh37 identity explicit instead
of silently discarding it.

```bash
biomcp --json --no-cache get variant 'chr7:g.140753336A>T' \
  | jq '{id, rsid, genome_build}' \
  | mustmatch like '{"id":"chr7:g.140753336A>T","rsid":"rs113488022","genome_build":"GRCh38"}'
```

```bash
biomcp --json --no-cache get variant 'chr10:g.87933119A>C' \
  | jq '{id, rsid, genome_build, build_ambiguous, build_candidates: [.build_candidates[]? | {genome_build, id, rsid}] | sort_by(.genome_build)}' \
  | mustmatch like '{"id":"chr10:g.87933119A>C","rsid":"rs759485888","genome_build":"GRCh38","build_ambiguous":true,"build_candidates":[{"genome_build":"GRCh37","id":"chr10:g.87933119A>C","rsid":"rs1212585646"}]}'
```

## Documented coordinate grammar

The user guide lists the build-qualified and alternate genomic forms accepted by
`get variant`, so callers can choose a known build instead of relying on a bare
coordinate.

```bash
grep -E 'GRCh38:chr|NC_000010\.11:g\.|chr10:87925512:G:A|NC_000010\.11:87925511:G:A|NC_000010\.11:g\.87925512del' ../../docs/user-guide/variant.md \
  | mustmatch like 'GRCh38:chr
NC_000010.11:g.
chr10:87925512:G:A
NC_000010.11:87925511:G:A
NC_000010.11:g.87925512del'
```

## Genomic indel round trips

A variant card resolved from an rsID keeps MyVariant.info's repeat-notation
identity in every variant follow-up. The same printed identity is accepted as a
direct lookup and resolves back to that source record.

```bash run id=indel-card-follow-ups
card="$(biomcp --no-cache get variant rs876657378)"
for command in \
  'get variant "chr19:g.11106928AAG[1]" civic' \
  'biomcp get variant "chr19:g.11106928AAG[1]" all' \
  'biomcp variant trials "chr19:g.11106928AAG[1]"' \
  'biomcp variant articles "chr19:g.11106928AAG[1]"'
do
  printf '%s\n' "$card" | grep -Fq "$command"
done
printf 'all indel follow-ups preserve the fetchable ID\n'
```

```text expect=indel-card-follow-ups
all indel follow-ups preserve the fetchable ID
```

```bash
biomcp --json --no-cache get variant 'chr19:g.11106928AAG[1]' \
  | jq '{id, rsid}' \
  | mustmatch like '{"id":"chr19:g.11106928AAG[1]","rsid":"rs876657378"}'
```

## Indel grammar surfaces agree

The CLI help, invalid-input recovery text, and user guide advertise the same
representative genomic indel forms accepted by the exact-ID parser.

```bash run id=indel-grammar-surfaces
help="$(biomcp get variant --help)"
error="$(biomcp get variant not-a-variant 2>&1 || true)"
guide="$(cat ../../docs/user-guide/variant.md)"
for surface in help error guide; do
  text="${!surface}"
  missing=false
  for form in \
    'repeat' \
    'range deletion' \
    'sequence-qualified deletion' \
    'duplication' \
    'insertion' \
    'inversion' \
    'delins'
  do
    grep -Fqi "$form" <<<"$text" || missing=true
  done
  if $missing; then
    printf '%s is missing genomic indel forms\n' "$surface"
  else
    printf '%s lists genomic indel forms\n' "$surface"
  fi
done
```

```text expect=indel-grammar-surfaces
help lists genomic indel forms
error lists genomic indel forms
guide lists genomic indel forms
```

## Captured CancerHotspots Recurrence

The same routine fixture replays observed MyVariant identity searches and
CancerHotspots by-gene rows. The returned card keeps each source-labelled
recurrence and its matched transcript, so callers do not mistake a different
cohort or transcript for the requested amino-acid change.

```bash
biomcp --json --no-cache get variant 'BRAF V600E' all \
  | jq '{source: .cancerhotspots.source, transcript: .cancerhotspots.matched_transcript, position_count: .cancerhotspots.position_count, same_aa_count: .cancerhotspots.same_aa_count}' \
  | mustmatch like '{"source":"cancerhotspots.org","transcript":"ENST00000288602","position_count":897,"same_aa_count":833}'
```

```bash
biomcp --json --no-cache get variant 'MYD88 L265P' all \
  | jq '{source: .cancerhotspots.source, transcript: .cancerhotspots.matched_transcript, position_count: .cancerhotspots.position_count, same_aa_count: .cancerhotspots.same_aa_count}' \
  | mustmatch like '{"source":"cancerhotspots.org","transcript":"ENST00000396334","position_count":37,"same_aa_count":37}'
```

## Coordinate Genome-Build Context

<!-- mustmatch-lint: skip -->

Variant and gene coordinate strings are source-derived genomic positions, so
consumer-facing output must say which genome build those coordinates use rather
than emitting a bare chromosome/start/end string. The deterministic renderer
contract covers the markdown and JSON envelopes without depending on live
MyVariant or MyGene responses.

## Gene-Scoped Variant Search

Gene-first search should still return the canonical variant identity columns and
preserve the BRAF V600E row as a recognizable anchor.

## Search Table Contract

The JSON path should keep the same follow-up shape so agents can pivot into the
default card without scraping markdown helper text.

## Protein-Filter Narrowing

Long-form protein filters should normalize to the same compact spelling that the
short-form query uses, rather than leaking a second variant identifier shape.

## Variant identity and filter evaluation

Exact-search identity and per-filter evaluation answer different questions.
`resolved`, `ambiguous`, and `unresolved` describe only whether the requested
variant identifies one compatible provider record. Every submitted filter
separately reports whether BioMCP `evaluated` it independently
of whether the combined query returned rows. The same per-filter outcome is
available in JSON and Markdown: a gene unavailable under its symbol or aliases
is not presented as a true negative, while a recognized gene and a valid
protein filter can truthfully produce an empty intersection. A successfully
retried gene alias and a non-identity threshold filter use the same channel.

```bash
unresolved_json="$(biomcp --json search variant -g NOTAREALGENE1091 --limit 1)"
unresolved_markdown="$(biomcp search variant -g NOTAREALGENE1091 --limit 1)"
alias_json="$(biomcp --json search variant -g H3-3A --limit 1)"
resolved_json="$(biomcp --json search variant -g RB1 --hgvsp Q999X --limit 1)"
resolved_markdown="$(biomcp search variant -g RB1 --hgvsp Q999X --limit 1)"
threshold_json="$(biomcp --json search variant -g H3F3A --min-cadd 99 --limit 1)"

jq -cn \
  --argjson unresolved "$unresolved_json" \
  --argjson alias "$alias_json" \
  --argjson resolved "$resolved_json" \
  --argjson threshold "$threshold_json" \
  --arg unresolved_markdown "$unresolved_markdown" \
  --arg resolved_markdown "$resolved_markdown" \
  'def outcome($text; $name; $status):
    $text | split("\\n") | any(
      (test($name; "i")) and
      (test("(^|[^[:alpha:]])" + $status + "([^[:alpha:]]|$)"; "i"))
    );
  {
    unresolved_gene_json: ($unresolved.filter_evaluation == {gene: "unavailable"}),
    unresolved_gene_markdown: outcome($unresolved_markdown; "gene"; "unavailable"),
    alias_gene_evaluated: ($alias.filter_evaluation == {gene: "evaluated"}),
    alias_rows_preserved: ($alias.count == 1 and $alias.pagination.total == 1156),
    resolved_empty_count: ($resolved.count == 0),
    exact_identity_unresolved: ($resolved.resolution.status == "unresolved"),
    evaluated_filters_json: ($resolved.filter_evaluation == {gene: "evaluated", hgvsp: "evaluated"}),
    retired_json_absent: ($resolved | has("filter_resolution") | not),
    evaluated_filters_markdown: (
      ($resolved_markdown | contains("Variant identity: unresolved")) and
      ($resolved_markdown | contains("## Filter evaluation")) and
      outcome($resolved_markdown; "gene"; "evaluated") and
      outcome($resolved_markdown; "hgvsp"; "evaluated") and
      ($resolved_markdown | contains("Filter resolution") | not)
    ),
    threshold_uses_common_channel: ($threshold.filter_evaluation == {gene: "evaluated", min_cadd: "evaluated"})
  }' \
  | mustmatch like '{"unresolved_gene_json":true,"unresolved_gene_markdown":true,"alias_gene_evaluated":true,"alias_rows_preserved":true,"resolved_empty_count":true,"exact_identity_unresolved":true,"evaluated_filters_json":true,"retired_json_absent":true,"evaluated_filters_markdown":true,"threshold_uses_common_channel":true}'
```

## Variant filter diagnostics

A current gene symbol that dbNSFP does not index should retry a known MyGene
alias, return the alias-backed rows, and make that substitution explicit in the
JSON envelope.

```bash
current="$(biomcp --json search variant -g H3-3A --limit 1)"
indexed="$(biomcp --json search variant -g H3F3A --limit 1)"
jq -cn --argjson current "$current" --argjson indexed "$indexed" \
  '{same_rows: ($current.results == $indexed.results), total: $current.pagination.total, diagnostics: $current.diagnostics}' \
  | mustmatch like '{"same_rows":true,"total":1156,"diagnostics":["gene H3-3A matched no dbNSFP records; retried as H3F3A and matched 1156"]}'
```

An already indexed gene must keep its direct result without claiming that an
alias retry happened.

```bash
biomcp --json search variant -g H3F3A --limit 1 \
  | jq -c '{count, total: .pagination.total, diagnostics, filter_evaluation, exact_identity_absent: ((has("requested_variant") or has("resolution")) | not)}' \
  | mustmatch like '{"count":1,"total":1156,"diagnostics":[],"filter_evaluation":{"gene":"evaluated"},"exact_identity_absent":true}'
```

If neither a symbol nor any known alias has dbNSFP rows, Markdown should explain
that the zero is a checked absence and render the common diagnostics section.

```bash
biomcp search variant -g NOTAREALGENE1091 --limit 1 \
  | grep -E '^## Filter diagnostics$|^gene NOTAREALGENE1091 matched no dbNSFP records under any known symbol or alias$' \
  | mustmatch like '## Filter diagnostics
gene NOTAREALGENE1091 matched no dbNSFP records under any known symbol or alias'
```

A parseable protein change that has no exact row may report bounded positions
for the same residue pair, but it must not silently renumber the request.

```bash
biomcp --json search variant -g H3F3A --hgvsp K27M --limit 1 \
  | jq -c '{count, status: .resolution.status, diagnostics}' \
  | mustmatch like '{"count":0,"status":"unresolved","diagnostics":["no dbNSFP record for H3F3A p.K27M; dbNSFP holds K to M at positions 19, 28, 37, 57"]}'
```

The indexed coordinate remains resolved and does not run or report the
position probe.

```bash
biomcp --json search variant -g H3F3A --hgvsp K28M --limit 1 \
  | jq -c '{count, status: .resolution.status, filter_evaluation, diagnostics}' \
  | mustmatch like '{"count":1,"status":"resolved","filter_evaluation":{"gene":"evaluated","hgvsp":"evaluated"},"diagnostics":[]}'
```

When each filter has provider rows but their intersection does not, the empty
result should be reported as an applied-filter answer rather than a broken
lookup.

```bash
biomcp --json search variant -g H3F3A --min-cadd 99 --limit 1 \
  | jq -c '{count, diagnostics}' \
  | mustmatch like '{"count":0,"diagnostics":["filters applied; no record matched"]}'
```

## Strict exact variant identity

Exact protein search keeps the supplied identity separate from its normalized
alias and checks the source's returned identity before including a row. Here the
healthy fixture offers only BRCA1 residue 16, so a request for residue 1783 is
explicitly unresolved instead of being relabeled as a match.

```bash
biomcp --json search variant -g BRCA1 --hgvsp p.Met1783Ile --limit 5 \
  | jq '{requested_gene: .requested_variant.gene, supplied_protein: .requested_variant.protein_change, normalized_proteins: .resolution.normalized_aliases.protein_changes, status: .resolution.status, exhaustive: .resolution.exhaustive, retained: (.results | length), filtered_total: .pagination.total, has_more: .pagination.has_more}' \
  | mustmatch like '{"requested_gene":"BRCA1","supplied_protein":"p.Met1783Ile","normalized_proteins":["M1783I"],"status":"unresolved","exhaustive":true,"retained":0,"filtered_total":0,"has_more":false}'
```

The same source response does contain residue 16. Asking for that identity keeps
its source row and records the source alias that proved the match, rather than
dropping every exact result indiscriminately.

```bash
biomcp --json search variant -g BRCA1 --hgvsp p.Met16Ile --limit 5 \
  | jq '{supplied_protein: .requested_variant.protein_change, normalized_proteins: .resolution.normalized_aliases.protein_changes, status: .resolution.status, exhaustive: .resolution.exhaustive, retained: (.results | length), matched_alias: .results[0].matched_alias, source_has_supplied_alias: (.results[0].source_identity.protein_changes | index("p.Met16Ile") != null), source_has_short_alias: (.results[0].source_identity.protein_changes | index("p.M16I") != null), filtered_total: .pagination.total, has_more: .pagination.has_more}' \
  | mustmatch like '{"supplied_protein":"p.Met16Ile","normalized_proteins":["M16I"],"status":"resolved","exhaustive":true,"retained":1,"matched_alias":"p.Met16Ile","source_has_supplied_alias":true,"source_has_short_alias":true,"filtered_total":1,"has_more":false}'
```

An exact match can be retained by dbNSFP while the displayed SnpEff tuple uses
another transcript. The structured explanation preserves whole provider tuples
and assigns roles without zipping the independent dbNSFP arrays.

```bash
biomcp --json search variant -g HSD17B4 --hgvsp H540R --limit 10 \
  | jq -c '.results[0] | {id, matched_alias, transcript, hgvs_c, hgvs_p, transcript_annotations_complete, transcript_annotations}' \
  | mustmatch like '{"id":"chr5:g.118860951A>G","matched_alias":"p.His540Arg","transcript":"NM_000414.3","hgvs_c":"c.1544A>G","hgvs_p":"p.His515Arg","transcript_annotations_complete":true,"transcript_annotations":[{"source":"myvariant.info/snpeff.ann","gene":"HSD17B4","transcript":"NM_000414.3","hgvs_c":"c.1544A>G","hgvs_p":"p.His515Arg","roles":["displayed"]},{"source":"myvariant.info/snpeff.ann","gene":"HSD17B4","transcript":"NM_001199291.2","hgvs_c":"c.1619A>G","hgvs_p":"p.His540Arg","roles":["matched"]},{"source":"myvariant.info/snpeff.ann","gene":"HSD17B4","transcript":"XM_011512026.2","hgvs_c":"c.1616A>G","hgvs_p":"p.His539Arg","roles":[]},{"source":"myvariant.info/snpeff.ann","gene":"HSD17B4","transcript":"NM_001199292.2","hgvs_c":"c.1562A>G","hgvs_p":"p.His521Arg","roles":[]},{"source":"myvariant.info/snpeff.ann","gene":"HSD17B4","transcript":"XM_017009363.1","hgvs_c":"c.1544A>G","hgvs_p":"p.His515Arg","roles":[]}]}'
```

Markdown leaves the result table intact and adds the bounded explanation only
after that table.

```bash
biomcp search variant -g HSD17B4 --hgvsp H540R --limit 10 \
  | mustmatch like '| chr5:g.118860951A>G | GRCh37 | HSD17B4 | NM_000414.3 | c.1544A>G | p.His515Arg | ...
...
## Transcript match explanations
...
- `chr5:g.118860951A>G`: matched `NM_001199291.2 | c.1619A>G | p.His540Arg` from a different source-provided transcript annotation; displayed `NM_000414.3 | c.1544A>G | p.His515Arg`.
...
Use `get variant <id>` for details.'
```

When the displayed tuple itself matches, one object receives both ordered roles
and Markdown adds no explanation. A broad search stays compact.

```bash
same_json="$(biomcp --json search variant -g HSD17B4 --hgvsp H515R --limit 10)"
same_markdown="$(biomcp search variant -g HSD17B4 --hgvsp H515R --limit 10)"
broad_json="$(biomcp --json search variant -g HSD17B4 --limit 1)"
jq -n --argjson same "$same_json" --arg same_markdown "$same_markdown" --argjson broad "$broad_json" \
  '{same_roles: $same.results[0].transcript_annotations[0].roles, note_absent: ($same_markdown | contains("Transcript match explanations") | not), broad_omits: (($broad.results[0] | has("transcript_annotations") or has("transcript_annotations_complete")) | not)}' \
  | mustmatch like '{"same_roles":["displayed","matched"],"note_absent":true,"broad_omits":true}'
```

## Residue-Alias Search

Residue aliases should stay on the typed variant path instead of falling
through to free-text or disease-style fallback behavior.

## Clinical Significance

ClinVar remains an opt-in deepen path. The section should keep the human heading
and a compact JSON disease anchor without bloating the default card.

## Population Frequency

Population frequency also stays opt-in. The markdown and JSON views should keep
the same compact gnomAD frequency story.

## Variant Follow-Ups

The default card should still advertise typed follow-ups for downstream trial
and article pivots even when those surfaces are covered elsewhere.

## Structure Helper Discoverability

The structure helper is an opt-in variant pivot for residue, domain, PDB,
AlphaFold, and hotspot context. It should be visible in help and structured
command listings before users try a live source join.

```bash
../../tools/biomcp-ci variant structure --help | mustmatch like 'biomcp variant structure "BRAF V600E"
residue
domain
PDB
AlphaFold
Cancerhotspots'
```

```bash
../../tools/biomcp-ci --json list variant \
  | jq -r '.entries[] | select(.kind == "template") | .template' \
  | mustmatch like 'variant structure <variant>'
```

## Variant Structure Blog Walkthrough

The public blog should teach the shipped variant-structure workflow as a real
BRAF V600E command sequence, link readers to the reference how-to, and be wired
into the MkDocs Blog nav.

```bash
grep -h -F \
  -e 'blog/variant-structure-in-commands.md' \
  -e '**TL;DR:**' \
  -e 'biomcp get variant "BRAF V600E"' \
  -e 'biomcp variant structure "BRAF V600E"' \
  -e '../how-to/annotate-variant-structure.md' \
  -e 'InterPro' \
  -e 'AlphaFold' \
  -e 'Cancerhotspots' \
  -e 'biomcp variant articles "BRAF V600E"' \
  -e '## Try it' \
  ../../mkdocs.yml ../../docs/blog/variant-structure-in-commands.md | mustmatch like 'blog/variant-structure-in-commands.md
**TL;DR:**
biomcp get variant "BRAF V600E"
biomcp variant structure "BRAF V600E"
../how-to/annotate-variant-structure.md
InterPro
AlphaFold
Cancerhotspots
biomcp variant articles "BRAF V600E"
## Try it'
```

## Variant Article Entity Recall

The default union remains honest when strict resolution finds no allele and
BioMCP uses labeled best-effort text. This healthy fixture serves the MYD88
paper only for the non-exact fallback path; exact annotation behavior is shown
with the diagnostic strategy below.

```bash
bash ../fixtures/run-variant-article-entity-fixture.sh ../.. myd88 | mustmatch like '## MYD88 S219C fallback
best-effort free-text fallback
24534189'
```

```bash
bash ../fixtures/run-variant-article-entity-fixture.sh ../.. myd88-json | mustmatch like 'JSON fallback path preserved
24534189'
```

## Variant Article Routes Are Unioned Before Pagination

<!-- mustmatch-lint: skip -->

The default literature strategy preserves every compatible annotation entity,
resolved protein/coding/genomic alias, and source-backed citation before it
deduplicates, ranks, and applies the public limit. A paper reached by two routes
remains one row with associated provenance, and offset pages retain global rank.

```bash run id=variant-article-union exit=0
bash ../fixtures/run-variant-article-entity-fixture.sh ../.. union-json
```

```json expect=variant-article-union contains
{
  "strategy": "union",
  "requested_gene": "BRAF",
  "supplied_protein": "p.V600E",
  "resolution": "resolved",
  "complete": true,
  "pmids": ["6010001", "6010002", "6010003", "6010004", "6010005", "6010006", "6010007", "6010008"],
  "all_rows_keep_requested_variant": true,
  "alias_matches": [
    {"pmid": "6010002", "matched_aliases": ["BRAF p.Val600Glu"]},
    {"pmid": "6010005", "matched_aliases": ["BRAF c.1799T>A"]},
    {"pmid": "6010006", "matched_aliases": ["chr7:g.140453136A>T"]}
  ],
  "shared_provenance": [
    {"route": "exact_lexical", "source": "pubtator", "matched_alias": "BRAF p.V600E"},
    {"route": "pubtator_variant", "source": "pubtator", "matched_alias": "BRAF p.V600E"}
  ],
  "citation_provenance": [
    {"route": "source_citation", "source": "civic", "matched_alias": "BRAF p.V600E"}
  ],
  "pubmed_provenance": [
    {"route": "exact_lexical", "source": "pubmed", "matched_alias": "BRAF p.Val600Glu"}
  ],
  "annotation_pmids": ["6010001", "6010003", "6010007"],
  "page_matches_full_slice": true,
  "page_ranks": [3, 4],
  "pagination": {"offset": 2, "limit": 2, "returned": 2, "total": 8, "has_more": true},
  "truncated": true,
  "source_status": [
    {"route": "exact_lexical", "source": "pubmed", "status": "ok"},
    {"route": "pubtator_variant", "source": "pubtator", "status": "ok"},
    {"route": "source_citation", "source": "myvariant", "status": "ok"}
  ]
}
```

## Verified variant-article identity does not promote retrieval aliases

<!-- mustmatch-lint: skip -->

Retrieval aliases explain why a paper was found; they are not observations from
that paper. With identity verification enabled, confirmation requires a provider
relation linking the gene and allele annotations; shared-passage annotations remain
unverified. The local fixture preserves alias-only collisions as unverified,
contradictory, or conflicting, and emits the captured evidence needed to audit a
confirmed result. `--confirmed-only`
filters that verified pool before ranking and pagination, so earlier collisions
cannot hide the confirmed paper.

```bash run id=variant-article-identity-verification exit=0
bash ../fixtures/run-variant-article-entity-fixture.sh ../.. identity-verification-json
```

```json expect=variant-article-identity-verification contains
{
  "normal_statuses": [
    {"pmid": "6010001", "status": "confirmed"},
    {"pmid": "6010002", "status": "unverified"},
    {"pmid": "6010005", "status": "contradictory"},
    {"pmid": "6010006", "status": "conflicting"}
  ],
  "alias_only_candidates_never_confirmed": true,
  "confirmed_observation_is_auditable": true,
  "confirmed_only_keeps_the_confirmed_result": true,
  "confirmed_only_excludes_nonconfirmations": true,
  "debug_plan_records_verification_artifact": true
}
```

## Pagination Limits Metadata Enrichment

<!-- mustmatch-lint: skip -->

Variant-article pagination selects the visible ranked page before optional
metadata enrichment. A small page does not spend source lookups enriching a
source-citation candidate that will not be returned.

```bash run id=variant-article-visible-enrichment exit=0
bash ../fixtures/run-variant-article-entity-fixture.sh ../.. page-enrichment-json
```

```json expect=variant-article-visible-enrichment contains
{
  "visible_pmids": ["6010003"],
  "hidden_candidate_enriched": false
}
```

## Strategy Modes Isolate Diagnostic Routes

<!-- mustmatch-lint: skip -->

Omitting `--strategy` is the dependable union behavior. The annotation and
lexical modes are diagnostic views: each returns only candidates acquired by
that route, while source-backed citations remain part of union.

```bash run id=variant-article-strategies exit=0
bash ../fixtures/run-variant-article-entity-fixture.sh ../.. strategies-json
```

```json expect=variant-article-strategies contains
{
  "omitted_equals_union": true,
  "annotation_pmids": ["6010001", "6010003", "6010007"],
  "lexical_pmids": ["6010002", "6010003", "6010005", "6010006", "6010008"],
  "union_pmids": ["6010001", "6010002", "6010003", "6010004", "6010005", "6010006", "6010007", "6010008"]
}
```

## Unresolved Fallback Does Not Claim Exact Provenance

<!-- mustmatch-lint: skip -->

When strict identity resolution is healthy but finds no resolved allele,
best-effort text can still help discovery. Such a row is explicitly non-exact
and carries no matched exact alias.

```bash run id=variant-article-unresolved exit=0
bash ../fixtures/run-variant-article-entity-fixture.sh ../.. unresolved-json
```

```json expect=variant-article-unresolved contains
{
  "resolution": "unresolved",
  "complete": true,
  "pmid": "24534189",
  "row_requested_gene": "MYD88",
  "routes": ["best_effort_free_text"],
  "matched_aliases": [],
  "has_exact_claim": false
}
```

## Healthy Empty Variant Literature Keeps Its Envelope

<!-- mustmatch-lint: skip -->

A healthy annotation miss is different from a provider failure. JSON keeps the
empty collection, resolution, source status, completeness, and pagination facts
so callers do not have to infer state from missing keys.

```bash run id=variant-article-empty exit=0
bash ../fixtures/run-variant-article-entity-fixture.sh ../.. healthy-empty-json
```

```json expect=variant-article-empty contains
{
  "strategy": "annotation",
  "resolution": "unresolved",
  "results": [],
  "complete": true,
  "truncated": false,
  "pagination": {
    "offset": 0,
    "limit": 3,
    "returned": 0,
    "total": 0,
    "has_more": false,
    "next_page_token": null
  },
  "source_status": [
    {"route": "pubtator_variant", "source": "pubtator", "status": "ok"}
  ]
}
```

## Caller-supplied RefSeq identities remain exact when MyVariant has no record

<!-- mustmatch-lint: skip -->

A complete versioned RefSeq chromosome identity and explicit assembly is an exact
caller assertion even when MyVariant has no matching row. Both decomposed fields
and genomic HGVS canonicalize to the same public identity shape; exact literature
routes retain the literal transcript, coding, and genomic aliases without
inventing chromosome coordinates or provider confirmation.

```bash run id=variant-article-refseq-not-found exit=0
bash ../fixtures/run-variant-article-entity-fixture.sh ../.. refseq-not-found-json
```

```json expect=variant-article-refseq-not-found contains
{
  "items": [
    {
      "request_id": "atm-grch38",
      "requested_variant": {
        "gene": "ATM",
        "transcript": "NM_000051.4",
        "coding_change": "c.1066-6T>G",
        "genomic_accession": "NC_000011.10",
        "genome_build": "GRCh38",
        "position": 108248927,
        "reference": "T",
        "alternate": "G"
      },
      "resolution": {
        "status": "resolved",
        "basis": "caller_supplied",
        "exhaustive": true,
        "provider_validation": {
          "source": "myvariant",
          "status": "not_found",
          "matched_alias": null,
          "contradictory_field": null
        }
      },
      "complete": true,
      "truncated": false,
      "source_citation": {
        "status": "skipped",
        "detail": "no compatible MyVariant record"
      },
      "literal_exact_aliases": [
        "ATM c.1066-6T>G",
        "NC_000011.10:g.108248927T>G",
        "NM_000051.4:c.1066-6T>G"
      ],
      "only_literal_exact_aliases": true,
      "literal_exact_route_queries": [
        "ATM c.1066-6T>G",
        "NC_000011.10:g.108248927T>G",
        "NM_000051.4:c.1066-6T>G"
      ],
      "only_literal_route_queries": true,
      "literal_route_source_provenance": true
    },
    {
      "request_id": "palb2-grch38",
      "requested_variant": {
        "gene": "PALB2",
        "transcript": "NM_024675.4",
        "coding_change": "c.3350+5G>A",
        "genomic_accession": "NC_000016.10",
        "genome_build": "GRCh38",
        "position": 23607859,
        "reference": "C",
        "alternate": "T"
      },
      "resolution": {
        "status": "resolved",
        "basis": "caller_supplied",
        "exhaustive": true,
        "provider_validation": {
          "source": "myvariant",
          "status": "not_found",
          "matched_alias": null,
          "contradictory_field": null
        }
      },
      "complete": true,
      "truncated": false,
      "source_citation": {
        "status": "skipped",
        "detail": "no compatible MyVariant record"
      },
      "literal_exact_aliases": [
        "NC_000016.10:g.23607859C>T",
        "NM_024675.4:c.3350+5G>A",
        "PALB2 c.3350+5G>A"
      ],
      "only_literal_exact_aliases": true,
      "literal_exact_route_queries": [
        "NC_000016.10:g.23607859C>T",
        "NM_024675.4:c.3350+5G>A",
        "PALB2 c.3350+5G>A"
      ],
      "only_literal_route_queries": true,
      "literal_route_source_provenance": true
    }
  ],
  "encoding_equivalence": {
    "same_requested_variant": true,
    "expected_normalized_aliases": true,
    "same_normalized_aliases": true,
    "same_route_queries": true,
    "same_public_behavior": true
  }
}
```

## Batch Variant Literature Is Ordered and Compact

<!-- mustmatch-lint: skip -->

A structured input file replaces caller-authored alias query matrices when several
exact variants need literature triage. The response keeps request order and each
sibling's resolution state while returning shortlist facts rather than hydrated
article cards. Its next commands can be parsed directly for article triage and
detail retrieval.

```bash run id=variant-article-batch exit=0
bash ../fixtures/run-variant-article-entity-fixture.sh ../.. batch-compact-json
```

```json expect=variant-article-batch contains
{
  "request_ids": ["braf-v600e", "myd88-s219c"],
  "requested_genes": ["BRAF", "MYD88"],
  "sibling_arrays_retained": true,
  "resolutions": [
    {"request_id": "braf-v600e", "status": "resolved"},
    {"request_id": "myd88-s219c", "status": "unresolved"}
  ],
  "match_reasons": {
    "braf_all_exact": true,
    "myd88_all_best_effort": true
  },
  "route_claims": {
    "braf_has_exact": true,
    "myd88_only_fallback": true
  },
  "aggregate": {"complete": true, "truncated": true},
  "item_state_present": true,
  "compact_rows": true,
  "followups": {
    "parseable": true,
    "article_batch": true,
    "article_detail": true,
    "fulltext": true,
    "assets": true,
    "citations": true
  }
}
```

## Variant Article Route Plans Are Opt In

<!-- mustmatch-lint: skip -->

Request a route plan only in JSON when aliases, provider work, ranking, or a
truncated acquisition needs explanation. Ordinary output stays compact. A single
request and every item in a batch expose the same typed route facts, while the
batch adds its fixed item-worker and request-budget summary. These diagnostic
facts never expose transport URLs, request or response bodies, paths, headers,
or credentials.

```bash run id=variant-article-plan exit=0
bash ../fixtures/run-variant-article-entity-fixture.sh ../.. debug-plan-json
```

```json expect=variant-article-plan contains
{
  "ordinary_omits_plan": {"single": true, "batch": true},
  "transport_fields_redacted": true,
  "single": {
    "aliases_present": true,
    "required_routes": {
      "annotation": true,
      "lexical": true,
      "source_citation": true
    },
    "shape_complete": true
  },
  "batch": {
    "item_concurrency_limit": 2,
    "items_planned": 2,
    "request_budget_consistent": true,
    "every_item_has_plan": true
  }
}
```

## ID Normalization

Exact variant lookup should normalize equivalent identifiers back to the same
canonical record instead of splitting the user into parallel identities.

## Transcript HGVS Normalization Proxies

Transcript HGVS strings are not exact MyVariant IDs, but agents often already
have a source-shaped transcript candidate from a report or another database. The
normalization proxy keeps that input separate from each upstream service's
returned notation and warnings.

## ERBB2 Transcript HGVS Canary

The proxy must handle transcript strings with substitution notation and shell
metacharacters such as `>` without losing source warnings or conflating service
outputs.

## Unsupported Normalization Inputs

BioMCP should not guess transcripts or convert gene-protein shorthand into a
transcript HGVS query. Unsupported input gets a typed guardrail so an agent can
choose a better source-shaped string.

```bash
set +e
out="$(../../tools/biomcp-ci --json variant normalize all 'BRAF V600E' 2>&1)"
rc=$?
set -e
test "$rc" -ne 0
mustmatch like 'unsupported_notation
BRAF V600E
transcript HGVS' <<<"$out"
```

## Normalization Command Discoverability

The explicit proxy command should be visible from help and structured list
output so agents can find it without trying hidden `get variant` rewrites.

```bash
../../tools/biomcp-ci variant normalize --help | mustmatch like 'all, mutalyzer, or variantvalidator
NM_000248.3:c.135del'
../../tools/biomcp-ci --json list variant \
  | jq -e '.entries | any(.kind == "template" and .template == "variant normalize <service> <transcript_hgvs>")' >/dev/null
```

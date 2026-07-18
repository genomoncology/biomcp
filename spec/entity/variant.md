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
../../tools/biomcp-ci --json list variant | jq -r '.commands[]' | mustmatch like 'variant structure <variant>'
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

Exact variant article pivots should use PubTator's normalized variant entity
when one can be selected confidently, then stay honest when PubTator has no
abstract-level variant annotation and BioMCP must fall back to free text. This
fixture serves the BRAF V600E article only for the `@VARIANT_...` query and
serves the MYD88 S219C article only for the labeled best-effort fallback path.

```bash
bash ../fixtures/run-variant-article-entity-fixture.sh ../.. braf | mustmatch like '## BRAF V600E limit 1
PubTator variant annotation recall
4260001
## BRAF V600E limit 3
PubTator variant annotation recall
4260001'
```

```bash
bash ../fixtures/run-variant-article-entity-fixture.sh ../.. myd88 | mustmatch like '## MYD88 S219C fallback
best-effort free-text fallback
24534189'
```

```bash
bash ../fixtures/run-variant-article-entity-fixture.sh ../.. myd88-json | mustmatch like 'JSON fallback path preserved
24534189'
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
../../tools/biomcp-ci --json list variant | jq -e '.commands | any(. == "variant normalize <service> <transcript_hgvs>")' >/dev/null
```

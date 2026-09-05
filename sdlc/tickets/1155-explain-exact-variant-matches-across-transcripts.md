---
flow: build
priority: 8
deps: [1154]
---

# Explain exact variant matches across transcripts

## Goal

An exact variant search explains why a retained result matched when its displayed
transcript uses different HGVS descriptions. On 2026-09-04,
`biomcp search variant -g HSD17B4 --hgvsp H540R --limit 10` retained
`chr5:g.118860951A>G` because `p.His540Arg` matched, but displayed
`NM_000414.3`, `c.1544A>G`, and `p.His515Arg`. The same MyVariant.info hit has
five `snpeff.ann` objects, including the intact matching tuple
`NM_001199291.2` / `c.1619A>G` / `p.His540Arg` and the intact displayed tuple
`NM_000414.3` / `c.1544A>G` / `p.His515Arg`. BioMCP currently keeps only the
selected tuple and flattens the other identity facts, so neither a person nor
an agent can establish which transcript owns the matched descriptions.

The captured reproduction and provider analysis are in
`sdlc/issues/feature-explain-exact-variant-matches-with-paired-transcript-annotations.md`
at commit `84f2343f`. Current code already deserializes each `snpeff.ann` object
as one `MyVariantSnpeffAnnotation`; the relationship is lost later, in search
result transformation and rendering.

## Public contract

For every retained **exact-search** result, JSON adds these two fields:

- `transcript_annotations_complete`: a required boolean. It is `true` only
  when the complete bounded SnpEff annotation set for the hit was preserved.
- `transcript_annotations`: a required array of zero to 32 objects. Every
  object has exactly these keys, in this serialized order:
  `source`, `gene`, `transcript`, `hgvs_c`, `hgvs_p`, and `roles`.

`source` is the non-null literal `myvariant.info/snpeff.ann`. Each of `gene`,
`transcript`, `hgvs_c`, and `hgvs_p` is a trimmed non-empty string or JSON
`null`; missing provider members remain null and are never filled from another
annotation, dbNSFP, ClinVar, the request, or another source. `roles` is a
unique array of zero to two values from the closed vocabulary `displayed` and
`matched`, ordered `displayed` before `matched` when both apply.

One output object comes from one and only one original `snpeff.ann` object.
After trimming, byte-for-byte duplicate identity-field tuples are coalesced and
their roles are unioned; annotations that differ in any identity field remain
separate. Output order is deterministic: the annotation with `displayed`
first, annotations with `matched` next, then unmarked annotations, with
original provider order breaking every tie. This reordering affects only the
explanatory array, not result order or the existing displayed columns.

The `displayed` role belongs to the SnpEff object selected by the existing
`select_transcript_annotation` policy. If the displayed tuple came only from
the current ClinVar fallback, no SnpEff object is marked displayed. The
`matched` role is assigned only when fields in that same SnpEff object prove
the submitted exact selector under the existing normalization rules:

- protein exact search: its `hgvs_p` matches the requested protein and its
  `gene`, when a gene was supplied, matches that gene;
- coding exact search: its `hgvs_c` matches the requested coding HGVS and its
  `gene`, when supplied, matches that gene;
- a combined exact request must satisfy every submitted transcript-specific
  selector in that one object; and
- an rsID or genomic-only match never marks a transcript annotation as
  `matched`, because those selectors do not prove a transcript tuple.

A partial annotation can be preserved, but it receives a role only when its
present fields satisfy the complete applicable rule above. A hit with no
`snpeff.ann` has an empty array with `transcript_annotations_complete: true`.
The role is explanatory only: it cannot admit or reject a hit, change
`matched_alias`, change the exact `resolution`, or imply transcript
equivalence, preference, renumbering, or clinical significance.

Broad searches remain compact and omit both new fields. CLI JSON and raw-MCP
JSON are identical. The typed `search` MCP tool returns the same JSON result
objects because its variant branch executes the same search path; no tool input
schema changes.

Markdown preserves the existing result table. For an exact result only, it
adds one bounded note immediately after that row when a complete annotation
set has distinct displayed and matched objects. The note names the result ID,
the matched tuple, and the displayed tuple and says that the match came from a
different source-provided transcript annotation. It does not say that either
transcript is preferred or that residues were renumbered. There is no note
when one object has both roles, either role is absent, the two tuples are
identical, or annotation preservation is incomplete. Raw and typed MCP
Markdown match CLI Markdown exactly.

## Resource and failure bounds

The existing shared MyVariant response-body ceiling remains 8 MiB. In
addition, annotation shaping applies all of these limits before allocating
public annotation strings:

- inspect no more than 32 `snpeff.ann` objects per hit;
- accept at most 256 UTF-8 bytes for each non-null identity field after
  trimming; and
- accept at most 256 KiB of those identity-field bytes cumulatively across the
  retained results in one returned page (roles and fixed source labels are not
  charged).

The bounds are fail-safe, not truncating. If one hit has a 33rd annotation, one
field is 257 bytes, or one annotation identity field has an unsupported JSON
type, only that result receives an empty array and
`transcript_annotations_complete: false`. The additive SnpEff decoder must
isolate that malformed field rather than discard an otherwise usable hit. If
the sum of otherwise valid hit projections would exceed 256 KiB, every result
in that returned page receives an empty array and `false`; no provider-order
prefix is exposed as though it were complete. In either case BioMCP keeps the
already validated core result and its existing identity and provenance fields,
but emits no alternate-transcript Markdown explanation for an incomplete
result. It must not emit a partial array, clip a field, zip independent arrays,
panic, or convert over-cap content into trusted match evidence. Existing
provider parse failures outside this additive projection retain their current
outcome.

Unit tests pin 32/33 annotations, 256/257-byte fields, and 256 KiB / 256 KiB
plus one byte across a page. They also prove that cap accounting is checked
before cloning provider strings and that an over-cap page remains bounded and
explanation-free.

## Completion evidence

- A deterministic captured MyVariant fixture contains all five HSD17B4
  `snpeff.ann` rows in their recorded order. Exact protein search marks
  `NM_001199291.2` / `c.1619A>G` / `p.His540Arg` as matched and
  `NM_000414.3` / `c.1544A>G` / `p.His515Arg` as displayed, preserves the
  other three rows as unmarked tuples, and renders the non-equivalence note.
- Synthetic fixtures cover one tuple with both roles, missing transcript,
  missing coding HGVS, missing protein HGVS, exact duplicates, two annotations
  sharing one HGVS value, no SnpEff data, ClinVar-only display fallback,
  protein and coding requests, a combined request that is split across two
  annotations, genomic exact search, and rsID exact search. No case constructs
  a tuple from separate objects.
- The compatibility matrix covers CLI Markdown, CLI JSON, raw MCP Markdown,
  raw MCP JSON, typed-search MCP Markdown, and typed-search MCP JSON. Every
  surface pins positive alternate-transcript output and the same-object/no-note
  case; structured surfaces additionally pin exact keys, nulls, role order,
  annotation order, broad-search omission, and fail-safe over-cap output.
- `docs/user-guide/variant.md`, `skills/use-cases/13-mutation-catalog.md`, and
  `spec/entity/variant.md` explain and exercise the additive exact-search fields
  and their explanatory-only meaning. `skills/schemas/variant.json` describes
  the `get variant` entity rather than the search envelope and therefore does
  not acquire these fields. `docs/reference/source-licensing.md` identifies
  MyVariant.info as the direct carrier and SnpEff as indirect annotation
  provenance; `tests/test_source_licensing_docs_contract.py` pins that
  attribution.
- Existing exact identity fixtures keep their requested identity, source
  identity, `matched_alias`, resolution, filtering, result ordering,
  pagination, and output envelopes. Routine `make lint`, `make test`, and
  `make spec` pass, and the package inventory remains exactly 1,300 files.

## Boundaries and dependencies

Ticket 1154 is a prerequisite and its direct NCBI ClinVar detail path,
current/contributing assertion semantics, and MyVariant fallback precedence do
not change. This ticket changes MyVariant-backed variant **search** projection
and its renderers only; it does not add the new annotation array to `get
variant` or use direct ClinVar data to synthesize it.

This work does not select a clinically preferred transcript, implement MANE
policy, normalize HGVS independently, infer missing fields, reconcile SnpEff
with dbNSFP or ClinVar, change classification, alter exact-match acceptance,
or change ranking, deduplication, pagination, provider queries, cache behavior,
or public command/tool inputs.

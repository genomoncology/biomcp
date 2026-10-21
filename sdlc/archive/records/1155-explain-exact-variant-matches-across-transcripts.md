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
selected SnpEff tuple and discards the other SnpEff tuples. Separately,
`SourceVariantIdentity::from_myvariant_hit` flattens the independent dbNSFP
gene, coding, and protein arrays. The exact matcher derives the current
`matched_alias` from those dbNSFP identity facts; it does not establish a
SnpEff tuple relationship. Neither a person nor an agent can therefore
establish which transcript owns the matched descriptions.

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
The role is explanatory only. Existing dbNSFP evidence can retain a result and
populate `matched_alias` even when no SnpEff object earns `matched`; the new
role must not be inferred merely because `matched_alias` exists. It cannot
admit or reject a hit, change `matched_alias`, change the exact `resolution`,
or imply transcript equivalence, preference, renumbering, or clinical
significance.

Broad searches remain compact and omit both new fields. CLI JSON and raw-MCP
JSON are identical. The typed `search` MCP tool returns the same JSON result
objects because its variant branch executes the same search path; no tool input
schema changes.

Markdown preserves the existing result table byte-for-byte, including its
header, one row per result, and closing blank line. After the complete table
and before `Use get variant <id> for details.`, exact output adds this section
only when at least one result qualifies:

```text
## Transcript match explanations

- `<result-id>`: matched `<transcript-or-> | <hgvs_c-or-> | <hgvs_p-or->` from a different source-provided transcript annotation; displayed `<transcript-or-> | <hgvs_c-or-> | <hgvs_p-or->`.
```

There is at most one bullet per returned result, bullets follow result order,
and the displayed and matched objects are chosen by their role and annotation
order. Thus the section contains at most the returned result limit (50 CLI,
25 typed MCP) and repeats only already bounded annotation values plus the
already emitted result ID. It appears for a result only when its complete
annotation set has distinct displayed and matched objects. It is absent when
one object has both roles, either role is absent, the two tuples are identical,
or annotation preservation is incomplete. The sentence does not say that
either transcript is preferred or that residues were renumbered. Raw and typed
MCP Markdown match CLI Markdown exactly.

The bullet renderer sanitizes the result ID with behavior equivalent to the
existing `sanitize_inline`, then wraps that result in one safe
`markdown_code_span`. It builds each displayed and matched tuple separately:
sanitize each of `transcript`, `hgvs_c`, and `hgvs_p`; substitute the literal
`-` when the field is null or sanitizes to empty; join the three sanitized
values with the authored separator ` | `; then pass that complete joined tuple
once through `markdown_code_span`. The fixed `: matched `, explanatory clause,
`; displayed `, and final period remain authored prose outside the spans.
Newlines, carriage returns, tabs, terminal/control sequences, and bidi controls
in annotation fields therefore cannot create a new line or reorder the new
explanation sentence; embedded backticks cannot close its tuple span; and an
embedded pipe remains literal code-span content. This sanitization contract is
limited to the new explanation section. It neither changes nor makes a new
sanitization claim about the existing table. Structured JSON retains each
trimmed source value with JSON escaping.

## Resource and failure bounds

The existing shared MyVariant response-body ceiling remains 8 MiB. In
addition, annotation shaping applies all of these limits before allocating
public annotation strings:

- project and allocate identity fields for at most 32 `snpeff.ann` objects per
  hit, then observe whether item 33 exists without projecting it;
- accept at most 256 UTF-8 bytes for each non-null identity field after
  trimming; and
- accept at most 256 KiB of those identity-field bytes cumulatively across the
  retained results in one returned page (roles and fixed source labels are not
  charged).

The shared `MyVariantHit` decoder treats absent or JSON-null `snpeff`, and an
object whose `ann` is absent or null, as a valid empty annotation set. It
accepts `ann` as either one annotation object or an array of annotation
objects. A non-object/non-null `snpeff`, a non-object/non-null `ann`, or any
non-object element in an `ann` array makes the complete SnpEff annotation set
malformed. A wrong non-null JSON type in any of the four identity fields does
the same. Valid sibling fields on the hit—including ClinVar and dbNSFP—remain
available; malformed SnpEff alone must not fail deserialization of the hit or
the whole provider response.

For an `ann` array, its deserializer projects the first 32 elements under the
field bounds. It then asks the sequence for item 33 as ignored data. If item 33
exists, it immediately rejects the whole annotation set as incomplete and
drains every remaining array value through streaming ignored-data decoding;
it does not construct an annotation object or any identity string for item 33
or the tail. This still consumes the JSON correctly and preserves valid sibling
fields while keeping retained annotation allocation bounded at 32 objects and
their bounded fields. A non-object among the first 32 is malformed as specified
above; tail values after item 33 need not be validated because the set is
already rejected and cannot become public evidence.

The bounds are fail-safe, not truncating. If one hit has a 33rd annotation, one
field is 257 bytes, or its SnpEff set is malformed under the rules above, only
that result receives an empty array and
`transcript_annotations_complete: false`. If
the sum of otherwise valid hit projections would exceed 256 KiB, every result
in that returned page receives an empty array and `false`; no provider-order
prefix is exposed as though it were complete. In either case BioMCP keeps the
already validated core result and its existing identity and provenance fields,
but emits no alternate-transcript Markdown explanation for an incomplete
result. It must not emit a partial array, clip a field, zip independent arrays,
panic, or convert over-cap content into trusted match evidence. Existing
provider parse failures outside this additive projection retain their current
outcome.

Exact variant search exposes the empty array and `false` and emits no
explanation for malformed or over-cap SnpEff. Broad variant search omits both
additive fields, still returns the otherwise usable hit, and selects its legacy
display tuple from a valid ClinVar sibling when available. `get variant` also
returns the otherwise usable MyVariant-backed card, may use the valid ClinVar
sibling for its existing displayed transcript/coding/protein fields, and does
not add either search-only annotation field or a new section outcome. If no
valid SnpEff or ClinVar tuple exists, broad search and get leave those display
fields absent rather than assembling them from dbNSFP arrays.

Unit tests pin 32/33 annotations, 256/257-byte fields, and 256 KiB / 256 KiB
plus one byte across a page. A separate long-tail array contains 10,000
annotations within the 8 MiB body ceiling and uses decoder
instrumentation to prove that at most 32 annotation objects and their bounded
identity fields are constructed, item 33 triggers incomplete state, and no
tail identity strings are allocated. The tests also prove that cap accounting
is checked before cloning provider strings and that an over-cap page remains
bounded and explanation-free.

## Completion evidence

- A deterministic captured MyVariant fixture contains all five HSD17B4
  `snpeff.ann` rows in their recorded order. Exact protein search marks
  `NM_001199291.2` / `c.1619A>G` / `p.His540Arg` as matched and
  `NM_000414.3` / `c.1544A>G` / `p.His515Arg` as displayed, preserves the
  other three rows as unmarked tuples, and renders exactly
  ``- `chr5:g.118860951A>G`: matched `NM_001199291.2 | c.1619A>G | p.His540Arg` from a different source-provided transcript annotation; displayed `NM_000414.3 | c.1544A>G | p.His515Arg`.``
  under the explanation heading after the complete table.
- Synthetic fixtures cover one tuple with both roles, missing transcript,
  missing coding HGVS, missing protein HGVS, exact duplicates, two annotations
  sharing one HGVS value, no SnpEff data, ClinVar-only display fallback,
  protein and coding requests, a positive combined coding-plus-protein tuple,
  a combined request that is split across two annotations, a requested
  transcript with a missing feature ID, genomic exact search, and rsID exact
  search. A retained complex deletion proves the matched role uses the same
  body-identical protein-HGVS equivalence as exact admission. No case
  constructs a tuple from separate objects.
- Decoder fixtures separately cover absent and null `snpeff`, non-object
  `snpeff`, absent and null `ann`, scalar `ann`, a non-object array element,
  and a wrong-typed identity field. Each malformed case proves exact search's
  empty/false result, broad search's compact usable result, and get's usable
  card with ClinVar fallback; a no-valid-sibling case proves absent display
  fields rather than dbNSFP zipping.
- An adversarial explanation fixture uses a safe ordinary result ID and places
  newlines, carriage returns, tabs, C0/C1 and terminal escape sequences, bidi
  controls, backtick runs, and pipe characters across every displayed and
  matched annotation identity field. The protein-only exact request carries
  the same hostile complex protein value, which lets the matched tuple retain
  a hostile gene without violating the public gene-filter grammar. For CLI,
  raw-MCP, and typed-search
  Markdown, the test extracts only the new section from its heading through
  the line before `Use get variant <id> for details.`. That extracted section
  has exactly one heading and one bullet on one physical line, preserves the
  fixed sentence and tuple-field order, contains no control or bidi character,
  and uses one tuple code-span delimiter longer than every embedded backtick
  run. No injected heading, list item, table row, or trailing prose occurs
  inside the extracted explanation section. A separate baseline assertion
  proves that the complete table prefix is byte-for-byte unchanged; the test
  makes no new sanitization assertion about provider text in that table.
- The compatibility matrix covers CLI Markdown, CLI JSON, raw MCP Markdown,
  raw MCP JSON, typed-search MCP Markdown, and typed-search MCP JSON. Every
  surface pins positive alternate-transcript output and the same-object/no-note
  case; structured process surfaces additionally pin exact keys and nulls,
  role and annotation order, broad-search omission, and hostile identity
  preservation. Rust page contracts pin malformed and over-cap fail-safe output.
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

## Result

Exact MyVariant-backed searches now retain a bounded, fail-safe projection of
whole `snpeff.ann` tuples after the existing dbNSFP identity decision. The
public exact-search rows expose completeness and deterministic annotation
roles, while broad search and `get variant` keep their prior envelopes and the
ticket-1154 ClinVar fallback boundary. Malformed or over-limit SnpEff data is
isolated to empty/incomplete explanation data and cannot admit a result or
change `matched_alias`.

CLI and raw/typed MCP Markdown add the bounded explanation only after the
unchanged result table when distinct complete displayed and matched tuples
exist. The new prose sanitizes every untrusted field and builds one safe code
span per tuple. JSON surfaces share the exact six-key annotation schema.
Documentation and licensing provenance identify MyVariant.info as the direct
carrier and SnpEff as the indirect annotation source. No new package file was
added; the source-size inventory records the exact package-neutral increases.

Focused implementation evidence passed: `cargo fmt --all -- --check`;
`cargo check --tests --no-default-features`; the MyVariant parsing (19), variant
search (17), and variant Markdown (24) Rust test groups; Clippy for the
no-default-feature library and test graph with warnings denied; 15 focused
Python source-licensing, public-skill, and package-boundary contracts; the
complete quality ratchet; the three new CLI mustmatch blocks; and the extended
raw/typed MCP contract block. `cargo package --list --allow-dirty --locked
--offline` remains exactly 1,300 files. The repository-wide `make lint`, `make
test`, and `make spec` gates were intentionally not claimed at this code-stage
handoff.

Independent review rejected the first implementation because it located the
explanation insertion point by searching the rendered body for footer prose.
Remediation passes the explanation through an explicit trusted template slot;
an exact regression places that same footer sentinel in an untrusted table cell
and pins the complete table prefix byte-for-byte before the one explanation
section. The accepted-case matrix now explicitly exercises absent/null SnpEff,
each malformed/33rd/257-byte case through exact projection plus broad/get
sibling fallback, the exact 256 KiB + 1 page boundary, protein/coding/combined
tuple matching and genomic/rsID/missing-field non-matches, and a six-surface
hostile/null fixture. Focused remediation checks passed the five affected Rust
test filters and the changed CLI and raw/typed MCP process blocks. Formatting,
Clippy with warnings denied, the complete quality ratchet, and `git diff
--check` passed; the offline package inventory remains exactly 1,300 files. No
repository-wide gate is claimed by this remediation.

A second independent rereview found that the role matcher used only
single-residue normalization even though exact admission first accepts
body-identical complex protein HGVS. It also found that the prior evidence did
not positively exercise a combined coding-plus-protein tuple, a missing
requested transcript feature, or hostile values in the matched gene and
protein fields. The second remediation centralizes the exact protein
equivalence rule for admission and tuple roles, adds those Rust cases, and
drives the hostile protein request and all four matched tuple fields through
CLI and raw/typed MCP fixtures. The 20-test variant-search group, formatting, Cargo
check, Clippy with warnings denied, the complete quality ratchet, and the
1,300-file offline package inventory pass. The complete routine `make spec`
gate passes with the prepared shared Python environment; `make lint` and
`make test` were not rerun or claimed. Fresh independent rereview remains
pending.

## Review

- Design review: accepted before implementation in this ticket.
- First code review: rejected for unsafe rendered-footer lookup and evidence
  gaps; both findings were remediated with focused passing evidence above.
- Second code rereview: rejected complex-protein matcher drift and three
  remaining evidence gaps; the focused remediation above addresses them.
- Code-stage implementation: complete; another fresh independent rereview
  remains pending.

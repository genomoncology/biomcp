---
flow: build
priority: 29
---

# Make every variant ID biomcp prints one it can read back

GitHub issue #249 (filed 2026-08-25 against 0.8.25 by an external user).
Filed from `sdlc/issues/2026-08-25-get-variant-rejects-indel-ids-it-prints.md`,
which carries the full verified chain.

## What a user hits

`biomcp get variant rs876657378` returns a card for an indel whose ID is
repeat notation — `chr19:g.11106928AAG[1]` — and every follow-up command on
that card (`More:`, `All:`, `See also:`, and the therapeutic-evidence
pointer) is built from that ID. Running any of them fails with
`Unrecognized variant format`. The user is then stranded: the only input
that reaches this variant is the rsID they already used. They tried the
gene+protein shorthand, a genomic range deletion, and the repeat form
biomcp itself printed; all three were rejected.

## The verified chain (confirmed on 0.9.0-dev.5, 2026-08-25)

- The card ID for an rsID lookup is the MyVariant.info document ID
  (`SourceVariantIdentity::from_myvariant_hit`,
  `src/entities/variant/resolution.rs:995` — `genomic_id: hit.id.clone()`),
  and for indels MyVariant's document ID is exactly the repeat notation.
- The printed commands are built from that ID: the More:/All: block from
  `format_sections_block("variant", &variant.id, ...)`
  (`src/render/markdown/variant.rs:137`), and the `biomcp variant
  trials/articles {id}` lines from `src/render/markdown/related.rs:275`.
- The input parser's genomic-HGVS grammar (`hgvs_re`,
  `src/entities/variant/resolution.rs:25-31`) accepts only SNV substitutions
  (`chr7:g.140453136A>T`) and a bare terminal `del`. Verified rejected, live:
  `chr19:g.11106928AAG[1]` (repeat), `chr2:g.47641567_47641569del` (range
  deletion), `chr19:g.11106928delAAG` (del with sequence), and
  `chr19:g.11106928dup` (duplication).
- Gene+protein shorthand cannot carry an indel either — its grammar matches
  substitution shapes only — so there is no working spelling for an indel
  except its rsID.

## The guarantee being added

Self-consistency: **every follow-up command biomcp prints must be accepted
by biomcp's own parser.** Today the producer (card ID from MyVariant) and
the consumer (input grammar) disagree, and each side is individually
reasonable — the fix is to close the loop between them, not to change what
MyVariant reports.

## Done when

- The genomic-HGVS grammar accepts, at minimum, the four indel forms
  verified rejected above: repeat notation, range deletion, deletion with
  trailing sequence, and duplication. The design settles the complete form
  list from the HGVS genomic grammar and says which additional forms
  (insertions, inversions, repeat-expansion ranges) are in or out and why.
- Each accepted form round-trips: fetching a variant by that form returns
  the same variant the form was printed from. The design must confirm which
  indel spellings MyVariant actually answers by document-ID fetch; a form
  that cannot round-trip must not appear in printed commands — the card
  must print a fetchable spelling instead.
- A regression test pins the guarantee mechanically: render variant cards
  (including at least one indel fetched by rsID), extract every `biomcp`
  command line the card prints, and assert the parser accepts each one's
  variant ID. This test must fail on today's code.
- The unrecognized-format error's supported-format list gains the newly
  accepted forms, so the message a user sees matches what the parser takes.
- Existing input paths are unchanged in behavior: rsID, SNV genomic HGVS,
  transcript HGVS normalization, and gene+protein substitution lookups all
  keep working exactly as today, and their assertions may only be extended,
  not weakened.
- The variant surface spec (`spec/`) and CLI help that describe accepted
  formats are updated to match, so spec, help, error text, and parser do
  not disagree.

## Hard choices, settled

- The fix is widening the parser and closing the printed-command loop —
  not changing the card's reported ID, which is upstream's canonical
  identity for the variant and correct to display.
- Repeat notation is accepted because it is what MyVariant prints as the
  document ID for indels; rejecting it is what created this bug.
- The gene+protein shorthand stays substitution-only in this ticket; giving
  it indel spellings is a separate surface with its own grammar questions.

## Out of scope

- No changes to variant search, normalization, or the ClinGen tool
  contracts. No new data sources. No MCP-surface changes.

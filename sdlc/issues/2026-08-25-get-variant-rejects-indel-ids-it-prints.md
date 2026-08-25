# get variant prints indel follow-up commands its own parser rejects

GitHub issue #249 (filed 2026-08-25, biomcp 0.8.25; verified against
0.9.0-dev.5 the same day — still present).

A user looks up an indel by rsID (`get variant rs876657378`), gets back a
card whose ID is repeat notation (`chr19:g.11106928AAG[1]`), and every
follow-up command on that card — `More:`, `All:`, `See also:`, and the
therapeutic-evidence pointer — is built from that ID. Running any of them
fails with `Unrecognized variant format`. The user then has no way back into
the variant except the rsID they already used.

Verified facts:

- The parser's genomic-HGVS grammar (`src/entities/variant/resolution.rs`,
  `hgvs_re`, lines 25-31) accepts only SNV (`A>T`) and bare `del`. Rejected
  live on the dev build: repeat notation `chr19:g.11106928AAG[1]`, range
  deletion `chr2:g.47641567_47641569del`, `chr19:g.11106928delAAG`, and
  `chr19:g.11106928dup`.
- The card's ID for an rsID lookup is the MyVariant.info document ID
  (`from_myvariant_hit`, `resolution.rs:995-997`, `genomic_id: hit.id`),
  which for indels is exactly the repeat notation — so `get variant`
  produces IDs it cannot consume.
- The printed commands all use that ID: `format_sections_block("variant",
  &variant.id, ...)` in `src/render/markdown/variant.rs:137` (More:/All:),
  and `biomcp variant trials/articles {id}` in
  `src/render/markdown/related.rs:275-277` (See also:).
- Consequence two, also reported: there is no way to fetch an indel except
  by rsID. Gene+protein shorthand only matches substitution shapes
  (`gene_protein_re`, `resolution.rs:230`), and the indel genomic forms are
  rejected as above.

The missing guarantee is self-consistency: every follow-up command biomcp
prints must be accepted by its own parser. The likely route is widening the
genomic-HGVS grammar to the indel forms (repeat, range del, del with
sequence, dup/ins) — MyVariant answers by that same document ID, so the
round trip should close — but a design pass must confirm each added form is
fetchable and settle any that are not (a form that cannot be fetched must
not be printed either). A regression test should pin the round trip
mechanically: render a card, extract every printed `biomcp` command, assert
the parser accepts each one's ID.

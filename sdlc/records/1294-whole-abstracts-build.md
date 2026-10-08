# 1294 build: return whole abstracts and cleaner full text

Ticket: sdlc/tickets/1294-return-whole-abstracts-and-cleaner-full-text.md
Branch: tickets/1294-whole-abstracts, sha d00f515c
Built by a lane worker across two cycles; reviewed fresh twice; driven by
the branch lead.

## Prior evidence

The owner measured abstracts of 1,467 to 5,057 characters (median about
3,550); 0.9.1 served a 1,500-byte truncated form. The QA pass showed
the truncation marker in reading output.

## Changes

- The 1,500-byte transform-time abstract cap is gone. get article and
  batch --mode detail carry whole abstracts through the three federation
  seams. Search rows keep the 240-byte snippet, compact and --full alike;
  the cap is named in --full help and the user guide.
- JATS element joins gain one space when both sides touch with
  alphanumeric characters (JemalA becomes Jemal A); markers, degrees,
  fusions, and footnote symbols stay attached. Defects living in source
  prose stay; BioMCP does not rewrite source text.
- Mixed-citation pub-id children render through the labeled identifier
  machinery (PMID: 20647400, linked DOI); glued forms are gone.
- PMC HTML keeps title and byline when conversion drops them, and drops
  viewer/scholar/open-in-new-tab furniture; short canonical DOI, PubMed,
  and PMC links stay.
- Ragged tables (uneven colspan-expanded widths) print a banner plus raw
  rows instead of nothing.

## Proof

- Spec page spec/entity/article-text-fidelity.md: six blocks over
  recorded bytes, including the whole abstract byte-for-byte against the
  PubTator3 export for PMID 30738221 (2,369 chars, the exact length the
  ticket evidence recorded) and the EFetch-shaped whole abstract on the
  TAILORx capture; the 240-byte cap named and asserted; JATS fidelity and
  PMC HTML fidelity on recorded captures; the ragged banner on an authored
  fixture document (no recorded capture contains a ragged table).
- Receipts: the pmc6172658 capture now records the production request URL
  (rettype=xml, sha256 unchanged, so a transcription correction, not a
  refetch); the PubTator receipt records its production export URL; the
  zero-coupling digest repinned.
- Cycle 1 review REJECT (missing spec page, receipt accuracy), cycle 2
  ACCEPT with both findings resolved and both proof substitutions ruled
  acceptable.

## Deferred gaps

- Merged-cell tables as Markdown, and JATS author bylines, defer per the
  ticket.
- A timings-table row for the new spec page, and one deduplicated fixture
  fetch across two blocks, are cosmetic follow-ups.

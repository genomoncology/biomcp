# 2018 — Article entity variant links name one allele

Status: complete.

Milestone: 0.9.2

## Outcome

Each variant row in `biomcp article entities <PMID>` prints a `get variant` command that opens that row's variant. A row never points at an rsID that covers several alleles.

## Evidence

Filed 2026-10-07 from the review of the work since v0.9.1 (`sdlc/issues/2026-10-07-review-of-the-work-since-0.9.1.md`, finding 1).

- `src/transform/article/annotations.rs:73-94` prefers the rsID over the HGVS form for variant rows. `src/render/markdown/related/article_support.rs:90` prints `biomcp get variant <rsID>`.
- For PMID 30738221, the G12A, G12D and G12V rows all print `biomcp get variant rs121913529`. `biomcp get variant rs121913529 -j` returns `chr12:g.25398284C>T p.Gly12Asp` (G12D). MyVariant lists three alleles under that rsID. Two of the three rows open the wrong variant.
- `spec/entity/article-entities.md:34-37` records the shared rsID but checks only the G12C and G13C commands.

- Starts from: ticket 1296 and `sdlc/records/1296-article-entity-identifiers-build.md`.
- Keeps: identifiers for genes, diseases and chemicals; rsID links where the rsID names one allele.
- Changes: prefer the allele-specific identifier PubTator returns (ClinGen allele ID or gene plus HGVS) over the rsID. Fall back to a search command when no allele-specific form exists.
- Proof: the spec page checks the G12A, G12D and G12V commands and each opens its own variant; a test fails on `37631c357`.
- Defers: a document that mentions only one allele of a multi-allele rsID keeps the rsID link, because BioMCP has no offline rsID-to-alleles table to see the other alleles. ClinGen-allele-ID support in `get variant` is separate variant-lane work.

## Root cause

rs121913529 names a codon-12 position in KRAS, not one allele: G12A (c.35G>C,
chr12:g.25398284C>G), G12D (c.35G>A, chr12:g.25398284C>T) and G12V (c.35G>T,
chr12:g.25398284C>A) all carry it. One rsID covering several alleles is real
dbSNP biology, not a provider error. `annotation_identity`
(`src/transform/article/annotations.rs`) prefers the rsID over the HGVS
expression PubTator3 returns, so the G12A, G12D and G12V rows each carry
identifier `rs121913529` and print the same `biomcp get variant rs121913529`.
`get variant rs121913529` resolves to G12D (p.Gly12Asp), so the G12A and G12V
rows open the wrong variant. The rows themselves stay separate — the
aggregation key is (text, identifier) — but the shared rsID cannot name the
allele each row mentions. The G12C and G13C rows keep working because their
rsIDs (rs121913530, rs121913535) name exactly one allele.

## Success criteria

- PMID 30738221 prints three distinct allele-named commands:
  `biomcp get variant "KRAS p.G12A"`, `biomcp get variant "KRAS p.G12D"` and
  `biomcp get variant "KRAS p.G12V"`. Each opens its own variant:
  chr12:g.25398284C>G p.Gly12Ala, chr12:g.25398284C>T p.Gly12Asp and
  chr12:g.25398284C>A p.Gly12Val.
- G12C keeps `biomcp get variant rs121913530` and G13C keeps
  `biomcp get variant rs121913535`; those rsIDs name one allele, and those
  rows keep the rsID identifier and namespace in JSON.
- A row whose rsID covers more than one change in the document's own
  annotations never prints that rsID. The gene-plus-HGVS form carries the
  gene symbol from the document's gene annotations and the HGVS change
  PubTator3 gives, and `get variant` accepts it as exact input. Rows with no
  allele-specific form keep the mention-text command.
- 1296's contract stays: rows carry identifier and namespace, same-text
  different-identifier rows stay separate, positions serialize only behind
  `--full`, and gene, disease and chemical rows keep their links.

## Change detail

- `extract_annotations` reads the document once before aggregating: which
  gene symbol each NCBI Gene id carries (most-mentioned gene text), and how
  many distinct protein changes share each rsID.
- A mutation row keeps the rsID identity when its rsID carries exactly one
  change in the document. When the rsID carries two or more changes, the row
  carries the allele-specific identity instead: gene symbol plus the HGVS
  protein change (`KRAS p.G12A`, namespace `HGVS`), only when `get variant`
  accepts that form as exact gene-protein input. A coding HGVS stands alone
  without the gene. A protein change with no usable gene symbol carries no
  identifier and keeps the mention-text command.
- ClinGen allele IDs are allele-specific but `get variant` accepts no CA
  form, so they do not become the link; giving them a typed-back form is
  variant-lane work.
- The renderer needs no change: the existing HGVS branch already prints
  `biomcp get variant` for exact forms and falls back to the mention text
  otherwise.

## Build status

- Built on branch `tickets/2018-article-entity-variant-links-name-one-allele`,
  commit 3c94532ae, 2026-10-07, after one revival after a timeout with a
  checkpoint (nothing lost).
- Code review: ACCEPT 2026-10-07. Verified statically: the qualified
  rsID-first fix (multi-allele rsIDs alone take the gene-qualified
  HGVS path, admitted only through the classifier); the renderer
  unchanged; the spec replay routing to recorded bytes with
  deterministic best-hit scoring; receipts and the digest repin
  honest; the 1345 repin the only inventory delta; deferrals
  truthful. One report-only P2: docs/sources/pubmed.md under-describes
  the new variant-row command forms (docs pass later).

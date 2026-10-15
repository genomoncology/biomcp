# 2029 — Article variant links prefer the gene form for every allele

Status: complete.

Milestone: 0.9.2

## Outcome

Every variant row in `biomcp article entities <PMID>` that has a gene symbol and a protein change prints the gene-plus-change command. This holds when the article mentions only one allele of a multi-allele rsID. Rows without either part keep 2018's fallbacks, so a gene-less single-mention row can still print an rsID that names several alleles; BioMCP has no allele-specific form for that row.

## Evidence

Filed 2026-10-07 from the second review of the work since v0.9.1 (`sdlc/issues/2026-10-07-second-review-of-the-work-since-0.9.1.md`, finding 1).

- Ticket 2018 landed in `6b504219e` and reads complete. Its Outcome says "A row never points at an rsID that covers several alleles."
- The landed fix notices a shared rsID only when the same article names two of its alleles. A probe test with one article that mentions only KRAS G12A (`rs121913529`, gene 3845) still prints `biomcp get variant rs121913529`. That command opens G12D.
- Ticket 2018's deferral says BioMCP has no offline rsID-to-alleles table. The table is not needed. When a row has a gene symbol and a protein change, the gene-qualified form names the right allele.
- Replacing `distinct_change_key(hgvs)` with the raw HGVS text in `MutationContext::collect` (`src/transform/article/annotations.rs`) passes all 89 article tests. No test covers one allele written two ways (`p.G12D` and `p.Gly12Asp`).

- Starts from: ticket 2018 and its landed fix in `6b504219e`.
- Keeps: rsID links for rows with no gene symbol or no protein change; the mention-text fallback; 1296's identifier and namespace contract.
- Changes: prefer the gene-plus-change command whenever both parts exist, whatever else the article mentions.
- Proof: a single-mention G12A article prints `biomcp get variant "KRAS p.G12A"`, and a test fails on `6b504219e`. A test covers one allele written as `p.G12D` and as `p.Gly12Asp`.
- Defers: ClinGen allele ID support in `get variant`.

## Root cause

Ticket 2018 disambiguates an rsID only after the document's own annotations
show that rsID naming two distinct protein changes
(`MutationContext::collect` tallies the distinct changes per rsID). A
document that mentions one allele shows one change per rsID, so
`mutation_identity` keeps the rsID even though that rsID names several
alleles in dbSNP. An article that mentions only KRAS G12A therefore prints
`biomcp get variant rs121913529`, which opens G12D. The row's own data
carries the precise form: its gene annotation (NCBI Gene 3845, symbol taken
from the document's gene annotations) and its protein change (`p.G12A`).
2018 consults that pair only after the document-level signal fires, and no
offline signal exists that can tell a single-mention multi-allele rsID from
a single-allele one. BioMCP has no offline rsID-to-alleles table, so the
row's own gene-plus-change form is the only disambiguator that always
names the allele the row mentions.

## Success criteria
## Success criteria

- An article that mentions only KRAS G12A (`rs121913529`, gene 3845,
  `p.G12A`) prints `biomcp get variant "KRAS p.G12A"`. A unit test pins
  this and fails on `6b504219e`.
- Every variant row with both parts — a gene annotation that resolves to a
  symbol and a protein HGVS change — prints the gene-plus-change command,
  whatever else the article mentions. Under this rule the spec page's G12C
  and G13C rows CHANGE from `biomcp get variant rs121913530` and
  `rs121913535` to `biomcp get variant "KRAS p.G12C"` and
  `"KRAS p.G13C"`. The recorded MyVariant responses resolve `KRAS p.G12C`
  to `chr12:g.25398285C>A` and `KRAS p.G13C` to `chr12:g.25398282C>A`, one
  hit each. Those rsIDs also name several alleles — MyVariant lists G12S,
  G12R and G12C under rs121913530 and G13S, G13R and G13C under rs121913535
  — so an rsID link opens the row's allele only when the provider's ranking
  picks it. The G12A, G12D and G12V rows are unchanged, and the spec proves
  all five rows open their own variant.
- Rows without both parts keep 2018's behavior: an rsID that the document's
  annotations show naming one change keeps its rsID link; a shared rsID
  with a coding HGVS keeps the coding form, which names the allele alone; a
  shared rsID with a protein change but no usable gene symbol carries no
  identifier and keeps the mention-text command. The no-rsID HGVS fallback
  and 1296's identifier and namespace contract stay.
- A test covers one allele written two ways, `p.G12D` and `p.Gly12Asp`
  under one rsID: the two writings count as one change, so the rows keep
  their rsID links. This guards `distinct_change_key` against the raw-HGVS
  shortcut the evidence bullet records.

## Third review correction

The third review (ticket 2033, finding 5) caught two faults in this ticket's
records. The spec page claimed rs121913530 and rs121913535 each name one
allele, and the Outcome's last sentence promised no row ever prints an rsID
that opens a different allele. Both were false: rs121913530 names G12S, G12R
and G12C, and rs121913535 names G13S, G13R and G13C (verified against
MyVariant 2026-10-08), so the old G12C and G13C links worked only through
MyVariant's ranking. The spec and this ticket now state the multi-allele
truth, and the Outcome is narrowed to what the code does: rows with both
parts print the gene-plus-change command; rows without a gene symbol keep
2018's fallbacks, and a gene-less single-mention row still prints its rsID
because no allele-specific form exists for it.

## Build status

- Built on branch `tickets/2029-article-variant-links-prefer-the-gene-form`,
  head 476da62c5 (8f6527201 plus the review folds), 2026-10-07. The
  lane escalated a jointly-unsatisfiable pin set; the supervisor
  confirmed following the ticket Outcome, with the G12C/G13C spec-row
  flip recorded plainly in the ticket.
- Code review: ACCEPT 2026-10-07. Verified: the row-own-data rule
  ahead of rsID consideration with 2018's fallbacks intact; the
  approved flip's factual basis in the real capture; the two new
  receipts following the 2018 convention; the guard test genuinely red
  against the raw-HGVS shortcut; the red-on-parent shape statically
  conclusive; no regressions in the no-gene-symbol contracts. Two P2s
  (a comment narrower than the code; a missing named pin for the
  no-rsID + gene path) folded in 476da62c5 with the annotations scope
  green on the build host.

- Scoped verification at cf7280710: `cargo fmt --check` clean;
  `cargo clippy --no-default-features --lib --tests -- -D warnings` clean;
  `cargo nextest run --no-default-features --locked -E 'test(article) or
  test(pubtator)'` 702 passed, 0 failed; `mustmatch test
  spec/entity/article-entities.md --lang bash --timeout 180` passed all 20
  blocks under the article and variant-identity fixtures after
  `make prepare-spec`, including the each-row proof that all five KRAS
  gene-plus-protein commands and the rs121913529 control open their own
  variants; `tools/check-source-capture-receipts.py --root testdata/sources`
  audits 321 files with the two new KRAS captures receipted.

- The probe tests fail on 2018's landed commit: with this branch's
  `annotations/tests.rs` applied over `6b504219e`,
  `single_mention_allele_of_a_multi_allele_rsid_carries_the_gene_form` and
  `rows_with_gene_and_change_prefer_the_gene_qualified_form` fail, and
  `one_allele_written_two_ways_stays_one_rsid_change` passes, as it should —
  2018 already collapses the two writings to one change.

- The two folded P2s in detail: the multi-allele protein branch comment now
  reads "no gene-qualified exact form exists" (the returned None is also
  right for a protein change a gene is present for but no exact form
  classifies from), and the named test
  `no_rsid_row_with_gene_and_change_gains_the_gene_qualified_form` pins the
  no-rsID row with a gene and a protein change upgrading from the bare
  `p.G12A` to the gene-qualified `KRAS p.G12A`.

## Delta review

- Code re-review (post-ACCEPT delta through df760a463): ACCEPT
  2026-10-08. The cap split is a pure move; the records fixes are
  correct (the boundary root line intact, the merged-tree count
  measured 1_405 with ticket 2020's probe removal accounted); the
  third-review correction states the multi-allele truth and narrows the
  Outcome honestly; nothing else in the delta. Two P2s: the duplicated
  success-criteria heading (folded in the landing merge) and the lane
  report's stale 1_406 arithmetic (noted here as a record error).

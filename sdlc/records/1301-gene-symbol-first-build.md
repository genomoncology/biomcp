# 1301 build: route a gene-symbol first token in variant search

Ticket: sdlc/tickets/1301-route-a-gene-symbol-first-token-in-variant-search.md
Branch: tickets/1301-gene-symbol-first, sha 22d15888
Built by a lane worker across two cycles; reviewed fresh twice; driven by
the branch lead.

## Prior evidence

The KB QA report showed `biomcp search variant "SCN5A Brugada"` routing
the whole phrase into the condition filter and returning nothing, while
the gene and condition were both real. No offline gene-validation oracle
exists; the admitted in-repo oracle is MyGene's unique canonical
symbol-or-alias resolution, already trusted by discover.

## Changes

- The positional parser ends its fallthrough with a GeneFirstCandidate
  when no explicit -g exists, the phrase has more tokens, and the first
  token passes the same exact gene-token check the exact forms use.
- The caller confirms the candidate with the MyGene oracle (2.5 s shared
  budget) behind BIOMCP_VARIANT_QUERY_GENE_ROUTING (mygene default, off
  restores the old whole-phrase search exactly). A confirmed symbol
  routes first token to gene and remainder to condition; refusal,
  ambiguity, timeout, outage, and off all keep today's behavior.
- Explicit --hgvsp and --consequence ride through both branches via a
  finalize extraction shared with the standard path (review cycle 2).
- A zero-row refused phrase prints the working form in markdown and
  carries it in JSON _meta.next_commands.
- Spec page spec/entity/variant-gene-first-routing.md proves routed,
  refused-uppercase, and lowercase behavior plus the working form from
  recorded fixtures; the variant-identity fixture 400s on unexpected
  queries, so rows cannot pass vacuously.

## Proof

- Cycle 1 review: FIX (one P1, explicit filters dropped in both
  branches). Cycle 2 review: ACCEPT, the fix verified as a pure move
  with no scope creep, including a zero-connection proof that off never
  contacts MyGene.
- 51 targeted cli::variant tests, spec page 6/6 standalone, make lint
  green. Branch CI is the full-lane proof.

## Deferred gaps

- Hyphenated symbols (H3-3A) never reach the oracle; explicit -g covers
  them.
- CHANGELOG bullet owed before the next release.
- A help example line for the free-text gene+condition form is a
  separate follow-up.

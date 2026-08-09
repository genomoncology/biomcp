# The benchmark calls itself offline but nothing enforces it

**The specific flake is fixed** (`BIOMCP_S2_BASE` is now pinned to the replay
server). This issue is what remains: the corpus is offline by convention, and
convention did not hold.

## What happened

`benchmarks/output-footprint/run.py` pins eight provider base URLs at a
loopback replay server and sets `HTTP_PROXY`/`HTTPS_PROXY` to a dead port. It
missed one — Semantic Scholar. Article rows are enriched from it, so the
"offline" corpus was quietly calling `api.semanticscholar.org` on every run.

The proxy variables did not stop it: the client does not read them.

The two variants differed by whether that live call beat its deadline:

    citation_count 31 and 1     <- live Semantic Scholar, PMIDs 123 and 456
    citation_count 1594         <- the committed Europe PMC fixture

31 and 1 are the real citation counts of two real 1970s papers. Confirmed
against the live API. `influential_citation_count` appeared only in the
leaked variant, because only Semantic Scholar supplies that field.

## What it cost

`test_offline_corpus_is_deterministic_and_reports_real_token_counts` sits in
the `test` gate, which is also what `prepare` runs to decide whether
`origin/main` is green. From 22:11 on 2026-08-08 to 07:53 on 2026-08-09 the
biomcp channel completed nothing: fourteen refusals, no attempts spent, about
half the machine's CPU. Ten hours lost to one unpinned URL.

## The finding

The benchmark's offline guarantee was three separate things that each looked
sufficient and none of which was: an explicit list of base URLs (incomplete by
construction — nothing checks it against the binary), proxy variables (ignored
by the client), and a replay server that 404s unknown routes (never consulted,
because an unpinned provider does not route through it).

There are 57 `BIOMCP_*_BASE` variables in the source. The benchmark pins nine.
The other 48 are fine today only because this corpus does not reach them.

## Ask

Make the offline claim enforceable rather than aspirational. Options, cheapest
first:

1. **Pin every base.** Point all 57 at the replay server; unknown routes
    already 404, which turns a leak into a loud failure instead of a drift.
    Simple, and the list can be generated from the source so it cannot rot.
2. **Run the corpus in a network namespace with only loopback.** A real
    guarantee rather than a list, but it needs user namespaces available on
    every machine that runs the gate, including the factory.
3. Leave it and fix leaks as they are found. This is what we were doing.

Recommend 1, with the list derived from `grep BIOMCP_.*_BASE src/` in a test
so a new provider cannot be added without appearing in the corpus env.

## Separate, smaller finding

The corpus runs with `semantic_scholar_enabled: false` in its own output, and
Semantic Scholar was still called. Whatever that flag governs, it is not
"do not contact Semantic Scholar." Worth a look on its own terms — a user who
turns a source off probably expects no traffic to it.

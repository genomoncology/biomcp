# 2061 — Disable sources by name or group

Status: COMPLETE.

Milestone: 0.9.3

Reviews: revision 4864762, accept

## Outcome

A user turns off any source, or any named group of sources, with one setting: the global flag `--disable-sources` or the variable `BIOMCP_DISABLE_SOURCES`. Either takes a comma- or pipe-separated list of source ids and group names, such as `kegg,enrichr` or `commercial-restricted|alphagenome`. BioMCP never calls a disabled source, and it leaves out fields that other services pass along from that source. `biomcp health` lists the disabled sources and says why each one is off. The CLI and the MCP server behave the same way.

## Evidence

- Issue #290 asks for a switch that keeps agent workflows away from KEGG and other sources that need a commercial licence. Ian approved the design on 2026-10-13: individual sources and groups, written as a comma- or pipe-separated list.
- `BIOMCP_DISABLE_KEGG` (`src/entities/pathway.rs`) is the only switch today. It stops KEGG calls, but gene results still show KEGG pathway names that MyGene.info carries (`src/entities/gene.rs`).
- Enrichr's terms require a commercial licence, and BioMCP calls it without a key, so nothing turns it off today. `docs/reference/sources.json` records each source's terms, tier, and whether it is reached directly or through another service.

- Starts from: `docs/reference/sources.json` as the one source inventory.
- Keeps: `BIOMCP_DISABLE_KEGG` as a working alias for disabling `kegg`; sources that need a key stay off when no key is set.
- Changes: one parser for the list (comma or pipe, case-insensitive, whitespace trimmed); an unknown id or group name is an error that lists the valid names; groups come from fields in `sources.json` rather than a list of names in code, starting with a commercial-restricted group drawn from the recorded terms; each source's caller asks one place whether that source is disabled; a disabled source's passed-along fields are dropped from output; `biomcp health` and the MCP server's description list the disabled sources.
- Proof: outside-in tests through the binary and the MCP server against recorded replies: `--disable-sources kegg` makes no KEGG request and shows no KEGG pathway names in gene output; `commercial-restricted` turns off KEGG and Enrichr; the pipe and comma forms give the same result; an unknown name fails with the list of valid names; `BIOMCP_DISABLE_KEGG=1` still works; health shows the disabled sources. The configuration and source-licensing docs describe the setting, and the changelog carries the bullet.
- Defers: per-tool disabling inside the MCP server.

## Group membership ruling (2026-10-13, amended the same day)

The group shapes come from a new `commercial_terms` field recorded per
source in `docs/reference/sources.json`: `license-required` (the recorded
terms require a commercial license for commercial use), `non-commercial`
(the recorded terms allow non-commercial use only), or `null` (no
recorded commercial restriction on what BioMCP serves).

- `license-required` group: kegg, enrichr, oncokb, drugbank.
- `non-commercial` group: ddinter, pharmacodb.
- `commercial-restricted` group: the union of both groups, derived from
  the field values, never a name list in code.

Exclusions and why: cosmic stays out because the recorded reading is that
the MyVariant v68 snapshot's v68-era terms govern the fields BioMCP
serves, and only current COSMIC releases are commercially restricted;
semantic-scholar and the WHO sources stay out on their recorded open
readings. ddinter and pharmacodb are as off-limits to the commercial user
as the license-required four, so the PM moved them into the union group
under their own `non-commercial` name; each named group also works alone.
By-id disabling (for example `--disable-sources ddinter`) covers any
other source on its own.

## Proof recorded on the branch

Red-on-main probes (branch `probe/2061-disable-sources-red`, deleted
after recording): 10 of 11 `tests/disable_sources_cli.rs` tests failed on
main (the `BIOMCP_DISABLE_KEGG=1` alias test passed, proving the kept
behavior); the rmcp description test, the sources.json schema contract
test, and the changelog dev-cycle test failed on main. On the branch:
11/11 binary tests pass in under a second, 151 targeted module tests
pass, 1646 gene/pathway/drug/variant/transform/error/facade tests pass,
and the targeted Python contracts pass. `tests/disable_sources_cli.rs`
replays recorded reply shapes through one local fixture server and counts
every request, so "no KEGG request" and "no Enrichr request" are asserted
from the provider side. Spec pages `spec/entity/pathway.md` and
`spec/entity/gene.md` add the disabling cases to the routine mustmatch
corpus.

Coverage notes: by-id disabling fully covers the group members (kegg,
enrichr, oncokb, drugbank, ddinter, pharmacodb) and drops the
MyVariant-carried fields for cosmic and cancer-genome-interpreter when
those ids are disabled. Other direct sources without a group still accept
their id (the parser resolves every inventory id) but currently ignore a
by-id disable; wiring the ask into every remaining source client is a
follow-up. `search gene --pathway <id>` still filters on MyGene's own
index (no KEGG call); only KEGG-carried output fields drop.

## Gate and live evidence (2026-10-13, resumed lane)

The interrupted lane resumed at 352686c24. CI on the heads before
ff8425d37 failed the clippy gates (`empty-line-after-doc-comments` and
test-only dead code in the disable registry) and the CLI line cap
(outcome.rs at 701); d38eb1121 fixed the clippy findings and 536f82b70
rewrapped the search-all note comment back under 700. CI on ff8425d37
failed the lint-gate quality ratchet on three findings fixed by the next
three commits: 0ff1d24bd authorizes the grown file baselines in the Rust
source-size inventory (ticket 2061 stamps), 558f1fc24 adds
`--disable-sources` to the trial capability declaration, and 18debc486
excludes the disable registry from the source-registry census as a
helper module, mirroring the docs census exclusion in 4109a28a1.
ff8425d37 also carries the package boundary file-count ratchet at 1420
naming the two 2061 files.

On 18debc486, `make lint`, `make test` (Rust 4226 passed, Python 1252
passed) and `make spec` each passed once on the Linux check host holding the checkout
and gate locks, and the CI run on 18debc486 finished green. Two full-test
attempts on the Linux check host failed single timing-sensitive tests
(`raw_and_typed_article_query_validation_converges_before_provider_work`
at 120s and 63s, and once `citation_evidence_returns_the_bounded_error_when_the_bridge_stalls`),
each passing in isolation in under 12s; the failures traced to
concurrent lanes sharing Linux check host's checkout, `.cache` and target dir. A
`make spec` gene.md failure under the same contention (help-text assert)
did not reproduce on a quiet machine in three full runs, and two
concurrent spec suites corrupt each other's shared fixture state by
design; gates must hold `~/.yr-checkout.lock` and `~/.<build-host>-gate.lock`
for the whole run.

One live pass over the built binary (spec artifact of 18debc486):
unknown name errors listing every inventory id and the three groups;
comma and pipe forms produce identical health rows and reasons;
`commercial-restricted` disables KEGG, Enrichr, OncoKB, PharmacoDB and
DDInter (DrugBank stays key-gated off the health list);
`license-required` alone disables KEGG, Enrichr and OncoKB;
`non-commercial` alone disables DDInter and PharmacoDB;
`BIOMCP_DISABLE_KEGG=1` disables KEGG with the alias reason; and a live
gene card carries no KEGG fields either way (the quick view does not
request the pathways section), so the carried-field drop rests on the
recorded-reply tests.

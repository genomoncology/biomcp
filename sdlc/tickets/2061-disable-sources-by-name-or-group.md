# 2061 — Disable sources by name or group

Status: OPEN.

Milestone: 0.9.3
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

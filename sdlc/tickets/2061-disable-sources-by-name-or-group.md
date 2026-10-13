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

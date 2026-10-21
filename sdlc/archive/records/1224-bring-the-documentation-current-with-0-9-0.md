---
base: 3f2ba137
head: 3f40e5b3
---

Brought the documentation current with 0.9.0 and closed the drift gaps the
2026-09-22 audit found.

Content: the trial guide and quick reference now carry the eight status values,
the bare `--status active` refusal with its replacement guidance, the comma
alias, the zero-result hint and `_meta.upstream_total`, and the hyphenated-term
behavior. `author` joined the entity tables in `README.md` and `docs/index.md`;
`study top-mutated` and `biomcp mcp tools` gained command entries; the
provider-capture store joined `docs/policies.md`. `unknown status` is gone, the
install example pins 0.9.0, `docs/reference/mcp-server.md:17` names v0.9.0, and
the CHANGELOG names the cell-line entity.

The catalog numbers were wrong: the released 0.9.0 catalog measures 16,052
bytes and 4,083 cl100k tokens, not the 15,841/3,996 copied into three pages.
The exact numbers were removed and the test now rejects any hand-copied
`N-byte, N-token catalog` pair while keeping the dated CI budget ceiling.

Site: `mkdocs.yml` excludes `conftest.py` and adds a Release notes nav link.
Guards: the ratchet now checks the trial status vocabulary inside its own
section, the `author` entity in the entity tables, and the release-process
version facts against `Cargo.toml` and `pyproject.toml`. The source-licensing
test age-checks `reviewed_on` at 365 days without fabricating dates.

Verified on yellow at 3f40e5b3: `make lint`, `make test` (3755 Rust tests
passed), and `make spec` all OK. CI run 35739248091 succeeded on all five jobs.
Documentation run 35739247812 deployed the revision and verified it live:
`https://biomcp.org/__biomcp_revision__/3f40e5b3...txt` returns the SHA,
`/conftest.py` returns 404, and the trial page carries `active_not_recruiting`
guidance. Code review ACCEPT with three guard-hardening notes remediated; the
one P1 (a stale ticket-1222 fact line) was corrected in the same branch.

Residual: the Cloudflare cache can serve pages up to ten minutes behind a
deploy. Ticket 1222 keeps the remaining release-hardening items; the
architecture-document facts are among them.

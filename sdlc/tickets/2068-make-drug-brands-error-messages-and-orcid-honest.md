# 2068 — Make drug brands, error messages and ORCID honest

Status: OPEN.

Milestone: 0.9.3

## Outcome

A drug card's brand list names only that drug's brands. Error messages describe the real failure. Article search writes no warning for a working fallback. ORCID author lookups work without a token when ORCID's public API answers.

## Evidence

Filed 2026-10-13 from the post-release install and product test of v0.9.2 (tag `4049666f5`), run on Linux x86_64 and macOS arm64 against every install channel.

- `get drug trastuzumab` lists Enhertu, which is trastuzumab deruxtecan, as a trastuzumab brand. `get drug Tarceva` lists the chemical name "[6,7-Bis-(2-methoxy-ethoxy)…]-amine" as a brand and omits Tarceva. The rows come from DrugBank through MyChem. 0.9.1 does the same.
- `get trial NCT99999999` says "Retry the remote source" for a trial that does not exist. `get article 999999999999` says "Unsupported identifier format… PMID (digits only)" for an all-digit input. `get variant "BRAF Z999Q"` reports not found and suggests a search with the invalid residue Z instead of rejecting it.
- Every article search writes "WARN biomcp_cli::error: External provider operation failed provider=\"PubTator 3\" operation=\"visible article metadata fallback\"" to stderr. 0.9.1 does the same.
- `get author orcid:0000-0002-4299-3018` and `author papers` refuse without `ORCID_ACCESS_TOKEN`, though ORCID's public API answers that iD without a token. GitHub #289's user relies on this path.
- `biomcp health` shows FDA Orphan Drug Designations as temporarily unavailable; the FDA page redirects scripted clients and answers a browser, so the message misstates the cause.

- Starts from: the drug brand transform, the trial, article and variant error paths, the PubTator fallback, the ORCID client, the FDA orphan source.
- Keeps: 0.9.2's drug and label resolution rules.
- Changes: drop brand rows that name a different ingredient or no product; return "not found", "not a PMID" and "invalid residue" errors; log the working fallback at debug level; use ORCID's public API when no token is set and the member API when one is; name the FDA redirect in the health row.
- Proof: recorded-reply tests for the brand lists, each error message, the quiet fallback, a tokenless ORCID lookup and author papers, and the FDA health row; each fails on `4049666f5`.
- Defers: fixing the FDA block itself.

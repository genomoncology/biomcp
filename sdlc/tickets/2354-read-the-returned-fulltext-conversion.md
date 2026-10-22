# 2354 — Read the returned fulltext conversion

Status: OPEN.

Milestone: 0.9.4

## Outcome

After BioMCP returns an existing managed fulltext conversion for an article, the documented immediate line-read commands can read that same conversion when a later provider attempt is unavailable. Check the actual file and persisted identity before repairing resolution. Preserve explicit cache controls and honest absence when no usable conversion exists.

- Changes: claim actual fulltext resolution/line-read files and outside-in tests after diagnosis, plus `CHANGELOG.md`. No broad asset or cache refactor.
- Proof: reproduce article10811118 initial retrieval followed by `--lines 1:60` and `--lines 61:100` in one isolated cache, retaining stat/read proof before each call; an outside-in replay succeeds after later provider unavailability and refuses absent/unusable local content.

## Evidence

- PM m3492 authorizes uncovered confirmed defects from every research observation. Research c001/call-0032 (TSV physical line6) is not the two-PMID sequence tested by withdrawn2347. Those earlier j4148 non-reproductions remain correct historical evidence. pm refuses reopening a withdrawn ticket and directs creation of a fresh OPEN ticket; 2347 remains withdrawn.
- Current-main M5 j4335 on f7165ac3b6e2c25367f31f25393343313564b5e7: `biomcp --json get article 10811118 fulltext` exited0 and returned a managed text path. Immediate `biomcp --json get article 10811118 fulltext --lines 1:60` and then `--lines 61:100`, same home/cache, both exited1/not_found after renewed provider work. The first probe did not stat the file before each read; the owning lane must establish that exact boundary before choosing a fix. Yellow ticket-evidence/2350/followup-probe retains actual outputs and commands.
- Coordinator restatement: make documented reads consume the available returned conversion, rather than allowing a new unavailable provider response to erase that capability. No hypothetical fix before local-file/identity verification.

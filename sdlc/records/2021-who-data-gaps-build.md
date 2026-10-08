# 2021 build record — WHO data gaps reach every caller

Landed 2026-10-08, merge 208858393. Review verdicts live on
sdlc/tickets/2021-who-data-gaps-reach-every-caller.md.

Outcome: a failed WHO export degrades the drug search with the reason
in the JSON envelope and raw MCP output; the sync error names the
failing file and its missing column with the resolved data directory
in the recovery; a partial sync reports partial with per-file outcomes
and a nonzero exit. Gates: branch CI green at d41d5e3ca; the fix-round
P1 (the error.rs pin) fixed and the third review's finding-12 re-review
ACCEPT; merged-tree CI green at 208858393.

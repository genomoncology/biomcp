# Degraded and capped sections should print their recovery command

Found in the botassembly knowledge-base run (2026-08-27,
experiments/188-shank3-knowledge-base, `kb/raw/RUN-LOG.md`), where a
foreign agent hit both shapes and handled them only by inference:

- A federated article search failed across all four sources; the agent
  recovered by retrying with `--source pubmed` on its own initiative. The
  output did not suggest that command.
- A diagnostics section was capped ("up to 10" of 31) without a printed
  continuation command; the agent re-queried differently to see the rest.
- Degraded sections (Open Targets, PubTator3 timeouts) report the
  degradation honestly but offer no next move.

The GWAS section of `search all` already does exactly the right thing: it
prints the timeout, keeps the card alive, and names the direct retry
command. The idea is one affordance family applied everywhere: whenever a
section is degraded, failed, or capped, the card prints the exact command
that recovers or continues it. The knowledge-base run shows agents will
find these moves anyway — the tool should hand them over.

Adjacent to the landed 1063 (partial-failure honesty for the pathway
section); this generalizes the printed-recovery behavior across sections.
Recorded for triage.

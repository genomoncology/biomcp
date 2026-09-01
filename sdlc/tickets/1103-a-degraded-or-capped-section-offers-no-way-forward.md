---
flow: build
priority: 4
---

# A degraded, failed or capped section reports its state and offers no way forward

A card that loses part of its content tells the reader honestly and then stops there. The reader has to work out the next move alone.

Three shapes, found when an outside agent built a knowledge base against BioMCP on 2026-08-27 and recorded in `sdlc/issues/2026-08-27-degraded-and-capped-sections-should-print-their-recovery-command.md`:

- A federated article search failed across all four sources. The agent recovered by retrying with `--source pubmed` on its own initiative. The output did not suggest that command.
- A diagnostics section was capped at 10 of 31 with no printed continuation command. The agent re-queried a different way to see the rest.
- Sections degraded by an upstream timeout report the degradation and offer no next move.

The behaviour wanted here already exists in one place. The GWAS section of `search all` prints its timeout, keeps the card alive, and names the direct retry command.

The knowledge-base run shows agents find these moves anyway. The point is that they should not have to, and a weaker consumer will not.

## Required behavior

A section that is degraded, failed or capped prints the command that recovers or continues it.

The printed command runs as printed.

The affordance is one behaviour applied across sections, not a per-section decision made again each time.

## Done, observably

- A capped section names the command that shows the rest.
- A section degraded or failed by an upstream problem names the command that retries it directly.
- The commands printed in those cases parse and run.
- A newly added section that can degrade or cap cannot ship without this affordance.

## Boundary

This ticket does not change which sections cap, what the caps are, or any timeout. It does not change the GWAS behaviour that already does this. Returning partial results when a source is unavailable is a separate ticket and is not in scope here; this ticket is about what a card says once it has already decided to report a degraded state.

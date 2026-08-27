---
flow: build
priority: 6
---

# Print one coordinate build per variant card

Filed from `sdlc/issues/2026-08-27-population-card-mixes-coordinate-builds.md`.

`get variant rs1426654 population` displays the resolved GRCh38 coordinate
in its header (`chr15:g.48134287A>G`) while the `More:`/`All:` follow-up
commands beneath it print the GRCh37 spelling (`chr15:g.48426484A>G`).
Both work — the parser serves either build, verified — but a reader
comparing the header against a copied command sees two different positions
for one variant with no explanation of the switch.

## Done when

- Every follow-up command a variant card prints uses the same coordinate
  spelling the card resolved and displayed in its header, or the card
  states the build of each command's coordinate explicitly. The design
  settles which, and the choice is written into the surface spec.
- The printed commands still round-trip (ticket 1056's guarantee holds) —
  whichever spelling prints must parse and fetch.
- Pinned by a test on a variant whose GRCh37 and GRCh38 positions differ,
  so a mixed-build regression fails the suite.

Filed as build, not quickfix: green suite; display-consistency proof is
authored.

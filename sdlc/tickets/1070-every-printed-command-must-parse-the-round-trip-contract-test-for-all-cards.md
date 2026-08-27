---
flow: build
priority: 9
deps: ["1068"]
---

# Every printed command must parse — the round-trip contract test for all cards

Ticket 1056 built the round-trip guarantee for variant cards: render the
card, extract every printed `biomcp` command, assert each one's variant
ID parses. Since then the guarantee has caught nothing new mechanically —
and this week's hunt found the same violation alive on another pivot
(trial cards printing `get drug JAG201`, which fails; 1068 fixes that
instance). The guarantee exists per-family; the enforcement does not.

## Done when

- An offline contract test renders a representative card for every card
  family that prints commands — trial, gene, disease, drug, article,
  author, adverse-event, pathway, diagnostic — from fixtures, extracts
  every `biomcp …` command line each card prints (More:, All:, See also:,
  inline suggestions), and asserts each command is accepted by the CLI's
  own argument parsing — the same parse a shell would perform, no network.
- The extraction reads the rendered markdown the way a user copies it,
  including quoted arguments, so a card that prints an unparsable string
  fails the test.
- The test fails on today's pre-1068 code for the JAG201 case (proving
  teeth) and passes once 1068 lands (the dep ordering enforces this).
- Adding a new card family means adding its fixture to this test — the
  test says so in a header comment.

This is the mechanical version of "captures or it didn't happen," turned
on our own output. It would have caught 1056's bug, this week's trial-card
bug, and the placebo-first variant — before any user did.

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

## Operator amendment — 2026-08-28

The first attempt correctly refused one contradictory requirement. The CLI
parser accepts `biomcp get drug JAG201` because drug resolution happens after
argument parsing and requires provider data. This ticket now enforces syntax
only. It does not claim to prove that a parsed identifier resolves, and it does
not require the permanent test to fail before 1068. Ticket 1068 owns the
JAG201, placebo, saline, and other unverified-drug-text regression proof.
This amendment supersedes the original third Done-when bullet and the final
claim that this parser contract would have caught those semantic drug bugs.

The permanent contract must render fixtures for every current detail-card
family that prints commands: variant, gene, disease, drug, trial, article,
author, adverse-event, diagnostic, protein, PGx, and pathway. It must extract
the copied command text with shell quoting intact and pass each tokenized
command through `Cli::try_parse_from` or the equivalent production Clap entry
point. The proof must also demonstrate that this contract rejects an injected
malformed command. A mutation during the test or design proof is sufficient;
the repository must not retain a deliberately malformed production command.

This ticket does not add an offline drug allowlist, a fixture-backed semantic
resolver, provider calls, or family-specific identity validation. It does not
cover search-result guidance, discovery responses, pagination continuations,
or commands that no detail-card renderer prints. Existing family-specific
semantic checks such as variant identifier parsing remain unchanged.

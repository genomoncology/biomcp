# 2335 — Point reversed help words to the right form

Status: COMPLETE.

Milestone: 0.9.4

Reviews: revision 54502b364f3c1b5d34659ecb653b67b8beacc798, accept

Reviews: revision 654c8a004e42fbd921bffef143cd842b54aa1ba4, accept

Reviews: revision f68c3f75dcdf5a1b4b595ce675e0ab43fdf71837, accept

## Outcome

When a user or agent types the article-search help with its words reversed, BioMCP shows the right help or an error that names the exact correct command. The same holds for every entity and verb pair that accepts both orders in speech but only one on the command line.

- Proof: CLI tests run each reversed help form the study recorded, and the reversed form for every other entity, and see the correct help or an error naming the exact correct command, with exit codes unchanged for real errors.

- Changes: `src/cli/shared.rs`, `src/cli/shared/reversed_help.rs`, `tests/reversed_help_contract.rs`, `tests/feature-links.json`, `CHANGELOG.md`.

## Evidence

- On 2026-10-21, research ran five fresh agents with BioMCP 0.9.4 (commit cdb9f4a1b) on literature searches for five hereditary cancer variants, with no answer key. Exact commands and output are in research's protected store under runs/0010/five-v1.
- Item 2: three of the five agents typed the article-search help with the words reversed. The help or the error for that form should point to the right one. The exact argv is in the study store.
- Ian asked on 2026-10-21 to clear every known issue in 0.9.4.

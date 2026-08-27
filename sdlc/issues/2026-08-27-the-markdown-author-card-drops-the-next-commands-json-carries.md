# The markdown author card drops the next-commands the JSON carries

`biomcp get author semanticscholar:1716151` in markdown ends at the ORCID
line — no More:/See also: block, no pointer to the author-papers pivot.
The same request with `--json` carries the real next command:

    next_commands: ["biomcp author papers semanticscholar:1716151"]

Verified 2026-08-27 against 0.9.0-dev.6 (captures:
experiments/193-biomcp-bug-hunt/calls/rt-author-card.txt). Mechanism,
verified in code: `src/render/markdown/author.rs:178` builds the markdown
card's meta with `next_commands: vec![]` — hardcoded empty — while the
JSON layer (fixed by ticket 1060) carries the truthful pivot command. The
1060 flight wired the entity side; the markdown renderer was never
connected, so human-readable users never see the card's one follow-up.

Fix shape: render the More block from the same next_commands source the
JSON uses, so the two surfaces cannot disagree again.

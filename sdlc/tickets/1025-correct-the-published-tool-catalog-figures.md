---
flow: build
priority: 4
deps: ["1024"]
---
# Correct the published tool catalog figures

`docs/blog/we-deleted-35-tools.md` quotes a tool catalog of 6,707 bytes and 1,628 tokens. Measured on 2026-08-19, the development build's catalog is 15,704 bytes and 3,974 tokens, and the published 0.8.25 release is 21,701 bytes and 5,599 tokens. The published figure is not close to either, and it is the number a reader would quote back at us.

The documentation also states a 16,000-byte and 4,000-token budget without making clear that the budget is enforced on the development branch and that the currently installed public release is above it. A reader who installs 0.8.25, measures its catalog, and compares it against the documented budget finds a discrepancy the documentation does not explain.

Correct the figures and say plainly which build each number describes. Do not delete the comparison — the story the post tells is real and the current numbers still support it.

## Done when

- Every catalog byte and token figure in public documentation matches a measurement of a named build, and names which build it measured.
- Where a budget is quoted, the text says what it applies to and which builds it is enforced on.
- No public figure claims the currently published release is inside a budget it is outside of.

## The stale assertion

`tests/test_documentation_consistency_audit_contract.py`, in `test_blog_try_it_and_install_copy_are_consistent`, asserts the literal string `1,628 \`cl100k_base\` tokens at the 0932 snapshot` and the literal string `16,000 bytes and 4,000 tokens`. The first of those pins a figure that is no longer true of any build, so that assertion is itself part of the defect and may be restated with the corrected figure. Say so in the commit message: the old figure is the defect, and the restated assertion is the proof of the fix.

Two other tests read the same blog file but do not assert any figure — `tests/test_mcp_tool_catalog.py` checks only that each tool name appears, and `tests/test_public_install_docs_contract.py` checks only headings and install commands. Neither should need touching; if either goes red, that is a signal the edit went wider than the figures.

## Why this is a build and not a quickfix

This was filed as a quickfix and refused on 2026-08-23: `lint` and `test` both ran green before any change, which is the quickfix flow's grounds for refusal. That refusal was correct. The stale figure is asserted by a passing test — the test agrees with the wrong number — so there is no red to reproduce. The proof here has to be authored, not reproduced, which is what the build flow's design stage is for.

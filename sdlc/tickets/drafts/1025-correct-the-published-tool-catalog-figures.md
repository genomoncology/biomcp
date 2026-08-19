---
flow: quickfix
priority: 5
hold: draft for review; do not promote until Ian releases this
---
# Correct the published tool catalog figures

`docs/blog/we-deleted-35-tools.md` quotes a tool catalog of 6,707 bytes and 1,628 tokens. Measured on 2026-08-19, the development build's catalog is 15,704 bytes and 3,974 tokens, and the published 0.8.25 release is 21,701 bytes and 5,599 tokens. The published figure is not close to either, and it is the number a reader would quote back at us.

The documentation also states a 16,000-byte and 4,000-token budget without making clear that the budget is enforced on the development branch and that the currently installed public release is above it. A reader who installs 0.8.25, measures its catalog, and compares it against the documented budget finds a discrepancy the documentation does not explain.

Correct the figures and say plainly which build each number describes. Do not delete the comparison — the story the post tells is real and the current numbers still support it.

## Done when

- Every catalog byte and token figure in public documentation matches a measurement of a named build, and names which build it measured.
- Where a budget is quoted, the text says what it applies to and which builds it is enforced on.
- No public figure claims the currently published release is inside a budget it is outside of.

---
base: ec1c8f9a
head: 4de8e686
---

Closed the round-two check gaps from the 2026-09-28 review file.

The workflow contract now pins each pinned release job's ENTIRE
step list — ids, order, and count — so a step slipped before a
pinned one fails even when every pinned hash stays clean, and
BASH_ENV is banned in every env block and with: value; duplicate
needs: lines fail both the parsed-document comparison and a
per-job line-count check. The review grammar became a whole-line
allowlist that scans headings and paragraphs too: a status
declaration in any decoration carrying a state word fails,
including the compound shape (a pending heading beside an honest
ACCEPT record); verdicts must declare a state; a negated
resolution word does not resolve; scopes may hold only batch/item
words and numbers. The stdio guard's .output() exemption is gone
and unknown spellings fail closed. The wait ratchet catches
elapsed()/Instant comparisons in either order and any operator,
resolves Rust use-aliases and assigned Python time modules, and
counts its markers against a raised ceiling.

The gate cycles caught what static work cannot: an unclosed
delimiter where a marker was inserted into a one-line block (the
line is now split so the marker sits where rustfmt keeps it), and
the discovery that rustfmt relocates trailing comments after an
opening brace — markers now ride statement lines (lets and bare
sleeps) that fmt preserves, with the inventory raised in the same
commits and the reason recorded. In-tree catches were normalized
honestly after review REJECT: 1233's and 1254's verdict lines now
say what actually happened (no independent acceptance is recorded;
no separate design review ran), and 1221's truncated verification
line is restored in full.

Evidence: review REJECT once (the allowlist never scanned
non-bullet lines; two normalizations invented verdicts; one
truncated evidence), all folded with the reviewer's payloads;
146-plus local tests green; yellow gate green at 4de8e686 — lint,
test, spec, stress, zero failed lines.

---
priority: 1
---

# 2250 — Mark supplement search incomplete when files are unreachable

Status: OPEN.

Milestone: 0.9.4

## Outcome

`article supplement-search` lists every supplement file it could not read and marks the search incomplete, so "no hits" never reads as "not present".

## Evidence

Reproduced on 0.9.3: `biomcp --json article supplement-search 38368708 I448T` returns success with no files and no hits, although `get article 38368708 assets --asset-view coverage` lists a blocked DOCX, and searching that file alone errors. The same happens for five unreachable files of 27397505.

## Keeps

Keep successful hits and the deadline. Use the same unavailable-file wording as the asset coverage view.

## Proof

Replay tests for both papers show the unreachable files named and the result marked incomplete in JSON and Markdown.

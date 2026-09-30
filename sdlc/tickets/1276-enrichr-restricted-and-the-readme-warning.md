# 1276 — Enrichr restricted and the README data-terms warning

Filed retroactively 2026-09-30 (the second go-request review found the record with no ticket file; the work landed 2026-09-29 on branch `tickets/1276-enrichr`, merge `9eb9a121`).

## What landed

Enrichr moved to tier 3 with its terms stated plainly (free for academic and non-profit use; commercial use requires a Mount Sinai Innovation Partners licence; the terms state it is not for treating or diagnosing human subjects), read from the live terms submenu page after the 09-27 pass hit the broken help page. The README gained the Data terms section ahead of the hero, linking the licensing page, with its content pinned by the landing-copy contract. AGENTS.md gained the standing licensing rule. The evidence file's counts became 45 verified, 2 changed, 1 restricted.

## Evidence

Branch run 36593025808 green at ac51eceb; main green at 36600156017 on the merge commit 9eb9a121. The 2026-09-29 review of this round found the licensing edits that had silently failed to land; the 1278/1279 rounds landed them (tier table, section heading, sources.json notes, evidence header) and the heading-aware tier test now pins page to registry.

## Review

- Code review: no dedicated reviewer dispatch existed at landing — stated plainly; the earlier record's prose acceptance hid that. The landed state was later verified by the 1281 round's read-only reviews (dispatches 85cf92fc and 94c43888, Claude vendor), which read the tier table, the README warning and the evidence corrections this ticket's follow-ups landed.

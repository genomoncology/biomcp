---
ticket: 1284
heads: 5c8e6112, 76b19c76, 3aba3928, 5f19f0a9
merge: (this record's merge)
---

# 1284 — the fifth go-request round

## What changed

The draft-visibility blocker (container-publish contents: write, with the guard test), the pinned exit-code case, the mid-line gh guard, the reference-tight raise-review grammar with the review's five rejections, record 1283 with both failing runs named, dispositions for the fourth and fifth review files, the changelog wording (RNA levels per the CLI help; the internal phrasing fully out), the DepMap and Cell Model Passports terms pages with evidence rows (52 pinned), and the rehearsal: checklist section 1 gained the item, and the rehearsal runs on the public scratch repository genomoncology/biomcp-release-rehearsal — a private copy stops mid-flight because private-repository runners are not available to this account — with the publish step as a credential-free twine check (no TestPyPI registration exists and none is planned). The go request waits on that rehearsal run.

## Evidence, read from the API

- Branch runs: 36853923862 and 36854070476 FAILED the TBD scan (a test fixture carried the literal placeholder; 36853923862 also flaked stress-lane); fixed at 3aba3928, run 36856199864 success all eight jobs; 5f19f0a9 run 36859236631 success all eight jobs.
- Code review: ACCEPT 2026-10-01 dispatch 16d922f8-6baa-4a4a-be90-959000d4580b (Claude vendor, read-only); one second-priority finding folded at 5f19f0a9 (the word "bookkeeping" still present in the changelog line; reworded to "a push that skips the Rust jobs still runs the record tests"), plus a note that the scratch repository's graph is external to the clone.
- The go request is withheld until the rehearsal run exists, per checklist section 1.

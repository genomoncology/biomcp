---
base: 1f29788a
head: dcae38e2
---

The 1251 and 1252 findings from the 2026-09-26 review, closed.

1251: the schema merge now panics on every clash it does not cover —
type clashes, differing scalars, enum against free text, and an enum
or const sitting on exactly one side of a same-type merge — each
pinned by a should_panic test, so a future branch change that
introduces an unhandled shape fails at test time instead of
publishing a quietly-wrong union. The drift tripwire pins the
collision outcomes (the merged source enum's eight values, the
text-or-list types) against hand-written literals; only the key set
derives from the branch table. ADR 0002 states the
narrower-than-branch case and marks the Gemini construct acceptance
as not verified here. The CHANGELOG bullets tell the whole truth:
in-body validation errors are isError, missing fields, wrong types,
and erepo unknown fields stay -32602, and erepo now rejects unknown
fields with a wrong-type limit or offset erroring.

1252: every "one CPU" text now says two, the scale-factor wording
covers helper-built watchdogs only, the wait ratchet requires a
reason word after its marker and catches from-imported sleep,
asyncio.sleep, bare sleep after a use-import, and strict elapsed()
deadline checks, the single-CPU pinning guard catches every spelling
including attached and equals forms, and the stress lane runs in CI
as its own job so it cannot rot (with a shallow checkout, keeping
the full-history contract at its pinned two).

Evidence: code review ACCEPT with two report-only P2s (the one-sided
enum corner, the taskset spellings) both folded; yellow gate at
dcae38e2 — lint, test, and spec OK after one cycle that caught the
stress job's full-history checkout against the offline gate contract.

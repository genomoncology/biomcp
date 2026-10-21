# Give every record and issue file an honest disposition

From the 2026-09-27 review file (Records section) and Ian's feedback
point 1: stop saying finished; mark every review item fixed, deferred
or open, with the commit. This ticket is documentation only; every
claim must name its evidence (a commit, a file, a run).

## Review

- Design review: n/a (documentation)
- Code review: n/a (documentation)
- Verification: both review files carry complete dispositions;
  `grep -L "Decision\|Resolved"` over sdlc/issues returns only
  deliberately-open files; CI green over the edited files

## Problem

- The 1238 record says every open issue file has a decision. Four
  have neither Resolved nor Decision: the DDInter Mac run, the
  one-CPU deadlock, `2026-09-26-review-follow-ups-for-1242-through-1254.md`,
  and spec-contracts (Verification only).
- `2026-09-26-review-follow-ups-for-1242-through-1254.md` is linked
  from nowhere and has no resolution. Four of its findings were
  dropped without a note: the DDInter HTML label (fixed by ticket
  1256), the unrecorded brand-name change, the GenCC tests (ticket
  1257), and the 1244 release-prep items missing from 1253.
- Ticket 1254 contradicts itself: `:111` says items 5-12 not
  started; `:187` says batch 3 review pending; `:188` and `:253`
  are stray merge text; the record frontmatter says `head: d8475987`.
- Ticket 1242 `:172` says the yellow gate has not run; the record
  says it passed at `bb516645`.
- Ticket 1243 has a stale "Unverified here" paragraph and cites old
  `manager.rs` line numbers.
- The 1254 record says credentials can never be redirected in
  release builds. A release build honors the override when both test
  variables point at loopback. State the real rule.
- Spec-contracts issue `:142` calls lifecycle-test failures
  pre-existing with no tracking issue.
- Ticket 1253 says 24 changelog bullets are missing; a dry run says
  26 (adding 1238 and 1243). Its after-tag list omits closing GitHub
  #284 once 0.9.1 reaches PyPI.
- `yellow-build-overlaps` `:3` and `test-server-terminated` `:3`
  say "Status: open" above a Resolved section.
- The yellow-overlap Resolved section says BioMCP gate scripts take
  the lock; the lock code lives in dotfiles, and dotfiles has an
  open issue showing the lock free during BioMCP checks.
- `2026-09-11-variant-display-mixes-transcript-numbering.md:11`
  parks a real clinical-report failure to 1.0 with no owner or
  trigger; the other 1.0 candidates lack owner and trigger too.
- Decision text uses shorthand a new reader cannot follow: "the
  1252 regime", "the marked-watchdog form", "the 1242 render walk".

## Fix

1. Item-by-item disposition in both review issue files
   (2026-09-26 and 2026-09-27): every finding marked Fixed (with
   the ticket and commit), Deferred (with an issue file and
   reason), or Open (with an owner and trigger). Nothing unmarked.
   Link the 2026-09-26 file from wherever its siblings are linked.
2. Decision or Resolved sections for the four undecided files, each
   stating what is true now.
3. Resolve the ticket contradictions (1254, 1242, 1243): the truth
   per the records and git history, stray text removed, frontmatter
   head corrected.
4. Correct the 1254 record's credentials claim to the real rule:
   debug builds need the loopback+signal pair; release builds honor
   the override when both test variables point at loopback — the
   protection is the pair requirement, not the build mode alone.
5. Open the tracking issue the spec-contracts `:142` claim needs,
   or correct the claim to what the guard fix proved.
6. Update 1253: 26 missing bullets (name 1238 and 1243), add the
   #284 close after PyPI, and fold in the 1244 release-prep items
   the 2026-09-26 review listed as missing.
7. Fix the two Status:open-over-Resolved files and the
   yellow-overlap lock wording (the lock is a dotfiles-side
   protocol; BioMCP scripts do not take it).
8. Owner and revisit trigger for every 1.0 candidate. The variant
   transcript numbering failure becomes its own ticket (a real
   report failure, not a parked idea).
9. Rewrite the named shorthand in plain words wherever it appears
   in sdlc/.
10. The unrecorded brand-name change from 2026-09-26: find it (the
    review named no file), state what it was in the disposition, and
    record or fix it.

## Acceptance

- Both review files carry a complete disposition table or inline
  marks; a reader can trace every item to a commit, an issue, or an
  owner+trigger.
- `grep -L "Decision\|Resolved"` over sdlc/issues returns only
  files whose openness is deliberate and stated.
- No ticket contradicts its record; frontmatter heads match
  reality.
- 1253's numbers match a fresh dry run of the changelog check,
  recorded with the command.
- The 1.0 candidates each name an owner and a revisit trigger; the
  transcript numbering failure has a ticket.
- CI green (the review-status contract must pass over the edited
  files).

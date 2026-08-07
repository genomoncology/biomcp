---
flow: triage
priority: 9
---
# Triage: 256 merge crash — PDF fixture breaks git diff UTF-8

The ticket committed a real 730-byte PDF test fixture at `tests/fixtures/article/fulltext/pdf/cdc_sti_guideline.pdf`. The biomcp repo has no `.gitattributes`, so git does not treat `*.pdf` as binary. `git diff pre_main..post_main` during march's `verify+merge` emits the PDF bytes inline in the diff text stream; march's `lib/flow.py:835 _git_stdout` calls `subprocess.run(..., text=True)`, which forces UTF-8 decoding of stdout, which fails on the PDF's binary header bytes (`e2 e3 cf d3 …`).

Completed under March on 2026-04-20, as March ticket 261. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/261-triage-256-merge-crash-pdf-fixture-in-git-diff

The landed commit range could not be recovered from git, so no
record accompanies this entry. The work products above are the
evidence that survives; the absence of a record is a gap in what
git can still prove, not a sign the work is missing.

---
flow: quickfix
priority: 9
---
# Declare binary asset types in .gitattributes before PDF fixture merges

Ticket 256 proved the article fulltext PDF path is ready, but verify+merge crashes before the branch can land because biomcp does not declare tracked binary assets in `.gitattributes`. When `git diff pre_main..post_main` includes a real PDF fixture as text, march decodes the raw diff stream as UTF-8 and crashes on the PDF header bytes. This is a repo prerequisite, not a redesign of 256: the feature ticket remains coherent once the repo advertises binary content correctly.

Completed under March on 2026-04-20, as March ticket 262. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/262-declare-binary-asset-types-in-gitattributes-before-pdf-fixture-merges

The landed commit range could not be recovered from git, so no
record accompanies this entry. The work products above are the
evidence that survives; the absence of a record is a gap in what
git can still prove, not a sign the work is missing.

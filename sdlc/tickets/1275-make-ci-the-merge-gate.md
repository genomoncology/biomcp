# Make CI the merge gate

Ian's 2026-09-29 direction: the round is slow because CI runs only
after a merge, the local gate does not match CI, gates serialize,
CI compiles from scratch, and flaky timing tests burn a cycle each.
This ticket lands the cheap speed fixes; the bigger rewrites move
to 1.0.

## Fix

1. CI's push trigger now includes `tickets/**`: a branch's green CI
   run is the merge condition. Branches run in parallel on a free
   public-repo service, so the one-at-a-time yellow bottleneck and
   the gate-passed-CI-failed gap both close. Yellow stays available
   for deep verification but is no longer required before every
   merge.
2. Every CI job that checks out now runs Swatinem/rust-cache, and
   nextest installs as a prebuilt binary instead of `cargo install`
   (three sites). Each run stops paying the from-scratch compile.
3. The remaining low-severity check spellings close as recorded
   decisions ("the check stops here, accepted") in the two review
   files' dispositions — the direction allows exactly that, and the
   1.0 rewrite to real tools (clippy disallowed-methods, actionlint)
   supersedes further pattern patches.

## Decisions recorded

- The review-grammar, stdio-guard and wait-ratchet spellings the
  2026-09-28 review listed stop here, accepted: the checks already
  catch the shapes any realistic drift takes; the remaining
  spellings are adversarial constructions, and the 1.0 replacement
  by compiler-backed tools (clippy disallowed-methods understands
  aliases and imports; actionlint/zizmor cover workflows) closes
  the class. Filed as 1.0 issues with owners and triggers.

## Review

- Design review: n/a (Ian's direction named the changes)
- Code review: n/a (CI infrastructure; the branch's own green CI run is the review, plus the 1276 catch proved the contract)
- Verification: CI run 36574425104 on the branch (success), then
  main green at e18501b8 (run 36578761920, success)

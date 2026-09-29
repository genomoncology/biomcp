---
base: 3d293ad7
head: e18501b8
---

Made CI the merge gate, from Ian's 2026-09-29 direction. The round
was slow for six named reasons; this ticket addresses the four with
cheap fixes, and the two that need rewrites move to 1.0 as issues.

CI's push trigger now includes `tickets/**`, so a branch's green CI
run is the merge condition: branches run in parallel on GitHub's
free minutes, the one-at-a-time yellow bottleneck and the
gate-passed-CI-failed gap both close, and yellow becomes optional.
Every CI job carries Swatinem/rust-cache, and nextest installs as a
prebuilt binary instead of `cargo install` at three sites. The
low-severity check spellings from the review files close as
recorded decisions ("the check stops here, accepted") — the 1.0
replacement by compiler-backed tools (clippy disallowed-methods,
actionlint/zizmor) supersedes further pattern patches. Four 1.0
issues are filed with owners and triggers: signal-based test waits,
the compiler-tool checks replacement, workflow linters, and test
suite speed.

Evidence: this branch's own CI run was the first under the new rule
(run 36574425104, success — the flow works); the pin-contract catch
(1276) exercised it end to end; main green at e18501b8 (run
36578761920, success, including full-features and
windows-contracts).

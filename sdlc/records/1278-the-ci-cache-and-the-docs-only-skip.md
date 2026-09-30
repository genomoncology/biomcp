---
ticket: 1278
branch: 37951529
ci: 36633279293
main: 36636929043
---

# 1278 — the CI cache and the docs-only skip

The docs-only skip classifies by the files changed since the merge-base with origin/main (the first landing diffed the previous tip, which let a failed commit followed by a markdown commit skip every Rust job — the 2026-09-29 second review demonstrated the hole with 95a4998b and 7e9b42c5). The allow list is sdlc/, notes/, plain docs pages, README, AGENTS.md and CHANGELOG.md; spec/, skills/, any src/ markdown and the compiled CLI reference run the full suite. The cache runs after checkout and after the pinned toolchain, is pinned to the v2 tag's dereferenced commit 6323deb1, and caches workspace crates. Nextest installs through scripts/install-nextest.sh with a SHA-256 check. repository-contracts always runs the docs and record tests plus the strict build, deselecting needs_binary modules; the marker guard parses pytestmark as AST and scans for cargo argv shapes, with the fake-fixture exemptions recorded.

The branch's green run is CI 36633279293 (all eight jobs); main confirmed green at 36636929043 on 37951529. The workspace-crates cache showed no material speedup on the first run after the key change (690 s full-features, 839 s release-panic); the honest comparison needs the next run on the same key, and the follow-up sits in the approved 1.0 speed-up issue.

Code review: ACCEPT 2026-09-29 by a fresh reviewer (dispatch 14f4e9e6-7a88-4bcf-9b75-18a44365b073); the follow-up findings (merge-base hole, executable markdown, cargo-driver escapes) were folded on the 1279 branch.

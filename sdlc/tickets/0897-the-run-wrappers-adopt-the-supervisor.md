---
flow: build
priority: 6
deps: ["0686"]
---
# The run-* wrappers adopt the supervisor

Slice three of the 0686 sequence. Five server-starting run-*
wrappers (run-article-semanticscholar-source-search.sh,
run-clingen-erepo-fixture.sh, run-section-outcome-mcp.sh,
run-variant-article-entity-fixture.sh,
run-variant-article-identity-fixture.sh) depend on EXIT traps that
SIGKILL skips. Route them through 0686's supervisor — the same
single implementation, self-supervising in the real exported-owner
path — or land behavioral proof that killing their owner cannot
leave a server alive. Owner-death tests for each; restating tests
authored in earlier 0686 attempts is authorized through design
commits.

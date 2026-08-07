---
base: e7df4915f090d47886659f40f3adbb14ee954232
head: 2c0ebcf4e77879b15001aef19299918bd05eafa1
---
`src/cli/article/tests.rs` is 1,374 lines and has become four distinct test suites in one flat sidecar: help/parse, exact-lookup/suggestions, JSON integration, and filter/ranking checks. The article runtime is already subdivided, but the test ownership is not, which keeps one test file over the cap and makes article regressions harder to localize.

Imported from March ticket 324. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/324-split-article-cli-tests-rs-into-domain-sidecars-under-src-cli-article-tests

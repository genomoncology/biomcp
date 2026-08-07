---
base: 982eeaec425bbfb34b51d1c5e9044cb2d788a0f5
head: f503b5c22c53916a53a1c6374cfff2165a85a56c
---
After CLI payloads and runtime dispatch move into family modules, the final slice is test relocation and bringing `src/cli/mod.rs` under the 700-line cap. 5,000+ lines of tests currently live inline in mod.rs; they need to move next to the code they exercise.

Imported from March ticket 185. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/185-move-cli-tests-to-sidecars-and-finalize-mod-rs-facade

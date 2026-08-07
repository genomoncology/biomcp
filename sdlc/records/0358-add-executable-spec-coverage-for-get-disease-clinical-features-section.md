---
base: 3b7f34cd27cc01261fd43b9fecd51b7633d0abd3
head: 287d9819d2618a8b4311bac62f77e84a2fb9e692
---
`get disease <name_or_id> clinical_features` is documented as a public CLI section in the v0.8.22 candidate (help text, SKILL.md, cli-reference) and has focused Rust/render/source unit tests, but no `spec/entity/disease.md` canary exercises the public CLI surface. Architecture review for ticket 348 identified this as a doc-vs-spec drift: the diagnostic entity now has full executable spec coverage in `spec/entity/diagnostic.md`, but `clinical_features` does not, and the architecture's "Help, list, docs, and specs are one CLI contract" invariant is not enforced for this section.

Imported from March ticket 358. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/358-add-executable-spec-coverage-for-get-disease-clinical-features-section

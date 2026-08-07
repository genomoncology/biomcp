---
flow: build
priority: 5
---
# Add executable spec coverage for get disease clinical_features section

`get disease <name_or_id> clinical_features` is documented as a public CLI section in the v0.8.22 candidate (help text, SKILL.md, cli-reference) and has focused Rust/render/source unit tests, but no `spec/entity/disease.md` canary exercises the public CLI surface. Architecture review for ticket 348 identified this as a doc-vs-spec drift: the diagnostic entity now has full executable spec coverage in `spec/entity/diagnostic.md`, but `clinical_features` does not, and the architecture's "Help, list, docs, and specs are one CLI contract" invariant is not enforced for this section.

Completed under March on 2026-04-30, as March ticket 358. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/358-add-executable-spec-coverage-for-get-disease-clinical-features-section

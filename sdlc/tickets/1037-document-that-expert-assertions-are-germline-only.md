---
flow: build
priority: 5
---
# Document that expert assertions are germline only

The ClinGen Evidence Repository holds germline variant interpretations made under the ACMG/AMP framework. Somatic tumour variant interpretation uses a different framework entirely, with its own tiers, and lives in different resources. Nothing in `variant erepo`'s help, its documentation, or its empty result says this.

The consequence is a silent wrong answer for a whole class of user. Someone working a somatic oncology case points the command at a tumour variant, gets "No expert assertions were returned," and concludes no expert opinion exists. The command is behaving correctly and the conclusion is wrong, because the question was outside what this source covers.

This is the same failure shape as reporting zero for a source that was never reached: a true statement that leads a reader somewhere false. The fix is words, not behaviour.

## Done when

- The command's help states that the source covers germline interpretations.
- The public documentation for the ClinGen commands states the same, and points a somatic question at CIViC, which BioMCP already serves through `get gene <symbol> civic` and its variant equivalents. Name that as the destination; do not invent or link a source BioMCP does not carry.
- A reader who arrives with a somatic question can tell from the output or the help that they are asking the wrong source, rather than concluding no evidence exists.
- No behaviour changes: this ticket adds no filtering, no detection of somatic intent, and no new source.

## Why this is a build and not a quickfix

This was filed as a quickfix and refused on 2026-08-23: `lint` and `test` both ran green before any change, which is the quickfix flow's grounds for refusal. That refusal was correct. Nothing here is broken in a way an existing assertion can catch — the missing words were never asserted by anything. The proof has to be authored, not reproduced, which is what the build flow's design stage is for.

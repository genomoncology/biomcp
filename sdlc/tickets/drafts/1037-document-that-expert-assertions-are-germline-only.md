---
flow: quickfix
priority: 4
hold: draft for review; do not promote until Ian releases this
---
# Document that expert assertions are germline only

The ClinGen Evidence Repository holds germline variant interpretations made under the ACMG/AMP framework. Somatic tumour variant interpretation uses a different framework entirely, with its own tiers, and lives in different resources. Nothing in `variant erepo`'s help, its documentation, or its empty result says this.

The consequence is a silent wrong answer for a whole class of user. Someone working a somatic oncology case points the command at a tumour variant, gets "No expert assertions were returned," and concludes no expert opinion exists. The command is behaving correctly and the conclusion is wrong, because the question was outside what this source covers.

This is the same failure shape as reporting zero for a source that was never reached: a true statement that leads a reader somewhere false. The fix is words, not behaviour.

## Done when

- The command's help states that the source covers germline interpretations.
- The public documentation for the ClinGen commands states the same, and says where somatic interpretation lives instead.
- A reader who arrives with a somatic question can tell from the output or the help that they are asking the wrong source, rather than concluding no evidence exists.
- No behaviour changes: this ticket adds no filtering, no detection of somatic intent, and no new source.

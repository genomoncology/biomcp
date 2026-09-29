---
base: a7d503be
head: 8bf9884a
---

Routed the last four direct XML parses through the depth cap, from
the 2026-09-27 review file (Ian confirmed the four sites himself).

VAERS, HPA, MedlinePlus, and PMC OA now parse through
`crate::xml::parse_external_xml` with the shared node bound, so the
1243 pre-parse nesting cap covers every external XML source; the
VAERS test helper routes too, leaving no allowlist beyond
`src/xml.rs` itself. Error wording survived the move: three sites
keep their exact `BioMcpError::Api` shape, and the shared
`ExternalXmlError::Parse` display now carries roxmltree's detail;
PMC OA kept its `invalid S3 version listing` prefix and gained the
reason it used to discard. Four nesting-bomb tests drive each
source's real parse entry point and assert the honest "nesting
exceeds 64 levels" error.

The guard is structural, as Ian's feedback demanded: it scans every
tracked `.rs` file under `src/`, resolves use-import aliases
(`use roxmltree::Document as Doc` and the braced form), strips
comments with a real lexer that cannot hide a call inside a string
literal, and has no `#[cfg(test)]` stop. Its self-tests plant a
scratch git tree with a direct call the checker never saw and watch
it fail; a comment that merely mentions the call passes; an
untracked file is ignored.

Red-run proof (the acceptance item): reverting the MedlinePlus site
to `roxmltree::Document::parse(xml)` fails the guard with
`src/sources/medlineplus.rs:185: roxmltree::Document::parse(xml)`
(run 2026-09-27 on the branch worktree; restore greens, 7 passed).
The other three sites are the same diff shape.

Code review ACCEPT with notes: the P1 (record this proof before
land) is discharged by this record. The P2s are recorded for
follow-up: deliberate-evasion spellings exist against the guard
(function-item bindings, qualified-path forms, inline comments in
the path, local type aliases — ticket 1258's class of hardening);
the single-item alias form deserves its own red test (noted for
1258); one `Document::parse` survives in an archived experiment
probe outside `src/`, deliberately outside the guard's scope.

Evidence: one rustfmt cycle caught by the yellow gate's lint phase
(worker edits are static on Blink by rule); re-gated clean at
8bf9884a — lint, test, spec, and stress all OK. `MAX_PACKAGE_FILES`
rose to 1,368 with the guard's test file; the size inventory took
the routed-site deltas.

# Sync current article maintenance into BioMCP 1.0

Status: COMPLETE. Fresh code review accepts64a03c8c46149d7ec8c62b492e359ca26d82ff9b after the quoting correction. Root integrates this result into the dedicated1.0 line.

Merge base: `bddf902d`. Incoming maintenance: `9482051086a825030a68f22ed293c46edd1e8bfc`. The six incoming commits change 42 paths. The variant ticket additions are records only. The source delta carries article entity identifiers, optional passage positions, follow-up commands, and the recorded article-entities specification.

## Retained behavior and changes

BioData records 0144, 0147 and 0152 establish the accepted ordinary publication detail and retained compatibility boundaries. PMID detail still calls PubTator publication_detail. DOI and PMCID still call Europe PMC publication_detail and reuse admitted hints. Original bytes, identity admission, empty results, terminal parser refusals, HTTP fallback, authorship, journal, date, abstract display, batches, and raw and typed MCP surfaces retain their owners.

Both annotation paths use the incoming text-and-identifier grouping, stable count ordering, namespace policy and optional provider locations. Borrowed BioData annotations feed the admitted path. The pinned adapter rejects the newer rsID and HGVS wire fields as unsupported shape. The existing single-record identity-bound compatibility decoder retains those fields. The recorded 30738221 fixture exercises that compatibility path. It earns no new shared-model admission or conformance claim. The existing admitted-versus-compatibility aggregation test now checks identifiers and positions against the later selected record with a decoy first record.

Article entities collects positions only with --full. Other callers pass false. The ordinary detail module keeps its migrated ownership. Its retained child adapts the existing context caller and retains variant work budgets. Direct detail tests receive the new argument. The source package test requires the new spec member and preserves its explicit inclusion and BioData assertions.

The inventory integrates the incoming identity verification, article model and article renderer test entries. It retains the dedicated article-search entry and all other dedicated floors and owners. The historical zero-coupling ledger remains absent. Cargo.toml, Cargo.lock, the common 4f dependency and MyVariant/SnpEff source remain unchanged.

## Checks

The final locked offline test compilation passed in 62.452 seconds. The final locked offline native compilation passed in 31.762 seconds. Both use no default features and two jobs in the free dedicated target. Runtime checks cover the final runtime implementation; final recompilation also includes the context helper's Clippy argument-count annotation.

347 scoped Rust tests passed. All ten article-entities mustmatch checks passed in 26.253 seconds. The prior binary failed eight of those ten checks before integration. Native captured compact/full entities and article annotation commands all passed. Their retained JSON preserves source PMID, title, journal, authors, the existing ten-character date display, annotation source labels, identifiers and positions. The captured fixture equals the incoming bytes.

| Existing Rust owner | Passed | Wall seconds |
| --- | --- | --- |
| pubtator-source | 18 | 6.323 |
| europepmc-source-detail | 8 | 0.032 |
| article-transform | 81 | 0.157 |
| article-detail | 25 | 38.655 |
| article-batch | 4 | 0.009 |
| article-cli | 76 | 0.175 |
| article-render | 23 | 0.020 |
| article-entities-serialization | 17 | 0.009 |
| next-commands | 59 | 0.124 |
| related-render-corrected | 1 | 0.007 |
| metadata-caller-fresh | 1 | 4.024 |
| caller-0 | 1 | 0.042 |
| caller-1 | 1 | 4.156 |
| caller-2 | 1 | 3.056 |
| caller-3 | 1 | 0.140 |
| caller-4 | 1 | 4.094 |
| caller-5 | 1 | 4.068 |
| caller-6 | 1 | 0.085 |
| enrichment | 5 | 0.030 |
| identity-verification | 21 | 0.060 |
| root-source-render | 1 | 0.033 |

The Python package and publication selection initially passed 27 cases and failed two in 30.621 seconds. The temporary-path invocation lacked the repository TMPDIR and base directory. That unchanged existing test passed after correcting both invocation settings in 0.149 seconds. The effective scoped result is 28 passing cases and one inherited failing selector. The publication catalog/schema, rendered annotation exclusions and limits, proof bindings and receipt selectors pass. The output-set selector still fails because the unchanged website generator expects ad621 while the accepted manifest and lock pin 4f. No website expectation or dependency was altered to hide that failure.

The first grouped deadline process passed its constructor case, failed six subsequent cases and stalled in pagination. It was terminated after 180.079 seconds. The inherited global limiter retains its first exact unpaced fixture origin. All eight registrations passed in fresh processes. One initial related-render selector matched no registrations; the corrected existing related-article owner passed. Raw failed attempts and corrected outcomes remain preserved. These are corrective invocations, not a claim of one uninterrupted green run.

Scoped formatting, forward whitespace and the existing source-size check pass. The size check reports no findings. macOS execution uses offline dependency resolution and repository loopback fixtures. It does not establish Linux network namespace isolation. No live provider, paid service, hosted job, broad suite, release, or shared-line merge ran.

## Remaining work

Root must review the final merge candidate before integrating it. The unchanged website generator revision mismatch requires separate disposition. Shared-model admission of newer provider annotation metadata remains outside this merge. The private handoff retains literal commands, full logs, compilation artifacts, raw article fields, original failed attempts and all measured walls.

## Review correction: quote disease follow-up identifiers

Root's fresh review of 35c46d1f found one P2 defect. The disease follow-up formatter inserted the provider identifier directly into the command. It now uses the existing quote_arg helper for both MESH and OMIM. Ordinary identifiers keep their existing command spelling. Provider whitespace, a semicolon and command substitution remain inside one quoted argument.

The existing article-entities renderer owner adds one regression claim for that quoted output. Its previous assertions still cover ordinary MESH, OMIM, gene, drug, rsID and HGVS commands. Its two disease setup rows use a local table; the file remains at its existing 1,335-line inventory ceiling. No test file, proof tool, inventory allowance or dependency changed.

The regression failed before the formatter fix: one failing test, exit 101, wall 1.572 seconds. Red offline compilation passed in 76.243 seconds. Green offline compilation passed in 62.689 seconds. Both compilations use the existing dedicated target, two jobs, no default features and the locked graph. The affected article renderer owner passed all 23 tests in 1.648 seconds. The related-article follow-up owner passed one test in 0.007 seconds. No selected test was ignored. Scoped formatting and forward whitespace pass. These walls include subprocess startup; libtest reported 0.02 seconds and 0.00 seconds for the two successful runtime checks.

Root owns finding closure and dedicated-line landing. The previous broader integration evidence and website revision exception remain as recorded. The later eight-commit maintenance batch through ddc631fd is outside this correction.

Review: accept at64a03c8c46149d7ec8c62b492e359ca26d82ff9b.
Landed:64a03c8c46149d7ec8c62b492e359ca26d82ff9b.
The later variant-input maintenance batch remains separate; this landing includes main through94820510.

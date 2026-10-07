# Adopt the shared InterPro domain response

Status: CODE qualified. Root started fresh Medium CODE review. Root owns dedicated 1.0 landing, push and paired closure. The builder owns only consumer2027. Another agent owns cleanup.

## Authority and exact pair

Ian released BUILD against accepted consumer design `475bcfb20eb468059daea5ff8dd0e53769f1d18f` and paired producer design `e031a07cc72091e6cfe519f71497321048ef692b`. The builder recorded that release and ownership in `47d00c22` before implementation. Merge `5f77215a` integrates dedicated target `b46064c42cf94a9b4142f0922a49e0893d0a127d` without rebasing. The target carries newer maintenance records only. No maintenance product code was edited.

The checked consumer candidate is `ef17e572ec5f5f812e71e46799bdd191c9b5a3b5`. Cargo manifest, lock and existing exact-pin checks use producer LAND `29e01881b24604a3d0ea079b79b0e60c2f506964`, BioData0.0.42. Root reports Sartre accepted corrected producer CODE `61bb5da2de09ee9652a968a067c3062055cefc87`. Its six public cases, lint and spec passed. The original whole test receipt applies only to `ec60c80ae1b4271d153253ab53fe2e31d11d1d93`. This consumer record claims no later producer whole test run.

Early runtime candidate `51c81ac0` was handed to Root during checks. Successor `7729114c` strengthens retry and provenance assertions, fixes an authored fixture and registers seven affected Rust owners. Candidate `ef17e572` retires the old parser claim and asserts exact ordered Markdown rows. Runtime code is unchanged across those successors. The final receipt commit changes only this record and the owning ticket. Root observed the checked retirement candidate and started fresh Medium review. The builder claims no independent CODE acceptance or landing.

## Product change

The InterPro client admits bounded original HTTP bytes through `biodata::InterProResponse::decode_json`. It returns the admitted shared response. Protein borrows `domains(20)` and allocates only its final output fields. Structure borrows `domains(25)` and chooses the first complete range that inclusively contains the selected residue. The application retains its distinct final output objects, request caps, overlap selection, source attribution and section outcomes. No serialized bridge, mirrored source graph or local parser survives.

Request plans, accession validation, provider base override, shared client, cache mode, transport limits, timeout policies and HTTP error handling retain their existing implementations. Shared structural failures map to the static application message `Invalid InterPro response.` with InterPro retry context. This removes source values from structural error Display, Debug and serialized error output. Existing non-success HTTP excerpts retain their established transport policy.

## Behavior proof and retirement

Three new tests use existing local fixture support and the existing executable/MCP harness. Protein runs through native and restored output, CLI JSON/Markdown and typed/raw get MCP. Structure runs through native and restored output, CLI JSON/Markdown and raw structure MCP. They assert complete ordered domain vectors and Markdown rows, omitted optional labels, InterPro source credit, section outcomes, retained protein and PDB fields, selected residue, additional dbNSFP position warning and recovery after serialization.

Authored original InterPro bytes contain ignored `1e400`, a blank first accession, metadata-less rows within the page cap, a distinguishable overflow row, repeated accessions, incomplete fragments, two overlapping ranges and an inclusive single-residue neighbor. Protein keeps the first 20 source rows before filtering. Structure keeps the first 25 and chooses 457..717 before 590..610 for residue600. The separate 600..600 row survives; a range ending599 does not overlap. The caller tests exercise source propagation and application policy. They do not replay the producer admission matrix.

HTTP failure through protein preserves its UniProt identity and reports domains unavailable with no successful InterPro credit. Malformed successful bytes through structure preserve protein, structures and residue context while reporting domains unavailable. Structure also proves healthy empty and no selected residue. The latter makes no InterPro request and reports inapplicable. Existing request construction, outcome matrices, Markdown failure ownership and shared body/chunk limits remain in place.

After the real caller checks passed, the source graph and mapper claims retired: `InterProResponse`, `InterProResult`, `InterProMetadata`, `InterProProtein`, `InterProLocation`, `InterProFragment`, `InterProDomain`, `InterProRange`, `decode_domains_response` and `interpro_ranges`. The source boundary ratchet prohibits the displaced type declarations. The source parsing module registration and its sole `decode_domains_response_maps_rows_and_skips_blank_accessions` test retired. Its file now records ownership without declaring tests. Producer0715 owns admission, trimmed views, incomplete fragment retention and source range ordering. The real consumer cases own propagation. No filesystem path was deleted. Existing construction, outcome and renderer owners retain their independent claims.

## Exact checks and costs

Host: M5 `imaurer-m5`, Darwin arm64, 18 logical CPUs, Rust/Cargo1.95.0. The worktree began with no target artifacts. Red compilation filled its target using existing host dependency and compiler caches. Cargo then resolved the exact producer offline through Root's existing host Git database `biodata-f1448eb14ff60aee`. The declared HTTPS Git URL and full immutable revision remain intact. The lock update changed only the BioData package from0.0.41 to0.0.42. No path patch, URL override, source acquisition or new cache mechanism ran. Later compilation reused target and dependency artifacts. Observed one-minute load was3.23 initially,5.93 during qualification and2.95 at final observation. These ordinary costs establish no benchmark comparison.

| Check | Result | Observed wall seconds |
| --- | --- | --- |
| Old decoder privacy regression | Expected failure | 44.18 |
| Exact-pin offline binary build | PASS | 27.90 |
| Initial affected test compilation, including artifact-lock wait | PASS | 73.50 |
| First caller batch | Five pass, authored structure fixture corrected | 33.16 |
| Corrected test compilation | PASS | 46.36 |
| Corrected 12-owner batch | PASS | 30.96 |
| Final test compilation at ef17e572 | PASS | 45.03 |
| Final 12-owner batch at ef17e572 | PASS | 27.86 |
| Final Python boundary/package/workflow batch | 147 pass, one explicit Darwin deselection | 34.81 |
| Four affected ratchet audits | PASS | 33.65 |
| Strict library Clippy | 12 inherited failures | 52.04 |
| Tracked-text scan | PASS | 1.64 |

The final Rust test executable discovers3974 library tests. It runs the `interpro` filter plus the existing domain outcome matrix, structure failure renderer and four shared body/chunk owners in one invocation. All12 passed in26.37 test seconds. `BIOMCP_BIN` points to the exact-pin binary built from the unchanged runtime. Final compilation after the last test-only changes used locked offline Cargo and no default features. The final Python batch ran the existing three boundary/package/workflow files with offline dependencies and routine build output as its temporary directory. It passed in34.70 test seconds. The Linux `/proc` namespace test is explicitly deselected on Darwin.

The exact-pin boundary passed. The four affected audits cover Rust source size, source states, remote resource bounds and external error logging. All report no findings; source size inspects862 Rust files. Changed Rust files pass rustfmt. Ruff0.13.2 passes the pin checker and the two pin/package test files. The focused selection now contains652 Rust owners and15 Python selections. Its existing registration test passed after the count update. Tracked-text and whitespace checks pass. No source-size allowance or gate ceiling increased.

## Attempts and inherited limits

The red regression exposed the original decoder's private fragment-start marker. Its test execution took0.09 seconds. An earlier draft used an invalid map that produced an already-private diagnostic and passed. That draft supplies no red credit. Initial test compilation caught a private sibling-module reference and a nonexistent error helper; both wiring issues were corrected using existing interfaces. The first caller batch passed protein and private errors but failed the alternate-residue assertion. The authored fixture had placed the alternate position in snpEff, while the unchanged structure warning reads dbNSFP aliases. Supplying both authored dbNSFP aliases corrected the fixture. No runtime change was needed.

The Python build-identity wrapper first used the system interpreter without tomllib and failed before compilation in0.20 wall seconds. Plain Cargo built the exact-pin binary successfully. No wrapper change was made. Final test and Python checks use existing offline inputs and loopback fixtures. They claim no Linux namespace isolation, live-provider admission, complete release qualification or whole deprecated maintenance gate.

Strict library Clippy reproduced the same12 inherited findings documented in record2025: unused members in entity, drug and trial owners; the shell fold; three genomic-assertion suggestions; an exact-scan borrow; and the OpenCitations loop. None belongs to changed runtime files. Workflow Ruff reproduces inherited E731 at line258; this ticket changes only its selection count. Record2025 retains the inherited repository-wide formatting differences. The minimal cached pytest environment retains its unknown asyncio_mode warning. These findings remain separate from successful affected qualification. No unrelated code or whole release gate was changed or run.

## Delivery and handoff

The first SSH443 push stopped at host-key verification. Reusing GitHub's existing host-key alias succeeded. Commits `47d00c22`, `51c81ac0`, `7729114c` and `ef17e572` are pushed normally on `ticket/2027-adopt-interpro-domain-response` with `[skip ci]`. GitHub recovered during the build; Ian reports the exact producer main and dedicated consumer target are also pushed. No server500 blocked this builder's final pushes.

Local logs use the `biomcp-2027-` prefix. They retain red, pin resolution, binary build, each test compilation and caller run, final Python, ratchet, Clippy, Ruff and tracked-text outputs. Build artifacts remain in the worktree target. The cleanup owner receives all retained output. No containers, hosted CI, provider requests, production reads, Pi jobs, delegates, shared programme edits, maintenance product edits, source acquisition or deletion ran.

The next action belongs to Root: consume fresh Medium review, resolve any findings through the builder, land the reviewed exact pair on the dedicated1.0 line and record paired closure. Ticket2027 remains OPEN until that action completes.

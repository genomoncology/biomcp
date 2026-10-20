# Fulltext license metadata

Before production edits, the six-row real CLI regression reproduced the lost fact on current-main runtime. Independent binary96a28b71189af8dd651ae97687de6968c63f610b7ddba917013405516208ea79 was built from e3db5a01a42add2a63cc30ab72104c3cf1d60cee. The complete src/Cargo/.cargo diff between that source and current main22255931e6e23b9087ac40f688d3121e31fb1af7 is empty. Only the new test overlay was copied into the published0cb3a1aac checkout; production code remained unchanged.

Exact command through pm job: `BIOMCP_BIN=/Users/ian/workspace/worktrees/biomcp-2236-job/.cache/2236/biomcp-cpic-repair /Users/ian/workspace/worktrees/biomcp-2236-job/.venv/bin/python -m pytest -q tests/test_article_fulltext_license.py`.

Job2296/1792516829766-66209 returned four failures and two passes in21.83seconds. Both stated CC BY and stated CC BY-NC-ND lost the license when the package was available and when it failed. Actual CLI output included license_present:false and the existing unknown-rights warning. Both absent-license controls passed, and every request was replayed. The first setup attempt lacked pytest; the next exposed a missing authored supplementary503 route. Those failures are retained separately and are not the clean regression result.

Private experiment2102/fulltext-license/2296/red retains all three original jobs and their exact outputs/state/results, alongside the unchanged observed metadata and authored control variations. The planned narrow repair is assets.rs plus this outside-in table and the affected fulltext output. It must use the existing nonempty stated license and PMC OA provider attribution, fill only unknown reuse, and retain the fact even if the later package download fails. No source permission or new licensing classification is introduced. The completed earlier coverage suite is not rerun.

## Structured license fields before JATS edits

The earlier absent-license rows meant absent PMC archive metadata, not absence from every fetched source: the retained JATS body also contains a license xlink href and an ALI license_ref. Those rows are not evidence for an all-source absence rule. The corrected authored absence controls remove the structured elements; the normal observed positive fixture remains unchanged.

Clean pre-JATS red job2296/1792517165158-70305 used the same unchanged current-main runtime, published0cb and a tests-only overlay with the same pytest command. It returned10failures and2passes in42.77seconds, with all requests replayed. JATS-only href and authored ALI-only controls lose license_present; known Europe PMC metadata keeps its restricted license but lacks license-source attribution. Both corrected all-source-absent controls pass. This is the preimage proof for the structured extractor/propagation paths named in the ticket.

Intermediate PMC-only source826a07c3d passed its six CLI rows and both affected golden formats in job1792516953164-90758, with formatting passed. Canonical Clippy then found two inherited fixture-wait annotations missing from2294, not a license implementation failure. The2294 reviewer corrected exactly those scoped test annotations at148474f344624686242eb8bc7ba70d314c9900d9; the coordinator merged that accepted mechanical fix here before final checks. The intermediate eight passes remain bounded PMC evidence, not final all-source proof. All jobs remain under experiment2102/fulltext-license/2296, including pmc-only and original red folders.

The existing JSON reuse fields carry the literal license and source attribution. The current Markdown renderer omits that manifest; its body contract remains unchanged, with the affected golden comparison retained. No new rendering surface is required for this observed field defect.

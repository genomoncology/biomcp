---
flow: build
priority: 3
---

# 2164 — Prepare the stable release metadata

Status: OPEN.

Milestone: 0.9.3

Reviews: revision 1dff5f3, accept

Reviews: revision 51940b5, accept

## Outcome

The final candidate consistently identifies itself as 0.9.3 across its published metadata before final measurements and the untagged release rehearsal.

Depends on: 2161

- Starts from: the existing version synchronization and release-version checks.
- Keeps: the existing formula version placeholder, release publication ownership and every product behavior. Create no tag or publication.
- Changes: claims `Cargo.toml` `Cargo.lock` `pyproject.toml` `uv.lock` `manifest.json` `CITATION.cff` `server.json` `CHANGELOG.md` `tests/test_citation_contract.py` `tests/test_release_metadata_contract.py` `tests/test_docs_changelog_refresh.py` `tests/test_version_sync_script.py`. Keep metadata contracts valid in development and stable states; explicit development fixtures must initialize that state rather than assume the current checkout is a development release. Retain the historical release inventory.
- Proof: before editing, show the current mismatch through the existing version checks in a build job. Set the package versions and lock entries to the stable release; update both server version fields, manifest, citation and top changelog release heading. Use the preparation date in dated metadata. Run `bash scripts/check-version-sync.sh`, `python3 scripts/check-release-versions.py --tag v0.9.3` and the revision-based changelog check added by 2161 on the committed candidate through a build job. No scratch tag is needed. The ordinary merged-candidate and actual-main checks still apply.
- Defers: Ian creates the release tag after the final measurements and full nonpublishing rehearsal pass.

Coordinator decision: stable metadata preparation is necessary for the approved exact-candidate rehearsal. Run 2082 after this ticket, using both product revisions and one corrected measurement harness. The formula remains a template; this ticket does not publish it.

Current-main integration review accepted 51940b580. Exact published candidate qualification 1792257014937-42177 passed version synchronization, stable v0.9.3 validation, current changelog coverage for all 84 tickets since v0.9.2 and 48 cases across the four affected release-aware modules. Preserve the formula placeholder. No tag or publication occurred. The combined candidate still receives its ordinary landing gate and actual-main full check.

Shared candidate gate 1792258000386-53996 failed the release-process version facts in the CLI surface quality check after stable metadata changed. Correct the release guide version facts to match the committed stable package versions and run the affected release-version contract plus quality checks. Preserve all release qualification and rehearsal behavior.

The fresh reviewer accepts documentation correction df2a55d9855f7e9b1a9e8b10e8de06ba068633e6. Job 1792258153963-71239 passed 24 release-version and CLI surface contract tests, stable version validation, version synchronization and the full quality ratchet on the published candidate plus the exact documentation patch. Qualification and rehearsal behavior remain unchanged.

Shared gate 1792258265870-81665 passed lint and Rust tests, then failed one stale directory submission assertion that hard-codes the development package version; 1238 Python cases passed and nine skipped. Correct the directory bundle contract for the prepared stable metadata, retaining meaningful package/manifest identity checks, and check the remaining release metadata assertions for this same stale-development-version class before the next gate.

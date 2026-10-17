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

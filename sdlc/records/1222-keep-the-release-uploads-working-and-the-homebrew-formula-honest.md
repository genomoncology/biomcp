---
base: 3b0c8fcf
head: a172d49a
---

Retired the archived release-asset action and closed the branch-named formula
path.

`actions/upload-release-asset@v1` read `github.event.release.upload_url`, which
is empty on `workflow_dispatch`, so a manual run failed at the asset upload.
Both upload steps are now one `gh release upload "$TAG" ... --clobber` step
guarded on `github.event_name == 'release'`, with `shell: bash` because the
Windows leg defaults to PowerShell. The build job resolves
`TAG: ${{ github.event.release.tag_name || inputs.tag }}`, and `homebrew-tap`
uses that one value for the checkout ref, the checksum download,
`VERSION="${TAG#v}"`, and the commit message; no `GITHUB_REF_NAME` remains in
the job. The runbook now records that a full dispatch proceeds past the upload,
fails at `pypi-publish` on the duplicate version, and still runs `homebrew-tap`
and `container-publish`, so a dispatch naming an older tag rewrites the public
formula backwards and republishes that tag's image.

Evidence: actionlint 1.7.12 clean over all workflows; the provenance suite grew
from 8 to 11 tests, each mutation-checked (reintroduced action, dropped guard,
dropped shell, branch-named version, dropped `GH_TOKEN`, dropped `TAG` source);
`QUALITY_RATCHET_AUDITS=cli_surface_contract` passes; `make lint` on yellow at
a172d49a passes.

Reviews: the design review rejected the first draft and the ticket was split
into 1222, 1226, 1227, and 1228; the code review found one P1 and one P2, both
fixed and verified closed.

Residual: the upload path itself is only exercised by a real release, and a
full dispatch for an older tag is now possible, which the runbook warns about.

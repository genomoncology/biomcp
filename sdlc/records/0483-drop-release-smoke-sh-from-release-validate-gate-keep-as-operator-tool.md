---
base: 2043b6551ef09f27ba32d2db4af8316a9300ded4
head: 6b7cce5ed9ba2a456f2498cda51c662d0c016cdc
---
The `Release` workflow's `validate` job blocks publishing on `scripts/release-smoke.sh`, but that script self-documents as *"an operator/release check with live calls; it is not part of make spec."* It makes **live ClinicalTrials.gov calls** — the `438 gene trials`/`438 disease trials` checks run `biomcp gene trials BRAF --limit 1` / `disease trials melanoma --limit 1` and assert the live response contains `Results: 1` within 45s. Live upstream results are variable, so these fail intermittently on CI runners (other live checks in the same script, e.g. alias and HGVS resolution, passed on the same run — so it is not a no-network problem; the trials endpoint is simply slow/variable). The script also hardcodes the release version (`444 --version reports 0.8.24`), so it goes red on any version it was not hand-updated for.

Imported from March ticket 483. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/483-drop-release-smoke-sh-from-release-validate-gate-keep-as-operator-tool

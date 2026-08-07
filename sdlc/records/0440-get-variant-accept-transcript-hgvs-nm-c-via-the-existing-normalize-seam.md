---
base: 1b8556c51500d9c9f3caf9e1358e0ad64f148a0e
head: af8ec940f90e2a1682d0b16a924c9a12ddb037f4
---
Route transcript HGVS (NM_:c.) through the existing Mutalyzer normalize seam in get variant instead of rejecting it; variant normalize already accepts it. Verified high-severity bug on 0.8.24.

Imported from March ticket 440. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/440-get-variant-accept-transcript-hgvs-nm-c-via-the-existing-normalize-seam

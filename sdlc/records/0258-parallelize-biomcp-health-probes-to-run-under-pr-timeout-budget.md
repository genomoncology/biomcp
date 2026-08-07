---
base: 10410a693c3d138796f6c930172fd7f47677c32b
head: 104572447265062c3aa03c2d6a030250bc81fb6d
---
`biomcp health` probes 52 external services serially. A fully healthy run takes ~20s cold; when even one upstream retries or hangs on a socket timeout, wall time jumps to 120-130s. Serial dispatch means one slow probe dominates the total rather than running alongside the others. Cold-CI `spec-pr` runs regularly blow the 60-second per-heading timeout because of this retry amplification, and operators running `biomcp health` interactively wait two minutes for what should be a fast diagnostic.

Imported from March ticket 258. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/258-parallelize-biomcp-health-probes-to-run-under-pr-timeout-budget

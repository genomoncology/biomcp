---
base: 262fbad739c4ccd26b4aa8f6ed8d25ec1a1086be
head: 79326eedd60c249b55f04fd324080e560e96a78a
---
An issue report (436) describes a command that prints the expected answer but does not exit within a practical timeout: `get disease ... survival`. For agents, producing the answer is not enough; the process must terminate promptly or callers waste turns and hit wrapper timeouts.

Imported from March ticket 467. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/467-bound-cli-command-exit-after-output-for-disease-survival-and-ctgov-helpers

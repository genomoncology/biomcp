---
base: 7c6f63a9ba26877e038f51ac39adb8ea51dd6b7f
head: e8eab1ed7c6e0995c1c10a3bd69e35e16a2c3d8d
---
`biomcp --json health` reports `healthy`, `warning`, `excluded`, and `total`, but omits the number of sources in `error`. During the audit it returned 54 healthy, 2 warning, 1 excluded, and 58 total while one row was visibly in error. JSON consumers must infer the missing category by subtraction, even though Markdown already prints the error count.

Imported from March ticket 498. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/498-add-the-missing-error-count-to-health-json

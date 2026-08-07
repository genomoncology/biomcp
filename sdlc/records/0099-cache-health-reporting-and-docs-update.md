---
base: af19e2c340bf0598d8ace10cd90eb0385d8962f2
head: dd3632ba0ec0c4d916b1cb6eb2a35f6b763012f8
---
After the runtime paths and migration helper land (T101, T102), shipped operator docs and example output still hardcode the old `http-cacache/` directory name and `/tmp/biomcp/` download location. This ticket refreshes all user-facing documentation and executable spec references to match the settled runtime contract.

Imported from March ticket 099. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/099-cache-health-reporting-and-docs-update

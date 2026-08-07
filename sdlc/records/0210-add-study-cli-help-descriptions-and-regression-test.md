---
base: e3d17ce46ea21d19506213f1d2f1a1150511f7a4
head: 546278c6bfd6722605b3125246afcfa692058b08
---
`biomcp study --help` lists nine subcommands with blank description strings. Individual subcommands (`study query`, `study filter`, `study compare`, etc.) also have blank descriptions for most flags. Every other command family in the CLI provides sentence descriptions at this level. The `biomcp list study` reference page has thorough descriptions for the same commands, so the information exists but is not surfaced where users look first.

Imported from March ticket 210. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/210-add-study-cli-help-descriptions-and-regression-test

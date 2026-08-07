---
base: 1b5b07b38de71488b851f1d9112b96ef648c35f2
head: 51ee24b670527cf08ba80b1d0b38083d721f7f8d
---
`search variant -g PLN` shows HGVS notation only (p.L39X, p.R25C). BioASQ gold uses legacy notation (PLN L39stop, PLN -42 C>G, Arg(9) to Cys). Both refer to the same variants. Agents can't match the formats.

Imported from March ticket 085. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/085-show-variant-legacy-nomenclature-alongside-hgvs

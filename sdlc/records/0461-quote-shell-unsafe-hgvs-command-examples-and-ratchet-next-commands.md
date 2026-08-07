---
base: 4207030c5701e73fb098254aee41f18864aa0620
head: 856e3bbccb5715882b0a00161b729582c2f13a85
---
Several public command examples contain transcript HGVS values with `>` unquoted. In a normal shell, `>` redirects stdout, so copying an example like `NM_004333.6:c.1799T>A` can truncate the CLI argument and create or overwrite a local file. This is both usability and safety: command examples must be copy-pasteable or clearly non-shell text.

Imported from March ticket 461. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/461-quote-shell-unsafe-hgvs-command-examples-and-ratchet-next-commands

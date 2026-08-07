---
flow: quickfix
priority: 8
---
# Quote shell-unsafe HGVS command examples and ratchet next commands

Several public command examples contain transcript HGVS values with `>` unquoted. In a normal shell, `>` redirects stdout, so copying an example like `NM_004333.6:c.1799T>A` can truncate the CLI argument and create or overwrite a local file. This is both usability and safety: command examples must be copy-pasteable or clearly non-shell text.

Completed under March on 2026-06-30, as March ticket 461. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/461-quote-shell-unsafe-hgvs-command-examples-and-ratchet-next-commands

---
base: ae27d96d
head: e5d88973
---

Combined Markdown drug search now places an exact, region-selecting continuation
under each US, EU, or WHO section that has another page, using the same command
builder as JSON. Generated GWAS help and the user guide no longer advertise the
unsupported region argument.

The allowed-Host boundary now wraps the whole HTTP router, so `/`, `/health`,
`/readyz`, and `/mcp` share the same policy; the explicit unsafe override still
allows arbitrary valid Host values. Canonical lint bootstraps the repository's
checksum-verified ShellCheck and actionlint versions on supported hosts, while
direct missing-tool failures are concise and actionable. The pre-commit
installer's read-only `--check` mode reports current, missing, and stale states
without modifying a hook.

Focused Rust and Python tests covered regional continuation placement, GWAS
help, authority normalization, every HTTP route and both policy modes, missing
lint tools without a traceback, and non-mutating hook checks. The current hook
was installed in this checkout and its check mode passes.

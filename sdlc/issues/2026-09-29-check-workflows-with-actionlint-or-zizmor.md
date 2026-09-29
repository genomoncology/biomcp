# Check workflows with actionlint or zizmor

Filed 2026-09-29 as part of the 1.0 move (see
2026-09-29-replace-text-scanning-checks-with-compiler-tools.md for
the umbrella). The workflow contract's text assertions walk YAML by
hand; a maintained linter (actionlint for schema and expression
errors, zizmor for security shapes like BASH_ENV and script
injection) covers the same ground with real parsers.

Owner: the developer agent on Ian's queue. Trigger: the 1.0
feature-track start.

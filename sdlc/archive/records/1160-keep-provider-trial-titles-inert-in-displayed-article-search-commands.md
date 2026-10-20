---
flow: build
priority: 1
---

# Keep provider trial titles inert in displayed article-search commands

Completed and terminated trial cards print an article-search command from the NCT identifier and the first six whitespace-delimited title words. `trial_results_search_command` in `src/render/markdown/related/article_support.rs` wraps that provider-derived seed in double quotes and escapes only literal double quotes. A title can therefore leave shell expansion active inside the displayed command.

This synthetic hostile title fragment demonstrates the defect:

```text
Alpha\path's $(touch /tmp/biomcp-trial-title-expanded) "quoted" $HOME; `uname` tail
```

The current renderer prints a command whose copied shell form expands the command substitution, environment variable, and backticks. I reproduced Bash changing the intended literal query and executing the controlled `touch` substitution. A backslash before `p` remains literal inside the current double quotes. The backslash stays in the regression because the shared helper escapes it and shell parsing reconstructs the same single literal backslash. The same command reaches trial Markdown, JSON `_meta.next_commands`, paginated JSON, batch output, and typed or raw MCP responses because those surfaces share this renderer.

BioMCP already has the required quoting behavior. `force_quote_arg` in `src/render/markdown/support.rs` always uses double quotes and escapes backslash, double quote, dollar sign, and backtick. The intervention, condition, and alias command paths already use shared shell-safe quoting. This ticket routes the complete trial-results seed through `force_quote_arg` instead of maintaining a second partial quoting rule.

## Correct behavior

A provider trial title remains one literal query argument when a user copies the displayed article-search command into a POSIX-style shell. Shell syntax inside the title never expands or executes. The command still includes the NCT identifier followed by at most six whitespace-delimited title words. A blank title still yields the quoted NCT identifier alone.

Ordinary output remains byte-for-byte unchanged. In particular, the existing NCT02576665 command and the blank-title command retain their exact text. Markdown and JSON continue to expose the same command. Command order and duplicate removal remain unchanged.

For a completed trial with NCT identifier `NCT35700001`, first intervention `SAFE-357`, and the title above, the exact command is:

```text
biomcp search article --drug SAFE-357 -q "NCT35700001 Alpha\\path's \$(touch /tmp/biomcp-trial-title-expanded) \"quoted\" \$HOME; \`uname\`" --limit 5
```

Shell parsing reconstructs this exact query argument without expansion:

```text
NCT35700001 Alpha\path's $(touch /tmp/biomcp-trial-title-expanded) "quoted" $HOME; `uname`
```

## Done, observably

- A focused renderer test starts red on the current hand-written quote and passes after the production path uses `force_quote_arg`.
- The focused test asserts the six-title-token cap, the exact escaped command, `shlex::split` argument recovery, and acceptance by the production Clap parser.
- Exact assertions preserve the existing ordinary completed-trial and blank-title commands.
- The fixture-backed trial specification updates the existing synthetic `SHELL_SAFE_STUDY` dictionary in the deterministic local ClinicalTrials.gov fixture server. It changes the title to the exact seven-token hostile title above and changes status from `RECRUITING` to `COMPLETED`. It retains the existing condition, alias, and `SAFE-357` intervention fields so their prior shell-safety contract remains intact. This input is synthetic and has no capture receipt. The specification extracts the displayed publication command from Markdown and JSON and proves both surfaces agree.
- The specification sends only that fixed synthetic command to a child shell. It removes `/tmp/biomcp-trial-title-expanded` first, installs an `EXIT` cleanup trap, and sets `HOME` to a fixed sentinel. It runs `eval "set -- $command"`, asserts an argument count of nine, and compares positional argument seven with a separately assigned, non-evaluated fixed literal containing the NCT identifier and six title tokens. It then proves the marker file was never created. The red test deliberately allows the controlled pre-fix `touch` and `uname` substitutions to run inside the child shell. It never executes the parsed `biomcp search article` command.
- A focused Rust test or another existing fixture proves that recruiting and other non-completed trials still omit the results-search command under the existing status policy. `SHELL_SAFE_STUDY` no longer supplies that proof after this ticket changes its status to `COMPLETED`.
- Existing printed-command parsing and cross-surface agreement contracts remain green.
- `make lint`, `make test`, and `make spec` pass.

## Boundary

Change only `src/render/markdown/related/article_support.rs`, its related trial test sidecar under `src/render/markdown/related/tests/`, `spec/fixtures/setup-ctgov-intervention-alias-spec-fixture.sh`, and `spec/entity/trial.md`.

Do not change trial parsing, provider requests, source records, article search behavior, command grammar, generic duplicate removal, MCP code, status policy, title-token count, or any ordinary command text. Do not add a new quoting helper. Do not execute a displayed BioMCP command in the specification.

## Evidence

`src/render/markdown/related/article_support.rs` contains the incomplete escaping rule. `src/render/markdown/support.rs` contains the existing complete helper. The shared trial renderer and dispatch paths establish that the same generated string reaches Markdown, JSON, pagination, batch, and MCP surfaces. Ticket 1070 proves parser syntax only, and ticket 1071 proves cross-surface agreement. Neither contract evaluates shell expansion. No active, draft, archived ticket, or issue owns this defect.

## Result

The shared results-search renderer now quotes the complete provider-derived query through `force_quote_arg`. The regression test and executable specification preserve the hostile title as one literal argument, enforce the six-title-token cap, validate the production command parser, and prove that command substitution, environment expansion, and backticks remain inert. Existing completed, blank-title, and recruiting behavior remains unchanged.

`make lint`, `make test`, and `make spec` passed after independent code review.

## Review

- Design review: accepted after revalidation against the ticket 1141 landing.
- Code review: accepted with no findings.

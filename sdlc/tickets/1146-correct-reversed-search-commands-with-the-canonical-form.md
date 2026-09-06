---
flow: build
priority: 3
deps: []
---

# Correct reversed search commands with the canonical form

## Goal

When a caller reverses BioMCP's search grammar, report one copyable canonical
command instead of Clap's unrelated similarity suggestion. For example,
`biomcp article search` currently suggests `article batch`, and
`biomcp trial search` suggests `article`; both must point to `biomcp search
article` or `biomcp search trial`. The canonical grammar remains
`biomcp search <entity>`: the reversed form is never accepted or executed.

The original observation is preserved at commit `fe2f9fc1` in
`sdlc/issues/reversed-search-grammar-suggests-an-unrelated-command.md`.

## Exact detector

Own the recovery hook beside `build_cli` and `try_parse_cli` in
`src/cli/shared.rs`. It applies only after the original argv fails ordinary
Clap parsing with an error other than `DisplayHelp` or `DisplayVersion`.

The searchable-name allowlist is exactly the fifteen `SearchEntity` spellings:

```text
all author gene disease diagnostic pgx phenotype gwas article trial variant drug pathway protein adverse-event
```

Starting after argv[0], find the first two command-position tokens before a
literal `--`. Ignore only the exact global boolean tokens `--json`, `-j`, and
`--no-cache` while finding those positions. Match only when the two tokens are
an allowlisted lowercase name followed by the exact lowercase token `search`.
Do not case-fold, prefix-match, normalize hyphens, inspect option values, or
interpret anything after `--` as command syntax.

Build the candidate by swapping only those two argv elements. Preserve every
other argument byte, spelling, position, duplicate, and the `--` delimiter.
Validate the candidate with a fresh raw `build_cli().try_get_matches_from(...)`
call, never `try_parse_cli`; this prevents recursive recovery. An `Ok` result
or candidate `DisplayHelp`/`DisplayVersion` is structurally valid. Any other
candidate error returns the original Clap error unchanged, so a reversal with
invalid search arguments does not replace a more relevant diagnostic.

The original parser result always wins for genuine help/version requests.
Thus root or family help/version that Clap already recognizes remains byte-for-
byte unchanged. A trailing help/version flag after the reversed pair is merely
preserved in the proposed command: `biomcp article search --help` is still an
exit-2 correction to `biomcp search article --help`, not an alias that displays
help. Global JSON selection recognizes `--json` or `-j` only before `--` for
this correction; a token after `--` is candidate data and cannot change the
error envelope.

The classifier examines at most 256 argv entries and 16 KiB of UTF-8 argument
data, and emits at most 32 KiB. Non-UTF-8 input, C0/C1 controls, or a breached
bound keeps the original sanitized Clap error. These are recovery-output
bounds, not new limits on valid canonical search commands.

## Copyable command and error envelopes

Render the candidate with a lossless POSIX-shell argv renderer; do not join raw
strings and do not use the current trim-based `NextCommand::render_shell`.
Every preserved argument, including empty or leading/trailing-space values,
must round-trip through `shlex::split` to the candidate argv. Quotes,
backslashes, spaces, Unicode, `$`, backticks, semicolons, ampersands, pipes,
redirections, parentheses, glob characters, and a leading dash after `--`
remain inert data. The display program is always `biomcp`, regardless of the
caller's argv[0]. Put the rendered command through the repository's adaptive
Markdown code-span renderer, so embedded backticks cannot terminate the span.
The exact diagnostic sentence for an ordinary command is:

```text
reversed search syntax; use `<copyable canonical command>`
```

For the native human CLI, return an `InvalidSubcommand` Clap error from the root
command with that sentence. The complete rendered stderr, including root usage
and help footer, is golden-tested; stdout is empty and exit status is 2. The
sentence appears exactly once and the old unrelated `similar subcommand`
suggestion is absent.

With an active pre-`--` `--json` or `-j`, retain the existing parse-error JSON
envelope: stdout is one pretty-printed object, stderr is empty, exit status is
2, `error.code` is `invalid_argument`, `error.message` contains the exact
sentence and quoted command once, and `_meta.not_found` is false. No result,
pagination, or provider metadata is added. Other human and JSON parse errors,
including candidate-invalid reversals, remain byte-for-byte unchanged.

For hostile commands the code-span fence grows as required; the prose before
the span is unchanged and the span's content is the exact shell command.

## MCP decision

Raw MCP's `biomcp` escape hatch uses the same bounded classifier on its
user-supplied, already `shlex::split` argv before adding the raw tool's synthetic
`--json` flag and before replacing an unparseable command with the generic
allowlist message. An embedded global flag is preserved, but the raw tool's
`json: true` selector does not appear in the suggested command. A valid
reversal returns a tool error (`isError: true`) with one text item exactly for
an ordinary command:

```text
Error: reversed search syntax; use `<copyable canonical command>`
```

Hostile commands use the same adaptive code-span rule. The tool does not
execute the candidate. The raw tool's existing 1,024-byte command
limit, unmatched-quote rejection, read-only allowlist, and generic response for
all other parse errors remain unchanged. The raw tool's `json` input and an
embedded `--json` do not turn this tool error into a JSON document.

Typed MCP is unaffected. Typed `search` already constructs canonical argv and
does not accept a free-form command grammar. Its schema, results, errors, and
the seven-tool catalog remain byte-for-byte unchanged.

## No-work and ownership guarantees

Detection, candidate parsing, quoting, and error rendering are pure preflight.
They must not initialize an HTTP client, open or create the managed cache, read
article session state, acquire a provider limiter, start a retry, dispatch an
entity handler, or issue a provider request. Candidate validation constructs
only a fresh Clap command.

Keep the small integration hook and global-flag/error-envelope ownership in
`src/cli/shared.rs`, which must remain at or below the repository's 700-line CLI
cap. Put matrix tests in the existing CLI test sidecars rather than inline.
Raw-MCP integration stays in `src/mcp/shell.rs`; do not raise its existing
source-size allowance. If production logic cannot fit those rails, perform a
package-neutral extraction/rename and lower the corresponding inventory rather
than increasing a ratchet. Add no dependency and keep the package at exactly
1,300 paths.

## Acceptance

1. A table-driven parser test covers all fifteen names and proves the exact
   canonical argv and rendered command. Each corrected candidate parses with
   raw `build_cli`; no test calls `try_parse_cli` recursively.
2. Global/delimiter cases cover each global flag before, between, and after the
   reversed words; multiple globals; `--` before the pair; `--` after the pair;
   and `--json` after `--`. Root/family help and version behavior is captured,
   as are the correction-or-baseline outcomes for trailing `--help`, `-h`,
   `--version`, and `-V` after raw candidate validation.
3. Candidate-invalid, unknown/case-varied entities, `search <entity>`, option
   values containing `search`, non-UTF-8/control input, and each classifier cap
   preserve the baseline error. Existing `skill uninstall` recovery remains
   unchanged.
4. Process goldens pin complete human stderr/stdout/exit and JSON
   stdout/stderr/exit for article, trial, adverse-event, and one global/delimiter
   case. Poisoned provider bases plus an absent temporary cache root prove zero
   requests and no cache/session filesystem creation.
5. Hostile-argument property tests render and `shlex::split` the correction and
   recover the exact swapped candidate argv. A subprocess test parses but never
   executes the rendered command and proves filesystem sentinels named in `$()`,
   backticks, redirections, and semicolon payloads are not created.
6. Raw stdio and Streamable HTTP MCP tests cover article/trial corrections,
   hostile round-trip text, both JSON selectors, the generic-error fallback,
   `isError`, exact content, and zero provider/cache work. Typed-search schema
   snapshots, behavior fixtures, and the seven-tool catalog remain unchanged.
7. Run focused CLI parser/process and MCP tests, the package/source-size and
   quality ratchets, then `make lint`, `make test`, and `make spec`.

## Dependencies and boundary

This ticket has no dependency on 1147. That ticket changes valid article batch
grammar; it does not change any `SearchEntity` spelling or this invalid-command
recovery seam. Draft 1163 changes reserved-keyword validation prose after a
canonical article search has parsed and is likewise independent. Whichever
lands first must preserve the other's public contract; do not add either edge
without new code evidence.

This ticket does not add reversed aliases, change valid search arguments,
rename `search|get|batch <entity>`, change provider behavior, redesign general
Clap suggestions, expose arbitrary parser diagnostics through raw MCP, alter
typed MCP, or update unrelated documentation.

## Review

Implementation starts only after independent design acceptance. Code review
must inspect the nonrecursive raw-Clap validation seam, full envelope goldens,
lossless quoting and bounds, delimiter/global precedence, raw-versus-typed MCP
behavior, no-work evidence, and both source/package ratchets.

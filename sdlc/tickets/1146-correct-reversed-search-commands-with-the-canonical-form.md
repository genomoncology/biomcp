---
flow: build
priority: 5
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

## Complexity and implementation route

Contract 2 + state/timing 0 + reach 1 + proof 2 + cost of error 2 = 7.
This is Level 3 because a copy/paste shell-quoting defect would be a credible
command-injection exposure across native CLI and raw MCP error surfaces. Use
GPT-5.6 SOL Medium for implementation and GPT-5.6 SOL Medium for independent
code review.

## Exact detector

Own the recovery hook beside `build_cli` and `try_parse_cli` in
`src/cli/shared.rs`. Run the bounded detector before accepting the parsed
command because the `gene`, `drug`, and `variant` families deliberately use
`external_subcommand` catchalls: their reversed forms are syntactically
accepted by Clap even though they are not valid search grammar. The detector
must intercept only the exact reversed pattern described below; genuine
external-subcommand shorthand remains accepted. Root or family help/version
that occurs before a complete reversed pair remains owned by Clap, while a
trailing help/version flag after that pair is preserved in the correction as
specified below.

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

Classify the lexical pair before applying recovery budgets or validating the
candidate. Scan only until a literal `--` or the first two non-global command
tokens; do not allocate, UTF-8-convert, render, or echo the remaining argv
during this phase. Keep three explicit outcomes: not reversed, a complete
correction, or a recognized reversal whose supplied arguments cannot be
accepted for correction. This distinction is required because the `gene`,
`drug`, and `variant` external-subcommand catchalls must never turn a
recognized reversal into runtime work merely because validation, encoding, or
a recovery bound suppresses the complete correction.

Build the candidate by swapping only those two argv elements. Preserve every
other argument byte, spelling, position, duplicate, and the `--` delimiter.
Validate the candidate with a fresh raw `build_cli().try_get_matches_from(...)`
call, never `try_parse_cli`; this prevents recursive recovery. An `Ok` result
or candidate `DisplayHelp`/`DisplayVersion` is structurally valid. Any other
candidate error normally returns the original Clap error unchanged, so a
reversal with invalid search arguments does not replace a more relevant
diagnostic. If the original parse is instead successful only because one of
the three external-subcommand catchalls accepted the reversed words, reject it
before dispatch with this fixed safe sentence, substituting the already
allowlisted entity name:

```text
reversed search syntax; use `biomcp search <entity>`; the supplied search arguments were not accepted
```

Use the same fallback when a recognized catchall reversal cannot produce the
complete correction because of a recovery entry/byte bound, non-UTF-8 or
control input, shell-rendering failure, or the complete-sentence bound. Do not
include any rejected argument in this fallback. Native JSON selection still
honors only `--json` or `-j` before `--`. Raw MCP returns the fallback as its
ordinary `Error: ` tool error and performs no work. The other twelve command
families retain their original Clap errors in these suppressed-correction
cases. Typed MCP remains unaffected.

The native fallback uses the same envelope as a complete correction: construct
a root `InvalidSubcommand` Clap error with complete root usage/help footer,
write stderr only, and exit 2. With a pre-`--` `--json` or `-j`, use the
existing pretty parse-error JSON envelope on stdout only, exit 2,
`error.code = invalid_argument`, and `_meta.not_found = false`. The fixed
fallback sentence appears exactly once, and neither human nor JSON output
contains bytes from rejected arguments.

Root or family help/version that Clap already recognizes before a complete
reversed pair remains byte-for-byte unchanged. A trailing help/version flag
after the reversed pair is merely
preserved in the proposed command: `biomcp article search --help` is still an
exit-2 correction to `biomcp search article --help`, not an alias that displays
help. Global JSON selection recognizes `--json` or `-j` only before `--` for
this correction; a token after `--` is candidate data and cannot change the
error envelope.

The classifier accepts at most 256 argv entries total, including argv[0]. The
limit is inclusive. The UTF-8 input budget is the sum of the byte lengths of
every argv entry, including argv[0] and with no separators added; it is at most
16,384 bytes inclusive. Non-UTF-8 input, C0/C1 controls in any entry, or a
breached entry/input bound keeps the original sanitized Clap error for the
twelve non-catchall families. A recognized `gene`, `drug`, or `variant`
reversal instead uses the fixed safe fallback above so it cannot dispatch.

After candidate validation and lossless shell rendering, build the shared
diagnostic sentence including its adaptive Markdown code span. That complete
sentence, from `reversed` through the closing code-span delimiter, must be at
most 32,768 UTF-8 bytes inclusive. If quoting or code-span expansion makes it
one byte larger, keep the original Clap error for the twelve non-catchall
families and use the fixed safe fallback for `gene`, `drug`, or `variant`. The
raw MCP `Error: ` prefix, fixed Clap usage/footer, and fixed JSON-envelope
syntax are outside this variable-content measurement. These are bounded
recovery-classification and diagnostic rules, not new limits on valid
canonical search commands.

## Copyable command and error envelopes

Render the candidate with a lossless POSIX-shell argv renderer; do not join raw
strings and do not use the current trim-based `NextCommand::render_shell`.
Every preserved argument, including empty or leading/trailing-space values,
must round-trip through `shlex::split` to the candidate argv. Quotes,
backslashes, spaces, Unicode, `$`, backticks, semicolons, ampersands, pipes,
redirections, parentheses, glob characters, and a leading dash after `--`
remain inert data. The display program is always `biomcp`, regardless of the
caller's argv[0]. Upgrade the shared adaptive Markdown code-span renderer in
`src/render/markdown/support.rs`: use a delimiter one backtick longer than the
longest run in the value, and add one ASCII padding space on both sides when
the value begins or ends with an ASCII space or backtick. A nonempty value
made entirely of ASCII spaces receives no added padding because CommonMark
preserves all-space code-span content instead of stripping the padding.
Ordinary values and ticket 1144's landed root-continuation rendering remain
byte-for-byte unchanged. Put the rendered command through that helper, so
embedded backticks cannot terminate the span.

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
including candidate-invalid reversals in the twelve non-catchall families,
remain byte-for-byte unchanged. Candidate-invalid `gene`, `drug`, and
`variant` reversals use the fixed safe fallback envelope above.

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
Their call chain may only inspect argv, construct a fresh Clap command, parse
the candidate, and render the correction; code review must trace that chain
and reject any route into runtime configuration, HTTP-client construction,
managed cache or article-session access, provider limiting/retry, or entity
dispatch. Candidate validation constructs only a fresh Clap command.

The externally observable half of this invariant is executable: subprocess
tests use poisoned provider bases, request logs, and absent temporary cache and
session roots to prove that correction performs no provider request and creates
no managed state. The ticket does not require test-only instrumentation for
each unreachable in-memory event.

Keep the small integration hook and global-flag/error-envelope ownership in
`src/cli/shared.rs`, which must remain at or below the repository's 700-line CLI
cap. The CommonMark-safe padding upgrade belongs in the existing shared
`src/render/markdown/support.rs` helper, with focused tests beside its current
owner. Put matrix tests in the existing CLI test sidecars rather than inline.
Raw-MCP integration stays in `src/mcp/shell.rs`; do not raise its existing
source-size allowance. If production logic cannot fit those rails, perform a
package-neutral extraction/rename and lower the corresponding inventory rather
than increasing a ratchet. Add no dependency and keep the package at exactly
1,300 paths.

## Acceptance

1. A table-driven parser test covers all fifteen names and proves the exact
   canonical argv and rendered command. Each corrected candidate parses with
   raw `build_cli`; no test calls `try_parse_cli` recursively. Production-hook
   tests additionally prove that the `gene`, `drug`, and `variant`
   external-subcommand catchalls cannot bypass correction while their genuine
   shorthand remains accepted.
2. Global/delimiter cases cover each global flag before, between, and after the
   reversed words; multiple globals; `--` before the pair; `--` after the pair;
   and `--json` after `--`. Root/family help and version behavior is captured,
   as are the correction-or-baseline outcomes for trailing `--help`, `-h`,
   `--version`, and `-V` after raw candidate validation.
3. Candidate-invalid, unknown/case-varied entities, `search <entity>`, option
   values containing `search`, non-UTF-8/control input, and each classifier cap
   preserve the baseline error. Tests pin exactly-at and one-over the 256-entry
   and 16,384-byte input bounds, plus exactly-at and one-over the 32,768-byte
   complete-sentence bound, including a case where shell quoting or Markdown
   fencing—not raw input alone—causes output overflow. The baseline-preservation
   rule applies when the original parse is an error; table-driven tests for
   `gene`, `drug`, and `variant` instead pin the fixed safe fallback for each
   candidate-invalid and suppressed-correction class and prove no runtime
   dispatch. Existing `skill uninstall` recovery remains unchanged.
4. Process goldens pin complete human stderr/stdout/exit and JSON
   stdout/stderr/exit for article, trial, adverse-event, and one global/delimiter
   case. Poisoned provider bases plus an absent temporary cache root prove zero
   requests and no cache/session filesystem creation. The same subprocess
   no-work proof covers all three catchall families for an ordinary reversal,
   an invalid swapped candidate, and representative input/output-bound or
   encoding suppression; no case may create a lock, cache, or session path.
   Golden the fixed human and JSON fallback envelopes for all three catchalls
   on candidate-invalid input, including pre-`--` JSON and post-`--` non-JSON
   selection/no-echo cases, plus one representative suppression class.
5. Hostile-argument property tests render and `shlex::split` the correction and
   recover the exact swapped candidate argv. Focused shared-helper goldens pin
   leading/trailing backticks, leading/trailing ASCII spaces, nonempty all-space
   values, embedded backtick runs, unchanged ordinary values, and unchanged
   ticket 1144 root-continuation output. A subprocess test parses but never
   executes the rendered command and proves filesystem sentinels named in `$()`,
   backticks, redirections, and semicolon payloads are not created.
6. Raw stdio and Streamable HTTP MCP tests cover article/trial corrections,
   hostile round-trip text, both JSON selectors, the generic-error fallback,
   `isError`, exact content, and zero provider/cache work. Both transports also
   cover the fixed catchall fallback for gene/drug/variant before synthetic
   JSON insertion: `json: true` and an embedded `--json` remain one text tool
   error, and poisoned providers plus absent cache/session roots prove no work.
   Unit tables cover the remaining bound/encoding suppression classes without
   duplicating every transport. Typed-search schema snapshots, behavior
   fixtures, and the seven-tool catalog remain unchanged.
7. Run focused CLI parser/process and MCP tests, the package/source-size and
   quality ratchets, then `make lint`, `make test`, and `make spec`.

## Dependencies and boundary

Ticket 1147 is completed on this main. Its valid article batch grammar did not
change any `SearchEntity` spelling or this invalid-command recovery seam, so no
dependency edge is needed. Ticket 1163 now has an accepted living design on
main; it changes reserved-keyword validation prose only after a canonical
article search has parsed and is likewise independent. This implementation
must preserve both contracts, and no edge should be added without new code
evidence.

This ticket does not add reversed aliases, change valid search arguments,
rename `search|get|batch <entity>`, change provider behavior, redesign general
Clap suggestions, expose arbitrary parser diagnostics through raw MCP, alter
typed MCP, or update unrelated documentation.

## Review

An earlier independent design review accepted the prior revision. A fresh SOL
review against current main rejected missing CommonMark padding ownership,
ambiguous resource-bound domains, overstated no-work proof, stale landed-state
facts, and an understated implementation level. This revision addresses those
findings and the same reviewer accepted it with no remaining material finding.
Code review must verify the complete 15-name detector,
delimiter/global/help/version precedence, nonrecursive raw Clap validation,
separate native/raw/typed MCP outcomes, lossless hostile-argv round trips,
observable zero-work proof, CommonMark padding, and source/package limits
against the implementation diff.

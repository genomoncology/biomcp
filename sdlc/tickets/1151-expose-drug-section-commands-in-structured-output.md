---
flow: build
priority: 6
deps: [1161]
---

# Expose drug section commands in every card surface

## Goal

A drug card exposes a bounded, ordered set of executable commands for useful
sections that this request did not load. On 2026-09-04,
`biomcp --json get drug eflornithine` omitted the `approvals`, `label`, and
`regulatory` commands that reveal the current Iwilfin approval and indication,
although those commands were accepted by the same CLI. Markdown had a separate
partial section list, while batch JSON emitted only related-entity pivots.

The result is command discovery only. It does not load another section, infer
that a populated side-effect field was requested, or change drug evidence.

## One projection owner

Add one shared owner,
`crate::render::markdown::drug_command_discovery(drug, requested_sections,
effective_region) -> DrugCommandDiscovery`. The returned value contains
`recovery`, `sections`, optional `all`, `related`, and the flattened
`next_commands`. Single-card and batch JSON use `next_commands`; single-card
and batch Markdown render the categorized lists from the same value. Raw MCP
and typed MCP continue to obtain their result from those production CLI
renderers rather than reconstructing commands.

The shared owner contains the sole drug discovery-priority table shown below
and asserts that every entry is a unique non-`all` member of
`DRUG_SECTION_NAMES`. Recovery eligibility/order still comes from the
source-state registry, related candidates come from the existing
`related_drug`, and every command is constructed with `NextCommand` argument
quoting. No renderer, JSON path, or batch path duplicates that table or command
builder. A test fails if the single or batch renderers call `related_drug`,
`sections_drug`, or `with_section_recovery` directly instead of this owner.

`DrugCommandDiscovery` is internal rendering state, not a new serialized
public object. JSON changes only `_meta.next_commands`; Markdown changes only
its existing recovery/`More`/`All`/`See also` guidance.

## Exact loaded-section semantics

Normalize requested section tokens by Unicode trimming and ASCII lowercase;
ignore empty tokens and JSON flags exactly as the current parser does. A
section is loaded because the request plan selected it, never because a
provider happened to populate a field or returned data/empty/failure.

- With no section tokens, only `targets` is implicitly loaded. Base identity,
  OpenFDA metadata, adverse-event preview, or any incidental field does not
  make `label`, `safety`, or another named section loaded.
- With one or more explicit non-`all` tokens, exactly those distinct named
  sections are loaded. For example, `regulatory` may populate the shared
  approvals payload, but `approvals` remains not loaded; `interactions` may
  acquire label text, but `label` remains not loaded.
- `all` loads its current parser expansion exactly: `label`, `regulatory`,
  `safety`, `shortage`, `targets`, `indications`, `interactions`, and `civic`.
  Legacy US-only `approvals` is not part of that expansion and remains the sole
  section-discovery candidate. Any additional token beside `all` is already
  covered by the expansion or `approvals`; duplicates do not change the set.
- A loaded section with `degraded` or `unavailable` outcome is still loaded.
  Its identical command may appear once in the recovery tier, but never again
  as not-loaded discovery. `data`, `empty`, `inapplicable`, and `not_requested`
  do not create recovery commands.

Changing the `all` expansion or default implicit `targets` behavior is outside
this ticket. Tests pin both so a later behavior change must update discovery
and acquisition together.

## Region-safe command construction

The owner receives the already resolved effective `DrugRegion`, not merely the
optional input flag. CLI single-card rendering passes
`resolve_drug_get_region`; ordinary entity/batch rendering passes `us`, which
is the current `drug::get` behavior. Raw and typed MCP therefore inherit the
same effective region as their underlying CLI request.

Commands for `regulatory`, `safety`, and `shortage` always append the canonical
`--region us|eu|who|all` matching that effective region. `ema` is input-only
and is rendered as `eu`. Under `who`, `safety` and `shortage` are excluded
because those standalone commands are rejected; `regulatory --region who`
remains eligible. `approvals` never carries `--region`. Nonregional `label`,
`targets`, `indications`, `interactions`, and `civic` never carry it. Thus a
default US card suggests `biomcp get drug <name> regulatory --region us`, while
an unflagged sole `regulatory` request, whose effective region is `all`, keeps
`--region all` on later regional commands.

Recovery commands use the same rule. This intentionally specializes the
generic registry route only at the shared drug projection owner; the registry
still owns whether recovery exists and its section identity. Ticket 1161 makes
the sole `interactions` recovery command executable and must land first.

## Exact ordering, cap, and deduplication

Build candidates in these tiers and flatten them in this order:

1. Recovery commands for loaded `degraded`/`unavailable` sections, in existing
   drug source-state-registry order: `approvals`, `safety`, `targets`,
   `indications`, `interactions`, `civic`.
2. At most three not-loaded section commands in this priority order:
   `approvals`, `label`, `regulatory`, `safety`, `shortage`, `interactions`,
   `indications`, `targets`, `civic`, after region-ineligible candidates are
   removed.
3. One aggregate command,
   `biomcp get drug <resolved-name> all --region <effective-region>`, only when
   the request did not contain `all` and at least one parser-expanded section
   is not loaded. The explicit canonical region prevents an EU/WHO/all card
   from silently becoming the default US `all` request.
4. Existing related candidates in their current order: conditional review
   article, trials, adverse events, pharmacogenomics, then first nonblank gene
   target.

Deduplicate the rendered command strings by exact bytes, first occurrence
wins, then truncate the flattened list to ten. Do not lowercase command strings
for deduplication. Recovery therefore wins over an identical discovery command,
section discovery wins over `all`/related commands, and related commands may be
displaced when higher-priority recovery consumes the cap. Categorized Markdown
contains only entries that survived global deduplication and truncation; it
cannot display a command absent from `_meta.next_commands`.

The default successful eflornithine fixture pins the currently observed sparse
review fallback and ODC1 target pivot, so its exact array is:

```json
[
  "biomcp get drug eflornithine approvals",
  "biomcp get drug eflornithine label",
  "biomcp get drug eflornithine regulatory --region us",
  "biomcp get drug eflornithine all --region us",
  "biomcp search article --drug eflornithine --type review --limit 5",
  "biomcp drug trials eflornithine",
  "biomcp drug adverse-events eflornithine",
  "biomcp search pgx -d eflornithine",
  "biomcp get gene ODC1"
]
```

Separate pure cases pin that a non-sparse fixture omits the review candidate
and a blank first target omits the gene candidate; neither slot is optional in
an assertion.

For `get drug eflornithine all`, the section portion is exactly
`["biomcp get drug eflornithine approvals"]`; there is no aggregate command.
For `approvals label`, both are loaded and the next three discovery choices are
`regulatory --region us`, `safety --region us`, and `shortage --region us`.
For a failed requested `safety --region eu`, its recovery command is first and
byte-identical to the command printed beside the failure status.

## Exact Markdown contract

Recovery remains adjacent to its status as the exact prefix `Retry: ` followed
by the command in the existing safe variable-length Markdown code span.
Surviving discovery commands render in one existing-style block:

```text
More:
  <command>   - <existing section description>
```

The surviving aggregate command renders as:

```text
All:
  <command>
```

Surviving related commands render in the existing `See also:` block and retain
their existing descriptions. Empty categories render no heading or blank
placeholder. Tests compare the complete guidance suffix byte-for-byte. They
separately assert every recovery-tier command appears exactly once beside its
owning status and no non-recovery command appears there; the `More`, `All`, and
`See also` categories contain exactly the surviving entries of their
corresponding plan tiers. `_meta.next_commands` keeps the normative flattened
tier order even when owning section statuses occur in a different physical
order in the existing Markdown card. Section payload text, evidence URLs,
source statuses, and provenance remain byte-identical.

## Executable production-path acceptance

Extend the existing drug fixture/spec and colocated command-property/surface
agreement tests; do not create a second fixture family.

1. Pin the pure matrix for default, each single section, `all`, duplicated and
   mixed sections, degraded/unavailable recovery, all four effective regions,
   WHO exclusions, conditional review present/absent, target present/blank,
   exact dedupe, and the 9/10/11 candidate cap boundary. Assert categorized
   lists and exact flattened arrays, not `contains` checks.
   Duplicate-section coverage uses the pure and CLI paths; typed MCP retains
   its current duplicate-section rejection byte-for-byte.
2. Through the executable CLI, cover single-card Markdown and JSON for default,
   `all`, a multi-section request, and an EU failure/recovery. Assert the exact
   JSON array and complete Markdown guidance suffix. Parse every emitted command
   with the real CLI parser and execute representative section, aggregate,
   recovery, and related commands against the same fixture.
3. Use a resolved fixture identity containing whitespace, quotes, backslash,
   dollar, backtick, semicolon, and ampersand. Assert the exact `NextCommand`
   rendering, parse it back to one unchanged drug-name argument, execute it,
   and use the fixture request log to prove the provider receives that exact
   identity with no extra command or shell effect.
4. Exercise raw MCP `biomcp` for single get in Markdown and JSON and for batch
   in Markdown and JSON. Exercise typed MCP `get` for the same single-card
   Markdown and JSON cases. Assert non-error tool results and exact agreement
   with CLI arrays/text. Raw MCP batch and CLI batch cover two ordered drugs;
   each Markdown item has its own exact guidance suffix and each JSON
   `items[i].result._meta.next_commands` equals the shared owner for that item.
5. Execute CLI batch Markdown/JSON with default, `all`, and multi-section
   requests. Prove input/result order is unchanged, no additional provider
   request is made merely to construct guidance, and commands use each resolved
   result identity rather than the caller spelling or another batch item.
6. Keep the typed MCP `get` request schema, section enum, tool count, and tool
   inventory byte-for-byte unchanged. There is no typed batch tool; batch MCP
   coverage is intentionally raw-shell only. Pin that exclusion so the ticket
   cannot be implemented by adding a schema field or tool.
7. Update the drug schema example, user/CLI guidance, and executable spec to
   describe `_meta.next_commands`, the ten-command cap, loaded-section rules,
   and regional flags. Run focused Rust/Python/mustmatch checks, then `make
   lint`, `make test`, and `make spec`; finish with the exact 1,300-file package
   inventory and `git diff --check`.

## Boundaries and dependency

This ticket changes only drug-card follow-up projection and its existing
Markdown guidance. It does not change search output, section parsing or `all`
expansion, default loading, provider requests, drug identity matching,
regional acquisition, outcome classification, evidence/provenance, related
candidate generation, MCP request schemas, typed tool inventory, or add a
file/dependency. Existing exact source-size baselines and CLI 700-line caps may
not increase; package inventory remains exactly 1,300.

Dependency `1161` is real: it reverses the old sole-interactions hard-failure
contract so the recovery command selected here can execute as a typed card.
Ticket 1151 must not duplicate or weaken 1161's reducer, error, request-count,
or pageable-report behavior.

## Review

The initial draft was rejected because “not loaded,” ordering, cap, region
safety, recovery precedence, batch ownership, and executable MCP evidence were
not specified. This revision freezes those contracts against current
production parsing/rendering and awaits independent design review.

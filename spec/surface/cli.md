# CLI Surface

The top-level CLI is the stable envelope around every entity card and helper
surface. These canaries keep the entrypoint discoverability, operator commands,
and command-reference pages honest without re-testing entity-specific data here.

## Top-Level Help Keeps the Surface Visible

The first thing a user sees still needs to teach the major surfaces and the one
documented JSON exception for cache paths.

```bash
out="$(../../tools/biomcp-ci --help)"
echo "$out" | mustmatch like "leading public biomedical data sources"
echo "$out" | mustmatch like "serve-http"
echo "$out" | mustmatch '/suggest\s+Suggest .*biomedical question/'
echo "$out" | mustmatch like "cache path, which stays plain text"
```

## Static Command Guides Stay Task-Oriented

`biomcp list` is the durable command-reference surface. The discover and batch
subpages should keep teaching when to use the command, not just list flags.

```bash
discover="$(../../tools/biomcp-ci list discover)"
echo "$discover" | mustmatch like '`discover <query>`'
echo "$discover" | mustmatch like "If no biomedical entities resolve"
batch="$(../../tools/biomcp-ci list batch)"
echo "$batch" | mustmatch like '`batch <entity> <id1,id2,...>`'
echo "$batch" | mustmatch like "up to 10 IDs"
```

## Validation Exit Codes Separate Bad Usage From Runtime Failures

Invalid command usage should exit `2` whether clap catches it during parsing or
BioMCP catches it during custom validation. Runtime and configuration failures
still use exit `1`, so scripts can distinguish bad usage from a command that was
well-formed but could not complete.

```bash
set +e
tmpdir="$(mktemp -d)"
../../tools/biomcp-ci get gene >"$tmpdir/clap.out" 2>"$tmpdir/clap.err"; clap_status=$?
../../tools/biomcp-ci search gene >"$tmpdir/gene.out" 2>"$tmpdir/gene.err"; gene_status=$?
set -e
test "$clap_status" -eq 2
test "$gene_status" -eq 2
cat "$tmpdir/gene.err" | mustmatch like "Query is required"
rm -rf "$tmpdir"
```

Diagnostic search has its own custom missing-filter validator and should follow
the same bad-usage exit policy.

```bash
set +e
tmpdir="$(mktemp -d)"
../../tools/biomcp-ci search diagnostic >"$tmpdir/diagnostic.out" 2>"$tmpdir/diagnostic.err"; diagnostic_status=$?
set -e
test "$diagnostic_status" -eq 2
cat "$tmpdir/diagnostic.err" | mustmatch like "requires at least one of"
rm -rf "$tmpdir"
```

The same mapping applies to custom validation outside entity search.

```bash
set +e
tmpdir="$(mktemp -d)"
ids="BRAF,TP53,EGFR,KRAS,NRAS,PIK3CA,ALK,ROS1,MET,RET,NTRK"
../../tools/biomcp-ci batch gene "$ids" >"$tmpdir/batch.out" 2>"$tmpdir/batch.err"; batch_status=$?
set -e
test "$batch_status" -eq 2
cat "$tmpdir/batch.err" | mustmatch like "Batch is limited to 10 IDs"
rm -rf "$tmpdir"
```

A non-validation failure stays on the runtime-failure exit code.

```bash
set +e
tmpdir="$(mktemp -d)"
../../tools/biomcp-ci search trial -c melanoma --source nci >"$tmpdir/nci.out" 2>"$tmpdir/nci.err"; nci_status=$?
set -e
test "$nci_status" -eq 1
cat "$tmpdir/nci.err" | mustmatch like "API key required"
cat "$tmpdir/nci.err" | mustmatch like "NCI_API_KEY"
rm -rf "$tmpdir"
```

## List Command Honors Global JSON

`biomcp list` remains the human command-reference page by default, but scripts
and agents that pass the global `--json` flag need structured reference data
instead of Markdown. The root list exposes the entity/command inventory, while an
entity page exposes the named command entries for that surface.

```bash
set -e
root_json="$(../../tools/biomcp-ci --json list)"
gene_json="$(../../tools/biomcp-ci --json list gene)"
ROOT_JSON="$root_json" GENE_JSON="$gene_json" uv run --no-sync python3 - <<'PY'
import json, os
root = json.loads(os.environ["ROOT_JSON"])
gene = json.loads(os.environ["GENE_JSON"])
def entries(value): return value if isinstance(value, list) else []
entities = entries(root.get("entities"))
assert "gene" in entities
refs = [*entries(root.get("patterns")), *entries(root.get("commands"))]
assert any("search all" in str(entry) for entry in refs)
assert gene.get("entity") == "gene"
assert any("get gene <symbol>" in str(command) for command in entries(gene.get("commands")))
PY
echo "$root_json" | mustmatch like '"entities"'
```

## Operator Commands Keep Distinct Output Modes

The operator-facing cache and version commands intentionally differ from the
query surface: cache path stays plain text, while verbose version output exposes
the executable/build identity for debugging.

```bash
path="$(../../tools/biomcp-ci --json cache path)"
echo "$path" | mustmatch '/^\/.*\/\.cache\/biomcp-specs\/http$/'
version="$(../../tools/biomcp-ci version --verbose)"
echo "$version" | mustmatch '/^biomcp 0\.[0-9]+\.[0-9]+/'
echo "$version" | mustmatch like "Executable:"
echo "$version" | mustmatch '/Build: version=0\.[0-9]+\.[0-9]+, git=[0-9a-f]+, date=[-0-9:+TZ]+/'
```

## Emitted Commands Stay Shell-Safe

Suggested commands are part of the CLI surface because operators and agents copy
and paste them directly into shells. Multiword phrases, apostrophes, and
parenthesized tokens must stay runnable when they appear inside emitted follow-up
commands, while plain single-token anchors should stay readable without extra
quoting.

```bash
set -e
plain="$(../../tools/biomcp-ci suggest "What drugs treat melanoma?")"
spaced="$(../../tools/biomcp-ci suggest "What drugs treat paclitaxel protein-bound?")"
echo "$plain" | mustmatch like "biomcp search drug --indication melanoma"
echo "$spaced" | mustmatch like 'biomcp search drug --indication "paclitaxel protein-bound"'
```

Discover is the brittle case because it emits commands directly from free text.
The quoted form below is the copy-paste contract, not presentation polish.

```bash
set -e
discover_disease_cmd="$(../../tools/biomcp-ci discover "Graves'" | grep 'biomcp search disease -q' | head -1 | tr -d '`' | sed 's/^- //')"
discover_article_cmd="$(../../tools/biomcp-ci discover "Graves'" | grep 'biomcp search article -k' | head -1 | tr -d '`' | sed 's/^- //')"
discover_disease_argv="$(uv run --no-sync python3 -c 'import shlex,sys; print(" ".join(shlex.split(sys.argv[1])))' "$discover_disease_cmd")"
discover_article_argv="$(uv run --no-sync python3 -c 'import shlex,sys; print(" ".join(shlex.split(sys.argv[1])))' "$discover_article_cmd")"
echo "$discover_disease_argv" | mustmatch like "search disease -q Graves'"
echo "$discover_article_argv" | mustmatch like "search article -k Graves'"
```

Cross-entity orientation needs the same contract in its counts-only follow-up
links, including parenthesized protein-complex-like terms.

```bash
set -e
paren_cmd="$(../../tools/biomcp-ci search all --keyword "AP-1(c-Jun/c-Fos)" --counts-only | grep 'type review' | head -1 | tr -d '`' | sed 's/^- //')"
paren_argv="$(uv run --no-sync python3 -c 'import shlex,sys; print(shlex.split(sys.argv[1]))' "$paren_cmd")"
echo "$paren_argv" | mustmatch like "AP-1(c-Jun/c-Fos)"
```

Apostrophe-bearing counts-only follow-ups are the current broken surface in
`search all` and must become copy-pasteable.

```bash
set -e
apostrophe_cmd="$(../../tools/biomcp-ci search all --keyword "Graves'" --counts-only | grep 'type review' | head -1 | tr -d '`' | sed 's/^- //')"
apostrophe_argv="$(uv run --no-sync python3 -c 'import shlex,sys; print(" ".join(shlex.split(sys.argv[1])))' "$apostrophe_cmd")"
echo "$apostrophe_argv" | mustmatch like "--keyword Graves'"
```

## Health and Admin Help Stay Explicit

The CLI-only operator surface should still render a health table for quick
inspection and keep local-runtime admin help truthful about what each sync owns.

```bash
health="$(../../tools/biomcp-ci health --apis-only)"
echo "$health" | mustmatch like "# BioMCP Health Check"
echo "$health" | mustmatch like "| API | Status | Latency | Affects |"
whoivd="$(../../tools/biomcp-ci who-ivd sync --help)"
echo "$whoivd" | mustmatch like "WHO Prequalified IVD diagnostic CSV export"
echo "$whoivd" | mustmatch like "Usage: biomcp who-ivd sync"
```

The health implementation should also keep its documented decomposition ratchet
executable in the spec lane so the operator surface cannot regress into one
large catch-all module.

```bash
set +e
structure_out="$(cd ../.. && cargo test --test health_cli_structure -- --nocapture 2>&1)"
structure_status=$?
set -e
echo "$structure_out" | mustmatch like "health_split_files_exist_with_doc_headers"
test "$structure_status" -eq 0
```

## List Command Reference Decomposition Stays Executable

The list command reference should keep its documented decomposition ratchet
executable in the spec lane so page builders cannot regress into one large
catch-all module.

```bash
set +e
list_structure_out="$(cd ../.. && cargo test --test list_cli_structure -- --nocapture 2>&1)"
list_structure_status=$?
set -e
echo "$list_structure_out" | mustmatch like "list_split_files_exist_with_doc_headers"
test "$list_structure_status" -eq 0
```

## Article CLI Test Ownership Stays Decomposed

The article CLI tests should keep the same executable ownership ratchet: a split
sidecar tree with named domains, module headers, and the CLI 700-line cap.

```bash
set +e
structure_out="$(cd ../.. && cargo test --test article_cli_tests_structure -- --nocapture 2>&1)"
structure_status=$?
set -e
echo "$structure_out" | mustmatch like "article_cli_test_split_files_exist_with_doc_headers"
test "$structure_status" -eq 0
```

## Global CLI Line-Cap Allowlist Is Fully Absorbed

The global `src/cli` 700-line ratchet should no longer need the ticket-334
bootstrap exceptions once residual oversized files are decomposed. Keep the
allowlist empty of ticket-347 follow-ups and keep every tracked CLI Rust file
under the cap.

```bash
set +e
structure_out="$(cd ../.. && cargo test --test cli_line_cap_absorption -- --nocapture 2>&1)"
structure_status=$?
set -e
echo "$structure_out" | mustmatch like "ticket_347_residual_allowlist_entries_are_absorbed"
test "$structure_status" -eq 0
```

## Update Verifies Release Checksum

The self-update command must fail closed when the release `.sha256` sidecar
is missing or mismatched. The unsafe override has to be opt-in per
invocation, marked UNSAFE in `--help`, and the underlying policy must stay
covered by named unit tests so the operator surface can never silently
downgrade to TLS-only trust.

```bash
help="$(../../tools/biomcp-ci update --help)"
printf '%s\n' "$help" | grep -Eiq "SHA-?256"
printf '%s\n' "$help" | grep -Eiq "checksum"
echo "$help" | mustmatch like "--allow-missing-checksum"
```

```bash
set +e
update_out="$(cd ../.. && cargo test --lib enforce_checksum_policy_missing_sidecar_without_override_fails_closed -- --nocapture 2>&1)"
update_status=$?
set -e
echo "$update_out" | mustmatch like "enforce_checksum_policy_missing_sidecar_without_override_fails_closed"
test "$update_status" -eq 0
```

## Benchmark Internal Harness Ratchet Stays Executable

The benchmark tree is an internal regression harness, not a public CLI command.
Its documented decomposition and runtime-wiring ratchet should stay executable
in the spec lane so suite execution, regression analysis, command normalization,
score rendering, and the non-public CLI contract cannot drift.

```bash
set +e
benchmark_structure_out="$(cd ../.. && cargo test --test benchmark_cli_structure -- --nocapture 2>&1)"
benchmark_structure_status=$?
set -e
echo "$benchmark_structure_out" | mustmatch like "benchmark_internal_harness_split_files_exist_with_doc_headers"
echo "$benchmark_structure_out" | mustmatch like "benchmark_internal_harness_contract_pins_runtime_and_docs"
test "$benchmark_structure_status" -eq 0
```

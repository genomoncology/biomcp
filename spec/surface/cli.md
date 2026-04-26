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
discover_disease_argv="$(uv run python3 -c 'import shlex,sys; print(shlex.split(sys.argv[1]))' "$discover_disease_cmd")"
discover_article_argv="$(uv run python3 -c 'import shlex,sys; print(shlex.split(sys.argv[1]))' "$discover_article_cmd")"
echo "$discover_disease_argv" | mustmatch '/Graves'"'"'/'
echo "$discover_article_argv" | mustmatch '/Graves'"'"'/'
```

Cross-entity orientation needs the same contract in its counts-only follow-up
links, including parenthesized protein-complex-like terms.

```bash
set -e
paren_cmd="$(../../tools/biomcp-ci search all --keyword "AP-1(c-Jun/c-Fos)" --counts-only | grep 'type review' | head -1 | tr -d '`' | sed 's/^- //')"
paren_argv="$(uv run python3 -c 'import shlex,sys; print(shlex.split(sys.argv[1]))' "$paren_cmd")"
echo "$paren_argv" | mustmatch like "AP-1(c-Jun/c-Fos)"
```

Apostrophe-bearing counts-only follow-ups are the current broken surface in
`search all` and must become copy-pasteable.

```bash
set -e
apostrophe_cmd="$(../../tools/biomcp-ci search all --keyword "Graves'" --counts-only | grep 'type review' | head -1 | tr -d '`' | sed 's/^- //')"
apostrophe_argv="$(uv run python3 -c 'import shlex,sys; print(shlex.split(sys.argv[1]))' "$apostrophe_cmd")"
echo "$apostrophe_argv" | mustmatch '/Graves'"'"'/'
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

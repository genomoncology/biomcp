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
echo "$out" | mustmatch '/suggest\s+Suggest the BioMCP skill\/playbook/'
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
echo "$version" | mustmatch '/Build: version=0\.[0-9]+\.[0-9]+, git=[0-9a-f]+, date=/'
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

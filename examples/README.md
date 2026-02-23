# BioMCP Examples

This folder contains supporting paper-demo assets (`prompt.md`, `run.sh`, `score.sh`) for local experimentation.

## Canonical Workflows

For project 116, the canonical workflow interface is embedded skills, not this examples folder.

Use:

```bash
biomcp skill list
biomcp skill <number-or-slug>
```

## Mapping

| Example folder | Canonical skill |
|----------------|-----------------|
| `genegpt/` | `09-gene-function-lookup` |
| `geneagent/` | `10-gene-set-analysis` |
| `trialgpt/` | `03-trial-searching` (patient matching section) |
| `pubmed-beyond/` | `11-literature-synthesis` |

## When to Use This Folder

Use example scripts when you want a quick local benchmark harness with captured outputs/metrics. Use skills when you want the production workflow instructions agents should follow.

---
flow: build
priority: 8
---

# A search filter that could not be resolved must say so, not report zero results

`biomcp search variant` reports a true absence of evidence and a query the backend never had a chance to answer with the same words. Verified against the repository build `biomcp 0.9.0-dev.6` (`./target/debug/biomcp`) on 2026-09-01:

```
$ biomcp search variant -g RB1 --hgvsp Q999X
Resolution: Unresolved
Found: 0 variant(s)
No variants found matching the filters.

$ biomcp search variant -g H3-3A
Found: 0 variant(s)
No variants found matching the filters.
```

The first is a genuine negative. The second is a lookup failure: 1,156 records exist under the withdrawn symbol `H3F3A`, and `biomcp get gene H3-3A` already resolves the same symbol and prints `H3F3A` among its aliases.

The `--hgvsp` path prints `Resolution: Unresolved`. The `-g` path prints nothing. Every other filter prints nothing. So the one signal that would separate the two cases exists on exactly one filter, and it is absent on the filter where the failure actually happens.

This is not a hypothetical risk. It is already load-bearing in agent reasoning. In a 31-task study driving BioMCP through its MCP interface on 2026-09-01, an agent asked to check an invented variant justified its correct negative on `Found: 0` together with `resolution: unresolved`. Those same two signals accompanied 1,156, 924, 10,608 and 28,307 existing records in six other tasks in the same study. The agent reached a right answer using a rule that is not valid, and the rule will produce a wrong answer on the next symbol the index does not carry under the name the caller used.

BioMCP also ships a skill playbook, `15 negative-evidence`, that teaches agents to treat this signal as trustworthy.

## Required behavior

Every filter reports whether its value was resolved, so a caller can tell a query that ran and matched nothing from a query whose input was never recognised.

The signal is available to a program, not only to a person reading a card. An agent consuming JSON must reach the same conclusion a person reading markdown reaches.

A result set of zero states which filters resolved and which did not, so the reader knows whether the zero is an answer.

## Done, observably

- `search variant -g H3-3A` reports that the gene filter did not resolve. `search variant -g RB1 --hgvsp Q999X` reports that both filters resolved and the search matched nothing.
- The resolution outcome for each filter is present in JSON output, not only in rendered text.
- A caller can distinguish the two cases above without running a second command and without consulting a different entity.
- The behaviour applies to search filters generally, not to `--hgvsp` alone.

## Boundary

This ticket adds a signal. It does not change what any query matches, does not add alias resolution to the variant search path, and does not change any count. Ticket 1091 covers making `-g H3-3A` find the records; this ticket covers telling the truth when a filter value is not recognised, and it must be correct whether or not 1091 has landed. Do not change the wording or meaning of the existing `--hgvsp` resolution line beyond what consistency across filters requires.

---
flow: build
priority: 13
deps: ["1030"]
---
# Give the MCP tool catalog real headroom under its budget

The development build's MCP tool catalog measures 15,704 bytes and 3,974 tokens against an enforced ceiling of 16,000 bytes and 4,000 tokens. That is 26 tokens of headroom. The next typed tool, or one more sentence in any existing tool description, fails the gate — and it fails as a budget violation on whatever unrelated ticket happens to add it, which is a confusing place to discover that the real problem is a catalog with no room in it.

The budget itself is a good idea and should stay. Every agent that connects pays for this catalog on every session, so a hard ceiling is the right instrument. The problem is that the ceiling is currently indistinguishable from a tripwire.

Measured on 2026-08-19: the published 0.8.25 release is 4 tools and 21,701 bytes, and the development build is 7 tools and 15,704 bytes, so the direction of travel is good — more capability for a smaller catalog. This ticket is about keeping it that way rather than reversing it.

## The hard choice to settle

Decide whether the answer is to reclaim space inside the current catalog, to raise the ceiling with a stated reason, or to warn before failing. Reclaiming space keeps the discipline but has a floor; raising the ceiling is honest but needs a defensible number rather than a round one; warning first is the gentlest but does not by itself create room. Pick one and justify it in the design. Do not simply raise the number to whatever the current catalog happens to measure, because that sets the ceiling to today's accident.

## Done when

- Adding one typical new typed tool, or a normal-length sentence to an existing tool description, does not fail the budget gate.
- When the gate does fail, the failure message states the current measurement, the ceiling, and which tool descriptions are largest, so the person who hit it can act without going to read the measurement script.
- The catalog measurement is reported on every run of the gate, not only when it fails, so the trend is visible before it becomes a wall.
- The chosen ceiling is written down with the reasoning behind it, so a later reader can tell whether it was measured or guessed.

## Related

The measurement script `scripts/measure-mcp-tools.py` currently depends on the committed tokenizer cache path and on being run from a repository checkout, so it cannot easily be pointed at an arbitrary installed binary. Making it runnable against any binary would let this be checked outside CI. Treat that as in scope only if it falls out naturally; otherwise leave it and say so.

## Existing tests that pin this

The budget gate itself is `tests/test_mcp_tool_catalog.py`, in `test_real_tools_list_stays_within_context_budget`, which asserts `tools/list UTF-8 bytes <= 16_000`, `tools/list cl100k_base tokens <= 4_000`, and `biomcp description UTF-8 bytes <= 4_000`. Restatement is authorized in that file, for that test by name, since changing the ceiling or the failure reporting is the point of this ticket. No other test file is authorized.

Whatever the chosen ceiling is, the assertion must remain a hard failure, not a warning — the gate keeps its teeth.

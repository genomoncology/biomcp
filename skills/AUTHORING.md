# Skill authoring guide

Use this guide when you are turning a repeated workflow into a reusable BioMCP skill playbook. A playbook should be short enough to run during an answer, specific enough to prevent tool wandering, and honest about what each command can and cannot prove.

## Playbook anatomy

Every playbook follows the same shape:

1. **Pattern name** — start with `# Pattern: ...` and name the decision the workflow supports.
2. **When to use it** — one sentence beginning with `Use this when ...` so routing can match the playbook to a user task.
3. **Command block** — one `bash` block with three or four `biomcp ...` commands in the order an agent should run them.
4. **Interpretation** — three to five bullets that explain how to read hits, misses, source labels, and when to stop.

Keep commands concrete. Use real example terms, IDs, or flags rather than placeholders in routable playbooks. If a workflow has optional branches, put the default path in the command block and explain the branch in interpretation.

## Ladder sidecar mapping

A `*.ladder.json` file is the machine-readable version of the command block. Runtime-routed ladder sidecars let BioMCP attach `_meta.workflow` and `_meta.ladder[]` guidance to matching command output; non-routed sidecars are still useful installed references for agents and authors.

Map fields this way:

- `workflow`: the playbook slug, matching the sidecar filename.
- `rationale`: the one-sentence reason to start with this workflow.
- `playbook`: `biomcp skill <slug>`.
- `ladder[].command`: byte-for-byte the commands from the playbook's `bash` block.
- `ladder[].what_it_gives`: the concrete evidence or decision support from that step.

Do not put a template sidecar into routing. Template files use the `_TEMPLATE.*` prefix and are installed as authoring references only.

## Interpretation discipline

A good interpretation section tells the agent what to do with both positive and negative evidence:

- Say which field or section is the strongest answer signal.
- Name source labels that matter, but do not invent missing values.
- Treat an empty structured result as evidence about that source, not as proof the fact is false everywhere.
- Stop when the available structured output supports the answer; widen to articles only when a claim remains unsupported.

## Worked example pattern

The normalize-to-codes playbook shows the pattern for a free-text normalization workflow: run `biomcp discover`, read source-labelled identifiers, and report absent code classes plainly instead of filling gaps with guesses.

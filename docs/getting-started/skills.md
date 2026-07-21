# Skills

BioMCP ships one embedded guide plus supporting reference files and worked
examples for agent workflows. The current workflow is:

```bash
biomcp skill list
biomcp skill
biomcp skill render
biomcp skill article-follow-up
biomcp skill exact-variant-literature
biomcp skill install ~/.claude
```

## Read the overview

`biomcp skill list` is the quickest way to choose the matching
worked-example playbook when you only know the biomedical question. It lists
playbook slugs you can open with `biomcp skill <slug>` for the full workflow.

`biomcp skill` prints the embedded `skills/SKILL.md` overview. Start there if
you want the current BioMCP workflow guidance without installing anything into
an agent directory.

`biomcp skill render` prints the same canonical agent-facing prompt for
scripts and eval runners. Redirected output from `biomcp skill render` is the
same content installed as `SKILL.md`.

## Learn the workflows

Use `biomcp skill list` to browse the embedded worked examples and
`biomcp skill <slug|number>` to open one in the CLI:

```bash
biomcp skill list
biomcp skill article-follow-up
biomcp skill variant-pathogenicity
biomcp skill exact-variant-literature
```

Current builds ship 17 worked examples. The catalog keeps the original
treatment lookup, symptom lookup, gene-disease orientation, and article
follow-up examples, plus expanded playbooks such as `variant-pathogenicity`,
`drug-regulatory`, `trial-recruitment`, `mutation-catalog`,
`negative-evidence`, `normalize-to-codes`, and `exact-variant-literature`. The
installed `skills/` tree also includes worked examples you can read directly in the repo or in an agent
directory:

- [Guide Workflows](../how-to/guide-workflows.md) - variant pathogenicity,
  drug safety, and broad gene-disease investigation

## Install into an agent directory

Install the embedded `skills/` tree into your agent directory:

```bash
biomcp skill install ~/.claude
```

Check whether an installed skill matches this BioMCP binary:

```bash
biomcp --json skill status ~/.claude
```

A fresh managed install reports `current`. Missing or invalid management metadata
reports `unmanaged`; changed managed files report `locally_modified`; and an
intact older payload reports `stale`. Only `current` omits the recovery command.

Plain install never overwrites an existing skill. Repair an installation
explicitly:

```bash
biomcp skill install ~/.claude --force
```

Force replacement updates only BioMCP-managed files and preserves unrelated
files. The `dir` argument can point at an agent root such as `~/.claude`, an
existing `skills/` directory, or a `skills/biomcp/` directory. When you omit `dir`,
BioMCP attempts supported agent-directory detection in your home directory and
the current working tree, then prompts before installing when stdin is a TTY.

## Install payload

Current builds install the full embedded reference tree into
`<agent>/skills/biomcp/`, including:

- `SKILL.md`
- `.biomcp-skill.json` (schema, BioMCP version, canonical render hash, install
  time, and managed-file hashes)
- `AUTHORING.md`
- `use-cases/`
- `use-cases/_TEMPLATE.md` and `use-cases/_TEMPLATE.ladder.json`
- `jq-examples.md`
- `examples/`
- `schemas/`

The install payload also includes `schemas/workflow-ladder.schema.json`,
seven runtime `use-cases/<slug>.ladder.json` sidecars, and the
`normalize-to-codes.ladder.json` worked-example sidecar. The runtime sidecars
are `treatment-lookup`, `article-follow-up`, `variant-pathogenicity`,
`trial-recruitment`, `mechanism-pathway`, `pharmacogene-cumulative`, and
`mutation-catalog`; BioMCP can attach those as `_meta.workflow` /
`_meta.ladder[]` guidance when a command matches their triggers. The
`normalize-to-codes` sidecar is installed authoring reference material paired
with its numbered markdown playbook, not runtime metadata. JSON sidecars are
not listed by `biomcp skill list`. The `_TEMPLATE.*` files are authoring
references, not routed playbooks.

When a first-call JSON response matches a ladder trigger, BioMCP can emit
`_meta.workflow` plus `_meta.ladder[]`. The ladder commands are static copies of
the matching playbook's fenced bash block; they are not templated with user
input. `_meta.next_commands` remains the dynamic one-hop follow-up list for the
current result.

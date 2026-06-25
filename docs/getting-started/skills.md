# Skills

BioMCP ships one embedded guide plus supporting reference files and worked
examples for agent workflows. The current workflow is:

```bash
biomcp suggest "What drugs treat melanoma?"
biomcp skill
biomcp skill render
biomcp skill list
biomcp skill article-follow-up
biomcp skill install ~/.claude
```

## Read the overview

`biomcp suggest "<question>"` is the quickest way to choose the matching
worked-example playbook when you only know the biomedical question. It returns
the playbook slug, two starter commands, and the `biomcp skill <slug>` command
for the full workflow.

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
```

Current builds ship 16 worked examples. The catalog keeps the original
treatment lookup, symptom lookup, gene-disease orientation, and article
follow-up examples, plus expanded playbooks such as `variant-pathogenicity`,
`drug-regulatory`, `trial-recruitment`, `mutation-catalog`,
`negative-evidence`, and `normalize-to-codes`. The installed `skills/` tree
also includes worked examples you can read directly in the repo or in an agent
directory:

- [Guide Workflows](../how-to/guide-workflows.md) - variant pathogenicity,
  drug safety, and broad gene-disease investigation

## Install into an agent directory

Install the embedded `skills/` tree into your agent directory:

```bash
biomcp skill install ~/.claude
```

Force replacement of an existing install:

```bash
biomcp skill install ~/.claude --force
```

The `dir` argument can point at an agent root such as `~/.claude`, an existing
`skills/` directory, or a `skills/biomcp/` directory. When you omit `dir`,
BioMCP attempts supported agent-directory detection in your home directory and
the current working tree, then prompts before installing when stdin is a TTY.

## Install payload

Current builds install the full embedded reference tree into
`<agent>/skills/biomcp/`, including:

- `SKILL.md`
- `AUTHORING.md`
- `use-cases/`
- `use-cases/_TEMPLATE.md` and `use-cases/_TEMPLATE.ladder.json`
- `jq-examples.md`
- `examples/`
- `schemas/`

The install payload also includes `schemas/workflow-ladder.schema.json` and
eight routed `use-cases/<slug>.ladder.json` sidecars for workflow ladders:
`treatment-lookup`, `article-follow-up`, `variant-pathogenicity`,
`trial-recruitment`, `mechanism-pathway`, `pharmacogene-cumulative`,
`mutation-catalog`, and `normalize-to-codes`. These JSON sidecars are not
listed by `biomcp skill list`; they are runtime metadata assets paired with the
numbered markdown playbooks. The `_TEMPLATE.*` files are authoring references,
not routed playbooks.

When a first-call JSON response matches a ladder trigger, BioMCP can emit
`_meta.workflow` plus `_meta.ladder[]`. The ladder commands are static copies of
the matching playbook's fenced bash block; they are not templated with user
input. `_meta.next_commands` remains the dynamic one-hop follow-up list for the
current result.

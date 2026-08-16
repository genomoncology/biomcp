# Skill Authoring Assets

The skill package is both executable guidance and an installable reference tree.
These checks keep the local install payload aligned with the public skill catalog
without depending on upstream biomedical services.

## Skill install ships authoring assets and the normalize-to-codes playbook

Installing the embedded skill tree into a local agent root should copy the
new authoring guide, the non-routing templates, and the worked example sidecar
alongside the canonical prompt.

```bash
../../tools/biomcp-ci skill install ../../.cache/spec-skill-install --force
find ../../.cache/spec-skill-install/skills/biomcp -maxdepth 3 -type f | sed 's#^.*/biomcp/##' | sort | mustmatch like ".biomcp-skill.json
AUTHORING.md
SKILL.md
use-cases/16-normalize-to-codes.md
use-cases/_TEMPLATE.ladder.json
use-cases/_TEMPLATE.md
use-cases/normalize-to-codes.ladder.json"
```

Structured install output tells automation whether the filesystem changed.

```bash
../../tools/biomcp-ci --json skill install ../../.cache/spec-skill-install | jq '.kind == "skill" and .action == "install" and .status == "unchanged" and .changed == false and .skill_status.state == "current"' | mustmatch 'true'
../../tools/biomcp-ci --json skill install ../../.cache/spec-skill-install --force | jq '.status == "repaired" and .changed == true and .skill_status.state == "current"' | mustmatch 'true'
```

## A fresh managed install reports current

The installation stamp lets BioMCP distinguish its own unchanged payload from
an unmanaged or drifted skill. A fresh install made by this binary therefore
reports `current` without rewriting anything.

```bash
../../tools/biomcp-ci --json skill status ../../.cache/spec-skill-install | jq -r .state | mustmatch 'current'
```

## The exact-variant literature playbook is executable guidance

The embedded playbook carries one retrieval-only sequence from strict identity
through evidence escalation. It uses the shipped union, batch, full-text, asset,
and graph grammar directly rather than teaching an article variant flag or a
clinical answer.

```bash
../../tools/biomcp-ci skill exact-variant-literature | mustmatch like 'biomcp search variant MSH2 p.L341P --limit 5
biomcp variant articles "MSH2 p.L341P" --limit 5
biomcp article batch 26951660 31433521
biomcp get article 26951660 fulltext
biomcp --json get article 26951660 assets
biomcp article citations 26951660 --limit 5
biomcp article references 26951660 --limit 5'
```

The template files are installed as authoring references, not runnable
playbooks. The public catalog should expose the worked example while keeping the
underscore template out of routing.

```bash
../../tools/biomcp-ci skill list | mustmatch like "# BioMCP Worked Examples
normalize-to-codes"
../../tools/biomcp-ci skill list | mustmatch not like "_TEMPLATE"
```

## Skill catalog is the discovery surface

Agents that need a worked example should inspect the shipped skill catalog
directly. The overview points to `biomcp skill list` and must not send users
through the retired offline `suggest` router.

```bash
../../tools/biomcp-ci skill | mustmatch like "biomcp skill list"
../../tools/biomcp-ci skill | mustmatch not like "biomcp suggest"
```

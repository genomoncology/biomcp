# Skill Authoring Assets

The skill package is both executable guidance and an installable reference tree.
These checks keep the local install payload aligned with the public skill catalog
without depending on upstream biomedical services.

## Skill install ships authoring assets and the normalize-to-codes playbook

Installing the embedded skill tree into a local agent root should copy the
new authoring guide, the non-routing templates, and the worked example sidecar
alongside the canonical prompt.

```bash
rm -rf ../../.cache/spec-skill-install
../../tools/biomcp-ci skill install ../../.cache/spec-skill-install --force
find ../../.cache/spec-skill-install/skills/biomcp -maxdepth 3 -type f | sed 's#^.*/biomcp/##' | sort | mustmatch like "AUTHORING.md
SKILL.md
use-cases/16-normalize-to-codes.md
use-cases/_TEMPLATE.ladder.json
use-cases/_TEMPLATE.md
use-cases/normalize-to-codes.ladder.json"
```

The template files are installed as authoring references, not runnable
playbooks. The public catalog should expose the worked example while keeping the
underscore template out of routing.

```bash
../../tools/biomcp-ci skill list | mustmatch like "# BioMCP Worked Examples
normalize-to-codes"
../../tools/biomcp-ci skill list | mustmatch not like "_TEMPLATE"
```

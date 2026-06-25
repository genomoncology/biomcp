# Skill authoring

BioMCP's installable skill package includes an authoring guide for turning repeated workflows into reusable playbooks.

Read the canonical guide in [`skills/AUTHORING.md`](https://github.com/genomoncology/biomcp/blob/main/skills/AUTHORING.md). Installed skill bundles also include it at `skills/biomcp/AUTHORING.md` alongside `SKILL.md`, the use-case playbooks, and the `_TEMPLATE.*` authoring files.

The guide covers:

- playbook anatomy: pattern name, when-to-use sentence, command block, and interpretation bullets;
- how a `*.ladder.json` sidecar maps to the playbook commands;
- how to handle missing source labels or absent evidence without inventing results;
- the normalize-to-codes worked example that uses `biomcp discover` for MONDO, HPO, ICD-10, SNOMED, and RxNorm-style code normalization.

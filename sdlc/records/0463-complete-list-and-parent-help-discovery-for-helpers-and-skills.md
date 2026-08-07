---
base: e7bd48b8097a484664aa301c31725b3089d1efc4
head: bca698c0a56c5dae9b1edfb2514dd4a2a781f70f
---
The review found discoverability drift that is not a runtime crash but still weakens the CLI contract: `biomcp list <entity>` omits runnable helpers such as drug trials/adverse-events/interactions, disease trials/articles/drugs, and variant trials/articles/oncokb; `biomcp skill <number-or-slug>` works and is documented, but `biomcp skill --help` only shows `[COMMAND]`; many parent/operator help pages do not show a first useful example. Agents and new users rely on help/list before reading long docs.

Imported from March ticket 463. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/463-complete-list-and-parent-help-discovery-for-helpers-and-skills

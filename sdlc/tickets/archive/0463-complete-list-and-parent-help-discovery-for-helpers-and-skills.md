---
flow: build
priority: 6
---
# Complete list and parent help discovery for helpers and skills

The review found discoverability drift that is not a runtime crash but still weakens the CLI contract: `biomcp list <entity>` omits runnable helpers such as drug trials/adverse-events/interactions, disease trials/articles/drugs, and variant trials/articles/oncokb; `biomcp skill <number-or-slug>` works and is documented, but `biomcp skill --help` only shows `[COMMAND]`; many parent/operator help pages do not show a first useful example. Agents and new users rely on help/list before reading long docs.

Completed under March on 2026-06-30, as March ticket 463. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/463-complete-list-and-parent-help-discovery-for-helpers-and-skills

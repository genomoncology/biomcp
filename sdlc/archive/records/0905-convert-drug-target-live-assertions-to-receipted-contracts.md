---
base: 3539c6b6
head: d537239d
---

The remaining drug target and interaction checks now run in the routine drug
page, so the temporary `drug-live.md` page and all three live-registry entries
are gone. The shared provider fixture serves receipted ChEMBL and Open Targets
responses, validates the expected request identity, records GET and POST
traffic, and keeps unknown traffic fail-closed. The existing bounded DDInter
bundle remains the production local-data path.

Routine proof covers the EU regulatory plus target overlay in Markdown,
observed ChEMBL and Open Targets requests, DDInter source-empty wording, and
the structured coverage status. Existing source construction/decoder and
bundle tests remain intact. The expanded lifecycle test exercises both target
providers through local HTTP.

Verification passed: all ten drug page blocks and 49 fixture, receipt, and
runner-registry tests. No source lines were added against the 120-line ceiling.

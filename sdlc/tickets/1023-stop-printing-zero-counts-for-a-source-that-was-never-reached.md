---
flow: build
priority: 17
---
# Stop printing zero counts for a source that was never reached

When an optional source is unreachable, the Markdown body still opens with counts of zero and a sentence saying nothing was returned, and only corrects itself much further down. With CIViC pointed at a closed port, `get gene BRAF civic` prints this at lines 4 to 6:

```
- Evidence Items: 0
- Assertions: 0
No CIViC records returned for this gene query.
```

and then at line 23, below the `More:` and `See also:` navigation blocks, prints the truth:

```
**CIViC status (CIViC):** unavailable; no conclusion can be drawn — CIViC gene evidence is unavailable.
```

The honest line is present and the JSON contract is already correct, so this is not a missing capability. The defect is that the output asserts a false fact first and retracts it nineteen lines later, with two blocks of unrelated navigation in between. A person skimming, or an agent reading the response top to bottom, takes the zeros. For a gene like BRAF the wrong reading is "no clinical evidence exists," which is the most damaging conclusion this tool could lead someone to.

A genuinely empty result from a source that answered must remain clearly different from a source that did not answer, and both must be legible from the top of the section without reading to the end.

Disease survival has the same shape with two strings that are easy to mistake for each other: "SEER survival data not available for this condition." and "No SEER survival data available." Settle in the design which of those means absence and which means unavailability, and make the difference obvious rather than a matter of word order.

## The hard choice to settle

Either suppress the count lines entirely when the outcome is not `data`, or keep them and move the status line above them. Suppression is cleaner to read but removes a line some existing caller may parse; reordering is safer but leaves a reader briefly holding a zero. Pick one, apply it the same way to every section that reports a source outcome, and say in the design why.

## Done when

- A section whose outcome is `unavailable` does not present a count of zero, or a sentence asserting that nothing was returned, ahead of the statement that the source could not be reached.
- The unavailability statement appears within the section's own body, before any `More:` or `See also:` navigation block.
- A section whose outcome is genuinely `empty` still reads as a real, trustworthy zero and is not confusable with unavailability.
- The same rule holds for the CIViC gene section and the disease survival section, and any other section that reports a source outcome the same way.
- The JSON contract is unchanged: `data`, `empty`, `unavailable`, and `inapplicable` keep their current meanings and a failed source still receives no credit in the `sources` list.
- The MCP Markdown response an agent receives carries the same corrected ordering as the CLI.

## Reproduction

```
BIOMCP_CIVIC_BASE=http://127.0.0.1:9 biomcp get gene BRAF civic   # unavailable
biomcp get gene OR4F5 civic                                       # genuinely empty
```

Both exit 0 today, which is correct; the difference must be readable from the top of the section rather than from the exit code.

## Existing tests that pin this

The current wording and ordering are asserted in shipped tests. Restatement is authorized in these files, for these tests by name, only to the extent they assert that a no-records sentence or a zero count appears for a section whose outcome is not `data`:

- `src/render/markdown/gene/tests/rendering.rs` — `gene_markdown_section_only_shows_new_gene_enrichment_sections`, which asserts the literal sentences `No GTEx expression records returned`, `No Human Protein Atlas records returned`, `No DGIdb interactions returned`, and `No ClinGen records returned`
- `src/render/markdown/disease/tests/rendering.rs` and `src/render/markdown/disease/tests/extended.rs`, for the SEER survival wording only

No other test file is authorized. Assertions that a genuinely `empty` section reads as a real zero must survive unchanged — that is the distinction this ticket exists to protect. If the design stage finds a further shipped assertion that pins the old ordering and is not named above, say so in the design output rather than restating it.

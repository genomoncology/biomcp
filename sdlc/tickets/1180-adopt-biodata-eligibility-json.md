---
flow: build
priority: 10
waits-on: ["botassembly/biodata/0103"]
---
# Adopt BioData eligibility JSON

## Goal

The BioMCP trial response uses BioData's public `ClinicalTrialEligibility` JSON contract and retains no duplicate eligibility value codec.

## Current evidence

BioMCP stores `ClinicalTrialEligibility` directly but reconstructs its JSON through product-owned age, code, classification, criterion, and no-limit-rule types. BioData ticket 0103 supplies the missing narrow value codec. The arms-and-eligibility checkpoint passed against BioMCP `aafb52b7`.

## Required behavior

Update the exact BioData Git revision after ticket 0103 lands. Use BioData to encode and decode every populated eligibility value.

Keep BioMCP's outer optional-field behavior. Missing or null eligibility becomes `None` and stays omitted from output. A populated object becomes `Some(ClinicalTrialEligibility)`. Explicit empty nested collections remain distinct from absent nested collections.

Pass the original eligibility JSON object to BioData without converting it through an untyped value tree. Preserve duplicate members for BioData's strict validation. Map BioData failures to the current fixed, data-free product serialization error.

Delete BioMCP's duplicate eligibility encoding, age reconstruction, code conversion, classification conversion, NCI no-limit constant, and corresponding duplicate validation policy. BioMCP continues to own the trial response, section selection, errors, JSON and Markdown presentation, and eligibility search behavior.

## Done, observably

- Missing, null, all-null, explicit-empty, and fully populated eligibility values preserve the intended outer response states.
- Populated eligibility output exactly matches BioData's canonical value JSON.
- Invalid nested values and no-limit rules fail safely without source data in diagnostics.
- Recorded provider mutations still change output or produce the intended safe failure.
- A mechanical ownership check proves the retired product rule names and duplicate wire types are absent.
- The exact BioData revision assertion, lockfile, CLI JSON, MCP, Markdown, and full-feature gates pass.

## Boundaries

No reference-output change, search eligibility change, provider request change, new clinical model, release, or publication belongs here.

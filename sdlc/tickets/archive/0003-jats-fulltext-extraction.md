---
flow: build
priority: 8
---
# JATS-aware full-text extraction for PMC articles

BioMCP's `get article <PMID> fulltext` currently uses naive tag stripping that drops everything between `<` and `>`. This loses all document structure — headings, tables, figures, citations become a flat text blob mixed with XML metadata, author affiliations, and reference lists. Experiment 031 demonstrated that a JATS-aware converter preserves 15-33 headings and 11-18 tables per article while producing cleaner, smaller output than naive stripping. Trafilatura fails completely on 2 of 5 JATS XML files — a purpose-built JATS converter is the right approach.

Completed under March on 2026-03-18, as March ticket 003. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/003-jats-fulltext-extraction

The landed commit range could not be recovered from git, so no
record accompanies this entry. That is a known gap for the
earliest tickets, not a sign the work is missing.

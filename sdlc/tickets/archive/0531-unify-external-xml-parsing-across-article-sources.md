---
flow: build
priority: 8
---
# Unify external XML parsing across article sources

BioMCP currently has three answers to the same external XML condition. `transform/article/jats.rs` and `sources/ncbi_efetch.rs` each define a copy-pasted regex named `strip_doctype_declaration`; PubMed citation parsing instead enables DTD parsing directly with `roxmltree::ParsingOptions`. The PubMed implementation now works and is bounded, but the duplicated policies make it easy for the next XML-backed source to repeat the live failure fixed by ticket 529.

Completed under March on 2026-07-15, as March ticket 531. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/531-unify-external-xml-parsing-across-article-sources

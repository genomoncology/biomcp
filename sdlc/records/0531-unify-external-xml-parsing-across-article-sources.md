---
base: 90ea473536d785e91356870d26ce578cd67afe91
head: f19cbed5fc0ab30ba344b308a429d3e36a22e682
---
BioMCP currently has three answers to the same external XML condition. `transform/article/jats.rs` and `sources/ncbi_efetch.rs` each define a copy-pasted regex named `strip_doctype_declaration`; PubMed citation parsing instead enables DTD parsing directly with `roxmltree::ParsingOptions`. The PubMed implementation now works and is bounded, but the duplicated policies make it easy for the next XML-backed source to repeat the live failure fixed by ticket 529.

Imported from March ticket 531. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/531-unify-external-xml-parsing-across-article-sources

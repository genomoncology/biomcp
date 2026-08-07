---
base: 7c38b28b300b205ca87faae0e5b3890c4db52c2c
head: e5e50fb5bcca31b2d809fafc0b92233f4fd31ea3
---
BioMCP's PubTator-backed article annotations cover genes, diseases, chemicals, and point mutations but miss clinically important cytogenetic events. In the reported myeloma corpus, PMID 30709865 produced rich point-mutation entities, while PMID 35637217 exposed only RB1/TP53 and PMID 37449980 exposed no genes at all despite structural and copy-number events being central to the papers. This is largely an upstream ontology/NER gap. Shipping a few hard-coded translocation-to-gene mappings would repeat the curated-answer-key behavior BioMCP recently removed, so the next step must measure a general approach before adding a production surface.

Imported from March ticket 515. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/515-measure-a-structural-variant-annotation-layer-for-biomedical-articles

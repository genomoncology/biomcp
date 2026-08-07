---
flow: spike
priority: 5
---
# Measure a structural-variant annotation layer for biomedical articles

BioMCP's PubTator-backed article annotations cover genes, diseases, chemicals, and point mutations but miss clinically important cytogenetic events. In the reported myeloma corpus, PMID 30709865 produced rich point-mutation entities, while PMID 35637217 exposed only RB1/TP53 and PMID 37449980 exposed no genes at all despite structural and copy-number events being central to the papers. This is largely an upstream ontology/NER gap. Shipping a few hard-coded translocation-to-gene mappings would repeat the curated-answer-key behavior BioMCP recently removed, so the next step must measure a general approach before adding a production surface.

Completed under March on 2026-07-14, as March ticket 515. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/515-measure-a-structural-variant-annotation-layer-for-biomedical-articles

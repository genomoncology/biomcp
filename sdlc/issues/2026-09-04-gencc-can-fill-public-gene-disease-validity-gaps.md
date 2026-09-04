# GenCC can fill public gene-disease validity gaps

BioMCP 0.9.0-dev.6 returned an honest empty ClinGen section for ODC1 on 2026-09-04:

```console
$ ./target/debug/biomcp --json get gene ODC1 clingen
...
"clingen": {"outcome":"empty","sources":["ClinGen"]}
```

That result answers whether ClinGen has curated ODC1. It does not answer the broader clinical question: does a public curation group support the ODC1–Bachmann-Bupp syndrome relationship, under which inheritance pattern, and with which evidence?

The Gene Curation Coalition has a matching public assertion at `https://thegencc.org/submissions/SGC-113621.1`. Labcorp Genetics classified the ODC1 relationship to MONDO:0033642 as Strong with autosomal dominant inheritance. The record cites PMID 30239107 and PMID 30475435. GenCC publishes all non-OMIM submissions as CC0 bulk CSV and TSV files at `https://thegencc.org/download`. The download includes gene, disease, classification, mode of inheritance, submitter, evidence links, PMIDs, and dates. GenCC states that a query API is not available yet.

BioMCP should expose GenCC as a named gene-disease validity source. A caller must be able to distinguish a ClinGen absence from a broader absence across public curation sources. Results must remain submission-level assertions. BioMCP must preserve the submitter, disease identifier, inheritance, classification, evidence links, dates, and disagreements among submitters. It must not collapse the strongest submission into an unqualified consensus claim.

The most consistent CLI shape appears to be `get gene <symbol> gencc`. A later design pass should decide whether `get gene <symbol> clingen` stays source-specific and a new source-neutral validity section composes both sources. The bulk artifact needs a version, retrieval date, conditional refresh, local query index, and source-health status because the upstream service offers a download rather than a query API.

Recorded for triage.

# Source licensing review evidence, 2026-09-27/28 pass

Per-source evidence for the delegated licensing review pass recorded
in `sdlc/records/2026-09-27-source-licensing-review-pass.md`. Every
row names the URL checked, the access date, and the finding. The
correct split across the 48 sources due 2026-03-20: 42 unchanged, 2
changed (COSMIC, CPIC), 4 unverifiable (CIViC, Enrichr, WikiPathways,
and PharmGKB — the last reconciled to verified on 2026-09-28 against
the CPIC page, leaving 3 open). JS-walled pages were read from their
live JS bundles, which are the citable sources for those rows.

| Source | Evidence URL | Accessed | Finding |
|---|---|---|---|
| alphafold-db | https://alphafold.ebi.ac.uk/ | 2026-09-27 | unchanged — "Data is available for academic and commercial use, under a CC-BY-4.0 licence" |
| alphagenome | https://deepmind.google/science/alphagenome/ | 2026-09-27 | unchanged — free research portal; commercial via Cloud Model Garden, as recorded |
| cancer-genome-interpreter | https://www.cancergenomeinterpreter.org/website-terms-of-use | 2026-09-27 | unchanged (terms read from site JS bundle) — CGI Open research-only, CGI Industry commercial |
| cbioportal | https://www.cbioportal.org/api/studies | 2026-09-27 | unchanged — open API answered 548 studies; study-level downstream terms as recorded |
| chembl | https://www.ebi.ac.uk/chembl/ | 2026-09-27 | summary corrected — CC BY-SA 3.0, the ShareAlike obligation the earlier summary omitted |
| civic | https://docs.civicdb.org/en/latest/about/faq.html | 2026-09-28 | verified (2026-09-28 correction of the 09-27 call): the docs FAQ is plain HTML — "The content of CIViC ... is released under the Creative Commons Public Domain Dedication (CC0 1.0 Universal)"; research-purposes disclaimer rides with it |
| clingen | https://clinicalgenome.org/about/terms-of-use | 2026-09-27 | unchanged — CC0 1.0, attribution requested |
| clinicaltrials-gov | https://www.nlm.nih.gov/web_policies.html | 2026-09-27 | unchanged — US-government works not subject to copyright |
| clinvar | https://www.ncbi.nlm.nih.gov/clinvar/docs/maintenance_use/ | 2026-09-27 | unchanged — attribution requested on distribution |
| complexportal | https://www.ebi.ac.uk/about/terms-of-use | 2026-09-27 | unchanged — EBI umbrella governs (site JS-walled), no added restrictions |
| cosmic | https://www.cosmickb.org/licensing/ | 2026-09-27 | CHANGED — terms moved to cosmickb.org; academic use not-for-profit only; commercial licence required including patient services and clinical reporting; BioMCP reads only MyVariant's v68 snapshot (pre-licence-change, per MyVariant metadata) so the current clause does not attach to our fields |
| cpic | https://www.clinpgx.org/page/dataUsagePolicy | 2026-09-27 | CHANGED — cpicpgx.org redirects to ClinPGx, former CC0 pages gone; successor policy states CC BY-SA 4.0 (read from JS bundle); whether it governs CPIC content is not stated; recorded as attribution-plus-share-alike |
| dgidb | https://dgidb.org/api/graphql | 2026-09-27 | unchanged — live per-source license list; several upstream licenses non-commercial (CGI CC BY-NC 4.0), aggregation warning stands |
| disease-ontology | http://purl.obolibrary.org/obo/doid.owl | 2026-09-27 | unchanged — CC0 per header |
| disgenet | https://www.disgenet.com/ | 2026-09-27 | unchanged (terms read from JS bundle) — internal noncommercial purposes only |
| drugbank | https://trust.drugbank.com/drugbank-trust-center/drugbank-terms-of-service | 2026-09-27 | unchanged — commercial use requires a commercial license |
| drugs-at-fda | https://www.fda.gov/about-fda/about-website/website-policies | 2026-09-27 | unchanged — public domain, credit appreciated |
| enrichr | https://maayanlab.cloud/Enrichr/help | 2026-09-27 | unverifiable — terms loader script 404s provider-side for every visitor; retry next pass; date NOT moved |
| europe-pmc | https://www.ebi.ac.uk/europepmc/webservices/rest/search | 2026-09-27 | unchanged — open REST, no key; ELIXIR core resource |
| gnomad | https://gnomad.broadinstitute.org/policies | 2026-09-27 | unchanged — policies page JS-walled; open bucket `gcp-public-data--gnomad` confirms open summary data |
| gprofiler | https://biit.cs.ut.ee/gprofiler/help.cgi | 2026-09-27 | unchanged (docs read from JS bundle) — open to all users free of charge; cite when used |
| gtex | https://gtexportal.org/ | 2026-09-27 | unchanged (rendered from JS bundle) — acknowledge the portal; republishers keep data current |
| gwas-catalog | https://www.ebi.ac.uk/gwas/docs/about/ | 2026-09-27 | unchanged — EBI ToU; summary statistics CC0 unless stated |
| hpo-jax-api | https://human-phenotype-ontology.github.io/license.html | 2026-09-27 | unchanged — three-condition license (cite, version, no content alteration) |
| interpro | https://www.ebi.ac.uk/about/terms-of-use | 2026-09-27 | unchanged — EBI ToU is the plain-HTML authority; terms_url repointed to it |
| kegg | https://www.kegg.jp/kegg/legal.html | 2026-09-27 | unchanged — academic free; non-academic needs a commercial license |
| medlineplus | https://medlineplus.gov/about/using/usingcontent/ | 2026-09-27 | unchanged — non-copyrighted content reproducible; acknowledge the source |
| monarch-initiative | https://github.com/monarch-initiative/monarch-app/blob/main/docs/Licensing/index.md | 2026-09-27 | unchanged — per-repo LICENSE precedence over general CC0/CC-BY guidance |
| mondo | https://mondo.monarchinitiative.org/pages/download/ | 2026-09-27 | unchanged — CC BY 4.0 |
| mychem-info | https://mychem.info/ | 2026-09-27 | unchanged — BioThings footer research-purposes disclaimer noted in reuse guidance |
| mydisease-info | https://mydisease.info/ | 2026-09-27 | unchanged — same BioThings footer |
| mygene-info | https://mygene.info/ | 2026-09-27 | unchanged — same BioThings footer |
| myvariant-info | http://myvariant.info/metadata | 2026-09-27 | unchanged — COSMIC fields are the v68 snapshot ("the last freely available ... before their licence change"), per the API's own metadata |
| ncbi-id-converter | https://pmc.ncbi.nlm.nih.gov/tools/idconv/ | 2026-09-27 | unchanged — US-government service |
| nci-cts | https://clinicaltrialsapi.cancer.gov/ | 2026-09-27 | unchanged in terms — API docs moved behind the developer-accounts app; /v1 not served unauthenticated on the public host; key-required model as recorded |
| ols4 | https://www.ebi.ac.uk/about/terms-of-use | 2026-09-27 | unchanged — EBI ToU; terms_url repointed |
| oncokb | https://faq.oncokb.org/licensing | 2026-09-27 | unchanged — license required for commercial/clinical/programmatic use; free academic research |
| opentargets | https://platform-docs.opentargets.org/licence | 2026-09-27 | unchanged — CC0 1.0 data; sources agreed to unrestricted use |
| pdb | https://www.rcsb.org/pages/usage-policy | 2026-09-27 | unchanged — CC0 for archive and API data |
| pharmgkb | https://www.clinpgx.org/page/dataUsagePolicy | 2026-09-28 | reconciled — own domain gone; verified against the same ClinPGx policy page as CPIC (JS bundle): CC BY-SA 4.0 |
| pubtator3 | https://www.nlm.nih.gov/web_policies.html | 2026-09-27 | unchanged — US-government works |
| quickgo | https://www.ebi.ac.uk/about/terms-of-use | 2026-09-27 | unchanged — EBI ToU plus GO CC BY 4.0 citation policy |
| reactome | https://reactome.org/license | 2026-09-27 | unchanged — CC BY 4.0 content, CC0 database and derived files |
| semantic-scholar | https://www.semanticscholar.org/product/api/license | 2026-09-27 | unchanged — S2 Data governed by accompanying licenses (CC BY-NC or ODC-BY); attribution required |
| string | https://string-db.org/cgi/access?footer_active_subpage=licensing | 2026-09-27 | unchanged — CC BY 4.0 with credit |
| umls | https://www.nlm.nih.gov/databases/umls.html | 2026-09-27 | unchanged — individual license, UTS account required |
| uniprot | https://rest.uniprot.org/uniprotkb/P12345.txt | 2026-09-27 | unchanged — CC BY 4.0 (stated in the live REST entry text) |
| wikipathways | https://www.wikipathways.org/terms.html | 2026-09-28 | verified (2026-09-28 correction of the 09-27 call): the current site's terms page is live and states WikiPathways "decided to adopt the Creative Commons CC0 waiver for our content" |

Open items: only Enrichr keeps `reviewed_on` 2026-03-20 (its terms
loader 404s provider-side for every visitor). The guard warns after
300 days — 2027-01-15 — and fails on 2027-03-21 for that one source.
Owner of the retry: the developer agent working Ian's ordered
BioMCP queue (the role that ran this pass), trigger the 2027-01-15
warning. Until verified, the 2027-03 crossing is NOT handled for
Enrichr; the pass handles it for the other 47.

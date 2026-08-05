# Source Versioning Matrix

This matrix tracks which upstream API endpoints are version-pinned and where unversioned endpoints remain intentional.

| Source | Base URL | Version status | Rationale | Last reviewed |
|---|---|---|---|---|
| AlphaGenome | `https://gdmscience.googleapis.com:443` | Unversioned | Google endpoint is service-versioned server-side; no stable path segment exposed | 2026-02-15 |
| cBioPortal | `https://www.cbioportal.org/api` | Unversioned | Public API path is stable without explicit version segment | 2026-02-15 |
| Cancerhotspots.org | `https://www.cancerhotspots.org` | Unversioned | Website-backed recurrence endpoint is stable without explicit URL versioning | 2026-06-11 |
| CDC WONDER VAERS | `https://wonder.cdc.gov/controller/datarequest/D8` | Unversioned | CDC WONDER exposes the VAERS D8 database through a stable dataset ID and XML POST contract; BioMCP freezes the request/response shape in fixtures instead of relying on a versioned path segment | 2026-04-18 |
| CDC CVX | `https://www2.cdc.gov/vaccines/iis/iisstandards/downloads/cvx.txt` | Unversioned file export | CDC publishes current CVX codes at a stable filename that refreshes in place | 2026-06-11 |
| CDC MVX | `https://www2.cdc.gov/vaccines/iis/iisstandards/downloads/mvx.txt` | Unversioned file export | CDC publishes current MVX manufacturer codes at a stable filename that refreshes in place | 2026-06-11 |
| CDC vaccine trade names | `https://www2.cdc.gov/vaccines/iis/iisstandards/downloads/TRADENAME.txt` | Unversioned file export | CDC publishes current trade-name mapping at a stable filename that refreshes in place | 2026-06-11 |
| ChEMBL | `https://www.ebi.ac.uk/chembl/api/data` | Unversioned | ChEMBL data API is stable at `/api/data`; no URL version convention | 2026-02-15 |
| ClinicalTrials.gov | `https://clinicaltrials.gov/api/v2` | Versioned (`v2`) | Endpoint already pinned to public v2 API | 2026-02-15 |
| ClinGen Search | `https://search.clinicalgenome.org` | Unversioned | Public search endpoint has no stable version path segment | 2026-06-11 |
| ClinGen Allele Registry | `https://reg.genome.network` | Request template v1; response version provenance | The read-only projection is part of lookup identity; nullable `X-CAR-Version` is post-response provenance, never a cache key | 2026-07-24 |
| ClinGen CSpec | `https://cspec.clinicalgenome.org` | Versioned resource documents | Full resource IRI is the selection identity; provider display version remains separate | 2026-07-23 |
| ClinGen ERepo | `https://erepo.clinicalgenome.org` | Versioned detail documents | Exact `docVersion` is required in the SEPIO detail route | 2026-07-23 |
| ClinGen LDH | `https://ldh.genome.network` | Unversioned routes; request/response subset provenance | BioMCP validates the fixed medium and direct routes, then records template and response-subset provenance after receipt rather than using it as a lookup key | 2026-07-26 |
| ComplexPortal | `https://www.ebi.ac.uk/intact/complex-ws` | Unversioned | EBI complex web service is path-stable without URL versioning | 2026-06-11 |
| CPIC | `https://api.cpicpgx.org/v1` | Versioned (`v1`) | Endpoint already pinned | 2026-06-11 |
| CIViC | `https://civicdb.org/api` | Unversioned | Public API root is stable without explicit major version path | 2026-06-11 |
| DDInter downloads | `https://ddinter.scbdd.com/download/` | Unversioned download bundle | Public CSV bundle refreshes in place under stable filenames | 2026-06-11 |
| DGIdb GraphQL | `https://dgidb.org/api/graphql` | Unversioned | GraphQL endpoint has no URL version segment | 2026-06-11 |
| DisGeNET | `https://api.disgenet.com/api/v1` | Versioned (`v1`) | Endpoint already pinned | 2026-06-11 |
| Enrichr | `https://maayanlab.cloud/Enrichr` | Unversioned | Service does not publish versioned path variant for current API | 2026-02-15 |
| EMA human medicines data | `https://www.ema.europa.eu/en/about-us/about-website/download-website-data-json-data-format` | Unversioned download page | EMA publishes current JSON batch links from a stable page | 2026-06-11 |
| Europe PMC | `https://www.ebi.ac.uk/europepmc/webservices/rest` | Unversioned | REST root is stable and not versioned in URL | 2026-02-15 |
| gnomAD GraphQL | `https://gnomad.broadinstitute.org/api` | Unversioned | Versioning is dataset-level (`gnomad_r4`, `gnomad_r3`, `gnomad_r2_1`) in query payload | 2026-02-15 |
| g:Profiler | `https://biit.cs.ut.ee/gprofiler/api` | Unversioned | Public endpoint does not expose version path segment | 2026-02-15 |
| GTEx Portal | `https://gtexportal.org/api/v2` | Versioned (`v2`) | Endpoint already pinned | 2026-06-11 |
| GWAS Catalog | `https://www.ebi.ac.uk/gwas/rest/api` | Unversioned | REST root is stable and has no exposed version segment | 2026-06-11 |
| HPO JAX API | `https://ontology.jax.org/api/hp` | Unversioned | API path is canonical and currently unversioned | 2026-02-15 |
| Human Protein Atlas | `https://www.proteinatlas.org` | Unversioned | Public endpoint is website-backed with stable path contracts rather than API versioning | 2026-06-11 |
| InterPro | `https://www.ebi.ac.uk/interpro/api` | Unversioned | Public endpoint has no URL versioning model | 2026-02-15 |
| KEGG REST | `https://rest.kegg.jp` | Unversioned | KEGG REST exposes stable operation paths without major versioning | 2026-06-11 |
| LitSense2 | `https://www.ncbi.nlm.nih.gov/research/litsense2-api/api` | Versioned-by-product (`litsense2-api`) | Version identity is in the product namespace rather than the path suffix | 2026-04-10 |
| MedlinePlus Search | `https://wsearch.nlm.nih.gov/ws/query` | Unversioned | NLM search endpoint is stable without URL versioning | 2026-06-11 |
| Monarch Initiative API v3 | `https://api-v3.monarchinitiative.org` | Versioned (`v3`) | Version identity is in the host name | 2026-06-11 |
| Mutalyzer | `https://mutalyzer.nl/api` | Unversioned | Public API root has no versioned path for current normalize calls | 2026-06-11 |
| MyChem.info | `https://mychem.info/v1` | Versioned (`v1`) | Endpoint already pinned | 2026-02-15 |
| MyDisease.info | `https://mydisease.info/v1` | Versioned (`v1`) | Endpoint already pinned | 2026-02-15 |
| MyGene.info | `https://mygene.info/v3` | Versioned (`v3`) | Endpoint already pinned | 2026-02-15 |
| MyVariant.info | `https://myvariant.info/v1` | Versioned (`v1`) | Endpoint already pinned | 2026-02-15 |
| NCBI Genetic Testing Registry | `https://ftp.ncbi.nlm.nih.gov/pub/GTR/data` | Unversioned bulk export | GTR bulk files are published at stable filenames and refreshed in place rather than by versioned path | 2026-04-17 |
| NCBI GTR condition-gene file | `https://ftp.ncbi.nlm.nih.gov/pub/GTR/data/test_condition_gene.txt` | Unversioned file export | Stable filename refreshed in place under the GTR bulk root | 2026-06-11 |
| NCBI GTR test-version file | `https://ftp.ncbi.nlm.nih.gov/pub/GTR/data/test_version.gz` | Unversioned file export | Stable filename refreshed in place under the GTR bulk root | 2026-06-11 |
| WHO Prequalified IVD | `https://extranet.who.int/prequal/vitro-diagnostics/prequalified/in-vitro-diagnostics/export?page&_format=csv` | Unversioned export | WHO IVD publishes a stable CSV export path that refreshes in place rather than exposing a versioned endpoint | 2026-04-18 |
| NCBI ID Converter | `https://pmc.ncbi.nlm.nih.gov/tools/idconv/api/v1/articles` | Versioned (`v1`) | Endpoint already pinned | 2026-02-15 |
| NCI CTS | `https://clinicaltrialsapi.cancer.gov/api/v2` | Versioned (`v2`) | Endpoint already pinned | 2026-02-15 |
| NIH Reporter | `https://api.reporter.nih.gov/v2` | Versioned (`v2`) | Endpoint already pinned to the public NIH Reporter v2 project search API | 2026-04-11 |
| OncoKB (prod/demo) | `https://www.oncokb.org/api/v1` / `https://demo.oncokb.org/api/v1` | Versioned (`v1`) | Endpoint already pinned | 2026-02-15 |
| OpenFDA | `https://api.fda.gov` | Unversioned | Public OpenFDA API is path-stable without version segment | 2026-02-15 |
| OpenFDA device 510(k) | `https://api.fda.gov/device/510k.json` | Unversioned | OpenFDA resource path is stable without URL versioning | 2026-06-11 |
| OpenFDA device PMA | `https://api.fda.gov/device/pma.json` | Unversioned | OpenFDA resource path is stable without URL versioning | 2026-06-11 |
| OpenTargets | `https://api.platform.opentargets.org/api/v4/graphql` | Versioned (`v4`) | Endpoint already pinned | 2026-02-15 |
| OLS4 | `https://www.ebi.ac.uk/ols4` | Versioned-by-product (`ols4`) | Version identity is in the product namespace | 2026-06-11 |
| PharmGKB | `https://api.pharmgkb.org/v1` | Versioned (`v1`) | Endpoint already pinned | 2026-06-11 |
| PMC HTML | `https://pmc.ncbi.nlm.nih.gov/articles` | Unversioned | NCBI article HTML route is stable and not API-versioned | 2026-06-11 |
| PMC OA | `https://pmc-oa-opendata.s3.amazonaws.com` | Versioned metadata objects | A version-prefix listing selects its flat versioned metadata object, which names durable XML/media objects; the retired FTP/archive files are removed in August 2026 | 2026-08-05 |
| PubMed | `https://eutils.ncbi.nlm.nih.gov/entrez/eutils` | Unversioned | PubMed search still uses legacy E-utilities endpoints without explicit path versioning | 2026-04-10 |
| PubTator3 | `https://www.ncbi.nlm.nih.gov/research/pubtator3-api` | Versioned-by-product (`pubtator3`) | Version identity is in product namespace | 2026-02-15 |
| QuickGO | `https://www.ebi.ac.uk/QuickGO/services` | Unversioned | Service endpoint is canonical and not path-versioned | 2026-02-15 |
| Reactome Content Service | `https://reactome.org/ContentService` | Unversioned | No explicit major version path in public endpoint | 2026-02-15 |
| SEER Explorer | `https://seer.cancer.gov/statistics-network/explorer/source/content_writers` | Unversioned | Undocumented PHP endpoints have no stable version segment; BioMCP validates requested site codes and decoded payload structure | 2026-04-10 |
| Semantic Scholar | `https://api.semanticscholar.org` | Unversioned | Public API base is stable without a version segment; endpoint versions live below the base path | 2026-03-15 |
| STRING | `https://string-db.org/api` | Unversioned | API route uses format path segment; no stable version URL segment | 2026-02-15 |
| UniProt REST | `https://rest.uniprot.org` | Unversioned | REST base is canonical and not versioned in URL | 2026-02-15 |
| UMLS REST | `https://uts-ws.nlm.nih.gov/rest` | Unversioned | UMLS REST path is stable without explicit major version segment | 2026-06-11 |
| VariantValidator | `https://rest.variantvalidator.org` | Unversioned | Public REST root has no URL version path for current variant normalization | 2026-06-11 |
| WHO active pharmaceutical ingredients | `https://extranet.who.int/prequal/medicines/prequalified/active-pharmaceutical-ingredients/export?page&_format=csv` | Unversioned export | WHO export refreshes in place behind a stable CSV URL | 2026-06-11 |
| WHO finished pharmaceutical products | `https://extranet.who.int/prequal/medicines/prequalified/finished-pharmaceutical-products/export?page&_format=csv` | Unversioned export | WHO export refreshes in place behind a stable CSV URL | 2026-06-11 |
| WHO vaccines | `https://extranet.who.int/prequal/vaccines/prequalified/export` | Unversioned export | WHO vaccine export refreshes in place behind a stable URL | 2026-06-11 |
| WikiPathways | `https://www.wikipathways.org/json` | Unversioned | JSON endpoint has stable route names rather than a public version segment | 2026-06-11 |

## Notes

- CDC WONDER VAERS intentionally stays out of `./scripts/contract-smoke.sh`: the
  D8 contract is POST/XML, relatively volatile, and already covered by the
  real-query health row plus fixture-frozen unit/spec tests.
- gnomAD versioning is handled by dataset selection in GraphQL variables and is verified by dataset fallback tests.

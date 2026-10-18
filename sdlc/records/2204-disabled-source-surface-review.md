# Disabled-source surface and terms review

Review base: `3dc15da27888454aac4a4e43b685f1ebabbbeff5`, 2026-10-18. This bounded review found two reporting defects and no observed request to a disabled provider. It does not establish complete route coverage or unrestricted commercial rights.

## Runtime evidence

Linux job `2204/1792353287357-15590` completed the prepared matrix: 596 commands, comprising 366 CLI commands and 230 typed MCP calls. The routine CLI was built once with `env -u CARGO_INCREMENTAL tools/with-build-identity cargo build --locked --no-default-features --bin biomcp`. The existing `tools/run-offline` verified blocked public DNS/TCP and available loopback. Banner: `biomcp 0.9.4-dev+g3dc15da2`; binary SHA-256: `64f092dd13d99d3b1ed16cf3350bfe9698e54ed8ddd0863a0ee7262a49b7a02b`.

The matrix derives 81 source IDs, 62 remote health descriptors, 15 searchable entities, 13 gettable entities and their sections from existing inventories and the actual CLI. It exercises each source enabled/disabled, individual disabled remote health rows, all-off health through both environment and flag, CLI sections with identity providers retained, typed get with sources enabled/disabled, all eight typed search entities and four specialized tools. Each case retains its request, exit/reply, output and replay trace. Collection success means the driver finished; error replies remain evidence gaps or findings.

All 62 individual disabled remote health probes and both all-off health variants made zero traced requests. For 56 source-walk IDs, the enabled command reached the registered provider prefix and the disabled command did not. Across traced section calls, no prefix belonging to a disabled provider was observed. The 1,428 requests comprise 304 replay hits and 1,124 missing replies, so a reached request does not prove a successful provider response.

The other 25 source-walk IDs had no reached provider baseline: `cosmic`, `cancer-genome-interpreter`, `alphagenome`, `clingen_ldh`, `litvar2`, `ncbi-e-utilities`, `ncbi-id-converter`, `figshare`, `pmc-oa`, `opencitations`, `pdb`, `alphafold-db`, `mondo`, `disease-ontology`, `chembl`, `fda-orphan-designations`, `ddinter`, `vaers`, `drugbank`, `drugs-at-fda`, `pharmgkb`, `gencc`, `pharmacodb`, `depmap`, `cell-model-passports`. These include carried identifiers/fields, local stores, unavailable features and chains stopped by missing replies. They do not count as proven downstream execution.

Of 110 CLI default/section calls, 51 returned success, 47 returned an error and 12 rejected the request. The article `asset` selector needs an asset key; the pathway sample lacks a replay reply; the original adverse-event sample was a drug name instead of a numeric report ID. The typed get matrix similarly includes 100 successful replies and 118 tool errors. No timeout or driver exception occurred. Existing qualified 2196 and 2207 regression evidence remains applicable to unchanged disabled-section, carried-field, article-skip and override-origin behavior; this review does not replace it.

One corrective invocation, job `2204/1792353607541-25045`, reused the same binary and corrected only the adverse-event ID and typed variant-literature field. `get adverse-event 1001` with all sources disabled returned `source_unavailable`, explicitly naming `BIOMCP_DISABLE_SOURCES=openfda`, with no requests. The typed literature correction produced the second finding below. A duplicate submission refused before executing cases because the correction output directory already existed; no matrix was repeated.

## Concrete findings

1. **2221: Disabled local health rows claim errors.** With every source ID in `BIOMCP_DISABLE_SOURCES`, `biomcp --json health` returns five `error` rows for EMA, CDC CVX/MVX, WHO Prequalification, GTR and WHO IVD. Each lacks `disabled_reason`; DDInter correctly returns `disabled`. The flag form behaves the same. Matrix cases 225 and 226 contain zero requests, `error: 5`, and the full row outputs. Shared cause: `src/cli/health/local.rs` checks the disabled setting for DDInter but directly inspects the other five stores. A repair should apply the same source-ID decision before those local probes and prove both setting origins.
2. **2222: Typed variant literature calls disabled resolution a provider error.** With all sources disabled, `variant_articles` arguments `{"items":[{"gene":"BRAF","protein":"V600E"}],"limit":1}` return `isError: true`, `complete: false`, `truncated: true`, resolution source `myvariant` status `unavailable` and `reason_codes: ["provider_error"]`; no setting is named and no request occurs. Correction case 2 records it. Inspect the shared resolution in `src/entities/variant/search/mod.rs` and status mapping in `src/entities/article/variant_search.rs`; preserve identity rules while distinguishing an operator-disabled source from provider failure. A separate repair must check the CLI and typed MCP surfaces.

Follow-up tickets 2221 and 2222 cover these runtime findings. This review changes no production code, fixtures, source registry or license groups.

## Terms evidence

The prepared assessment examined every registry row and its recorded primary terms URL on 2026-10-18. Missing pages, JavaScript shells, generic descriptions and ambiguous agreements stay unverified. A null `commercial_terms` field does not establish unrestricted rights. The table preserves those limits instead of treating public access or a code license as a data license.

ORCID's public/anonymous API restriction found in the initial assessment is already corrected in the review base by 2216; [Public API terms §2](https://info.orcid.org/public-client-terms-of-service/) distinguish API service access from public-data reuse. Reactome's database-data wording remains covered by 2217: [License §1](https://reactome.org/license) assigns CC0 to database data/derived files, CC BY to art and separate licenses to software. No duplicate fix is proposed.

| Source | Assessment | Recorded primary reference |
| --- | --- | --- |
| `alphafold-db` | Prior registry reading retained; current page missing/shell/generic or not independently sufficient for data-rights verification. | [Provider reference](https://alphafold.ebi.ac.uk/faq) |
| `alphagenome` | Confirmed hosted-output non-commercial restriction; downloadable/cloud exceptions differ. | [Provider reference](https://google-deepmind-alphagenome.readthedocs-hosted.com/faqs.html#what-terms-of-use-apply-to-alphagenome-outputs) |
| `cancer-genome-interpreter` | Prior restricted classification retained; current terms retrieval failed. | [Provider reference](https://www.cancergenomeinterpreter.org/conditions) |
| `cancerhotspots` | UNVERIFIED: provider page empty; public endpoint is not a reuse grant. | [Provider reference](https://www.cancerhotspots.org/) |
| `cbioportal` | Prior registry reading retained; current page missing/shell/generic or not independently sufficient for data-rights verification. | [Provider reference](https://www.cbioportal.org/) |
| `cdc-cvx` | Confirmed government public-domain baseline with attribution/disclaimer and exceptions. | [Provider reference](https://www.cdc.gov/other/agencymaterials.html) |
| `cell-model-passports` | Identifier-only via Cellosaurus; restricted linked portal policy does not establish acquired dataset rights. | [Provider reference](https://depmap.sanger.ac.uk/documentation/data-usage-policy/) |
| `cellosaurus` | Confirmed CC BY4; preserve attribution. | [Provider reference](https://www.cellosaurus.org/description.html) |
| `chembl` | Confirmed CC BY-SA3; preserve attribution and ShareAlike. | [Provider reference](https://www.ebi.ac.uk/chembl/) |
| `civic` | Confirmed content CC0; code license separate. | [Provider reference](https://docs.civicdb.org/en/latest/about/faq.html) |
| `clingen` | Confirmed curated content CC0; attribution/date requested. | [Provider reference](https://clinicalgenome.org/) |
| `clingen_car` | ClinGen CC0 curated-content umbrella confirmed; attachments/linked resources need their own rights. | [Provider reference](https://reg.genome.network) |
| `clingen_cspec` | ClinGen CC0 curated-content umbrella confirmed; attachments/linked resources need their own rights. | [Provider reference](https://cspec.clinicalgenome.org) |
| `clingen_erepo` | ClinGen CC0 curated-content umbrella confirmed; attachments/linked resources need their own rights. | [Provider reference](https://erepo.clinicalgenome.org) |
| `clingen_ldh` | ClinGen CC0 curated-content umbrella confirmed; attachments/linked resources need their own rights. | [Provider reference](https://ldh.genome.network) |
| `clinicaltrials-gov` | Government service baseline retained; source-specific/third-party content rights not waived by public access. | [Provider reference](https://clinicaltrials.gov/data-api/about-api) |
| `clinvar` | Confirmed attribution requested; generic NCBI/third-party caveats retained. | [Provider reference](https://www.ncbi.nlm.nih.gov/clinvar/docs/maintenance_use/) |
| `complexportal` | Confirmed EBI umbrella adds no owner restrictions; specific underlying data terms still apply. | [Provider reference](https://www.ebi.ac.uk/complexportal/) |
| `cosmic` | Historical MyVariant v68 scope recorded; current terms do not prove historical rights; metadata URL inaccessible. | [Provider reference](https://docs.myvariant.info/en/latest/doc/data_source.html) |
| `cpic` | JS policy page; prior successor CC BY-SA4 reading retained; CPIC-specific applicability unresolved. | [Provider reference](https://www.clinpgx.org/page/dataUsagePolicy) |
| `dbsnp` | Confirmed NCBI no imposed data restrictions, but submitter/third-party rights not transferred. | [Provider reference](https://www.ncbi.nlm.nih.gov/home/about/policies/) |
| `ddinter` | Confirmed data CC BY-NC-SA4. | [Provider reference](https://ddinter.scbdd.com/terms/) |
| `depmap` | Identifier-only via Cellosaurus; file-specific restrictions and commercial licenses remain downstream. | [Provider reference](https://forum.depmap.org/t/license-for-data-found-in-the-depmap-portal/130) |
| `dgidb` | Aggregator/service description accessible; original provider licenses remain; no blanket relicensing verified. | [Provider reference](https://www.dgidb.org/about) |
| `disease-ontology` | Prior registry reading retained; current page missing/shell/generic or not independently sufficient for data-rights verification. | [Provider reference](https://disease-ontology.org/) |
| `disgenet` | Confirmed commercial license and redistribution prohibition absent custom permission. | [Provider reference](https://support.disgenet.com/support/solutions/articles/202000087487-do-i-need-a-commercial-license-can-i-use-disgenet-data-in-my-product-) |
| `drugbank` | Confirmed commercial license; indirect delivery does not establish downstream permission. | [Provider reference](https://trust.drugbank.com/drugbank-trust-center/drugbank-terms-of-service) |
| `drugs-at-fda` | Government service baseline retained; source-specific/third-party content rights not waived by public access. | [Provider reference](https://open.fda.gov/apis/drug/drugsfda/) |
| `ema` | Confirmed commercial/non-commercial reuse with attribution; third-party exceptions. | [Provider reference](https://www.ema.europa.eu/en/about-us/about-website/legal-notice) |
| `enrichr` | Prior2026-09-29 verified commercial-license restriction retained; current URL fetch failed. | [Provider reference](https://maayanlab.cloud/Enrichr/templates/help/terms-submenu.html) |
| `europe-pmc` | Prior registry reading retained; current page missing/shell/generic or not independently sufficient for data-rights verification. | [Provider reference](https://europepmc.org/RestfulWebService) |
| `fda-orphan-designations` | Government service baseline retained; source-specific/third-party content rights not waived by public access. | [Provider reference](https://www.fda.gov/about-fda/about-website/website-policies) |
| `figshare` | Prior registry reading retained; current page missing/shell/generic or not independently sufficient for data-rights verification. | [Provider reference](https://figshare.com/terms) |
| `gencc` | Prior registry reading retained; current page missing/shell/generic or not independently sufficient for data-rights verification. | [Provider reference](https://thegencc.org/terms) |
| `gnomad` | Prior registry reading retained; current page missing/shell/generic or not independently sufficient for data-rights verification. | [Provider reference](https://gnomad.broadinstitute.org/policies) |
| `gprofiler` | Prior registry reading retained; current page missing/shell/generic or not independently sufficient for data-rights verification. | [Provider reference](https://biit.cs.ut.ee/gprofiler/help.cgi) |
| `gtex` | Prior registry reading retained; current page missing/shell/generic or not independently sufficient for data-rights verification. | [Provider reference](https://gtexportal.org/home/documentationPage) |
| `gtr` | Government service baseline retained; source-specific/third-party content rights not waived by public access. | [Provider reference](https://www.ncbi.nlm.nih.gov/gtr/docs/faq/) |
| `gwas-catalog` | Confirmed EBI umbrella; summary-statistics CC0 unless dataset says otherwise. | [Provider reference](https://www.ebi.ac.uk/gwas/docs/about) |
| `hpo-jax-api` | Confirmed free use with acknowledgement, version and integrity conditions; not generic CC license. | [Provider reference](https://human-phenotype-ontology.github.io/license.html) |
| `human-protein-atlas` | Provider page accessible; existing CC BY4 reading retained; copyrightable/third-party scope matters. | [Provider reference](https://www.proteinatlas.org/about/licence) |
| `interpro` | Confirmed EBI umbrella adds no owner restrictions; specific underlying data terms still apply. | [Provider reference](https://www.ebi.ac.uk/about/terms-of-use) |
| `kegg` | Confirmed non-academic commercial license requirement. | [Provider reference](https://www.kegg.jp/kegg/legal.html) |
| `litsense2` | Government service baseline retained; source-specific/third-party content rights not waived by public access. | [Provider reference](https://www.ncbi.nlm.nih.gov/research/litsense2-api/) |
| `litvar2` | Government service baseline retained; source-specific/third-party content rights not waived by public access. | [Provider reference](https://www.ncbi.nlm.nih.gov/research/litvar2-api/) |
| `medlineplus` | Prior registry reading retained; current page missing/shell/generic or not independently sufficient for data-rights verification. | [Provider reference](https://medlineplus.gov/about/using/) |
| `monarch-initiative` | Aggregator/service description accessible; original provider licenses remain; no blanket relicensing verified. | [Provider reference](https://monarchinitiative.org/) |
| `mondo` | Confirmed CC BY4. | [Provider reference](https://mondo.monarchinitiative.org/pages/download/) |
| `mutalyzer` | UNVERIFIED: JavaScript shell; service availability is not data rights. | [Provider reference](https://mutalyzer.nl/) |
| `mychem-info` | Aggregator/service description accessible; original provider licenses remain; no blanket relicensing verified. | [Provider reference](https://docs.mychem.info/en/latest/) |
| `mydisease-info` | Aggregator/service description accessible; original provider licenses remain; no blanket relicensing verified. | [Provider reference](https://docs.mydisease.info/en/latest/) |
| `mygene-info` | Aggregator/service description accessible; original provider licenses remain; no blanket relicensing verified. | [Provider reference](https://docs.mygene.info/en/latest/) |
| `myvariant-info` | Aggregator/service description accessible; original provider licenses remain; no blanket relicensing verified. | [Provider reference](https://docs.myvariant.info/en/latest/) |
| `ncbi-e-utilities` | Government service baseline retained; source-specific/third-party content rights not waived by public access. | [Provider reference](https://www.ncbi.nlm.nih.gov/books/NBK25501/) |
| `ncbi-id-converter` | Government service baseline retained; source-specific/third-party content rights not waived by public access. | [Provider reference](https://pmc.ncbi.nlm.nih.gov/tools/idconv/) |
| `nci-cts` | UNVERIFIED: developer-account shell; no current specific API terms readable. | [Provider reference](https://clinicaltrialsapi.cancer.gov/) |
| `nih-reporter` | Government service baseline retained; source-specific/third-party content rights not waived by public access. | [Provider reference](https://www.nih.gov/web-policies-notices) |
| `ols4` | Confirmed EBI umbrella adds no owner restrictions; specific underlying data terms still apply. | [Provider reference](https://www.ebi.ac.uk/about/terms-of-use) |
| `oncokb` | Confirmed commercial/clinical paid license; academic programmatic access also licensed. | [Provider reference](https://faq.oncokb.org/licensing) |
| `opencitations` | Prior registry reading retained; current page missing/shell/generic or not independently sufficient for data-rights verification. | [Provider reference](https://opencitations.net/) |
| `openfda` | Government service baseline retained; source-specific/third-party content rights not waived by public access. | [Provider reference](https://open.fda.gov/apis/authentication/) |
| `opentargets` | Prior registry reading retained; current page missing/shell/generic or not independently sufficient for data-rights verification. | [Provider reference](https://platform-docs.opentargets.org/licence) |
| `orcid` | Public/anonymous API restriction confirmed; corrected in review base by 2216. | [Provider reference](https://info.orcid.org/public-client-terms-of-service/) |
| `pdb` | Confirmed archive and API data CC0; website text/art separate. | [Provider reference](https://www.rcsb.org/pages/usage-policy) |
| `pharmacodb` | UNRESOLVED: provider shell supplies no data terms; paper/code licenses do not establish data rights. | No recorded URL |
| `pharmgkb` | JS policy page; prior successor ClinPGx CC BY-SA4 reading retained; no new current text verified. | [Provider reference](https://www.clinpgx.org/page/dataUsagePolicy) |
| `pmc-oa` | Confirmed article-specific licenses; OA is not uniformly commercial permission. | [Provider reference](https://pmc.ncbi.nlm.nih.gov/tools/openftlist/) |
| `pubmed` | Government service baseline retained; source-specific/third-party content rights not waived by public access. | [Provider reference](https://www.ncbi.nlm.nih.gov/books/NBK25501/) |
| `pubtator3` | Government service baseline retained; source-specific/third-party content rights not waived by public access. | [Provider reference](https://www.ncbi.nlm.nih.gov/research/pubtator3/api) |
| `quickgo` | Confirmed EBI umbrella adds no owner restrictions; specific underlying data terms still apply. | [Provider reference](https://www.ebi.ac.uk/about/terms-of-use) |
| `reactome` | Database data/derived files CC0, art CC BY4; wording correction already covered by 2217. | [Provider reference](https://reactome.org/license) |
| `seer-explorer` | Government service baseline retained; source-specific/third-party content rights not waived by public access. | [Provider reference](https://seer.cancer.gov/about/using-website.html) |
| `semantic-scholar` | UNRESOLVED: canonical API terms403; official dataset/API agreement conflict; verify service applicability. | [Provider reference](https://www.semanticscholar.org/product/api/license) |
| `string` | Confirmed API/data CC BY4; incorporated third-party material retains terms. | [Provider reference](https://string-db.org/cgi/access?footer_active_subpage=licensing) |
| `umls` | Confirmed individual license and terminology-specific further restrictions/fees; not blanket unrestricted. | [Provider reference](https://www.nlm.nih.gov/databases/umls.html) |
| `uniprot` | Prior registry reading retained; current page missing/shell/generic or not independently sufficient for data-rights verification. | [Provider reference](https://www.uniprot.org/help/license) |
| `vaers` | Confirmed data-use restrictions; statistical use/no re-identification. | [Provider reference](https://wonder.cdc.gov/datause.html) |
| `variantvalidator` | UNVERIFIED: service description contains no data/output rights grant. | [Provider reference](https://variantvalidator.org/) |
| `who-ivd` | UNRESOLVED: generic WHO commercial permission clause; specific dataset applicability unverified. | [Provider reference](https://www.who.int/about/policies/publishing/copyright) |
| `who-prequalification` | UNRESOLVED: generic WHO commercial permission clause; specific dataset applicability unverified. | [Provider reference](https://www.who.int/about/policies/publishing/copyright) |
| `wikipathways` | Prior registry reading retained; current page missing/shell/generic or not independently sufficient for data-rights verification. | [Provider reference](https://www.wikipathways.org/terms.html) |

Semantic Scholar's current API agreement applicability remains unresolved: the canonical recorded page refused retrieval and other official agreements differ in scope. WHO's generic commercial-permission rule does not settle these specific datasets. PharmacoDB's paper/code licenses do not establish database rights. UMLS requires an individual agreement plus terminology-specific permissions. Identifier-only downstream references do not establish acquisition of a provider dataset. These are evidence limits for separate ownership decisions, not new license classifications.

The review stops here. Missing fixtures, untested interleavings, full-feature AlphaGenome execution and unresolved terms remain explicitly unverified. No provider recordings or live data requests were added.

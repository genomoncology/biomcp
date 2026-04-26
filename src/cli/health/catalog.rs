//! Static source descriptors and catalog helpers for `biomcp health`.

#[derive(Debug, Clone, Copy)]
pub(in crate::cli::health) struct SourceDescriptor {
    pub(in crate::cli::health) api: &'static str,
    pub(in crate::cli::health) affects: Option<&'static str>,
    pub(in crate::cli::health) probe: ProbeKind,
}

#[derive(Debug, Clone, Copy)]
pub(in crate::cli::health) enum ProbeKind {
    Get {
        url: &'static str,
    },
    PostJson {
        url: &'static str,
        payload: &'static str,
    },
    AuthGet {
        url: &'static str,
        env_var: &'static str,
        header_name: &'static str,
        header_value_prefix: &'static str,
    },
    OptionalAuthGet {
        url: &'static str,
        env_var: &'static str,
        header_name: &'static str,
        header_value_prefix: &'static str,
        unauthenticated_ok_status: &'static str,
        authenticated_ok_status: &'static str,
        unauthenticated_rate_limited_status: Option<&'static str>,
    },
    AuthQueryParam {
        url: &'static str,
        env_var: &'static str,
        param_name: &'static str,
    },
    #[allow(dead_code)]
    AuthPostJson {
        url: &'static str,
        payload: &'static str,
        env_var: &'static str,
        header_name: &'static str,
        header_value_prefix: &'static str,
    },
    AlphaGenomeConnect {
        env_var: &'static str,
    },
    VaersQuery,
}

pub(in crate::cli::health) const HEALTH_SOURCES: &[SourceDescriptor] = &[
    SourceDescriptor {
        api: "MyGene",
        affects: Some("get/search gene and gene helper commands"),
        probe: ProbeKind::Get {
            url: "https://mygene.info/v3/query?q=BRAF&size=1",
        },
    },
    SourceDescriptor {
        api: "MyVariant",
        affects: Some("get/search variant and variant helper commands"),
        probe: ProbeKind::Get {
            url: "https://myvariant.info/v1/query?q=rs113488022&size=1",
        },
    },
    SourceDescriptor {
        api: "MyChem",
        affects: Some("get/search drug and drug helper commands"),
        probe: ProbeKind::Get {
            url: "https://mychem.info/v1/query?q=aspirin&size=1",
        },
    },
    SourceDescriptor {
        api: "PubTator3",
        affects: Some("article annotations and entity extraction"),
        probe: ProbeKind::Get {
            url: "https://www.ncbi.nlm.nih.gov/research/pubtator3-api/publications/export/biocjson?pmids=22663011",
        },
    },
    SourceDescriptor {
        api: "PubMed",
        affects: Some("PubMed-backed article search foundation"),
        probe: ProbeKind::Get {
            url: "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi?db=pubmed&retmode=json&retmax=1&term=BRAF",
        },
    },
    SourceDescriptor {
        api: "Europe PMC",
        affects: Some("article search coverage"),
        probe: ProbeKind::Get {
            url: "https://www.ebi.ac.uk/europepmc/webservices/rest/search?query=BRAF&format=json&pageSize=1",
        },
    },
    SourceDescriptor {
        api: "NCBI E-utilities",
        affects: Some("article fulltext fallback resolution"),
        probe: ProbeKind::Get {
            url: "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=pmc&id=9984800&rettype=xml",
        },
    },
    SourceDescriptor {
        api: "LitSense2",
        affects: Some("keyword-gated semantic article search"),
        probe: ProbeKind::Get {
            url: "https://www.ncbi.nlm.nih.gov/research/litsense2-api/api/sentences/?query=test&rerank=true",
        },
    },
    SourceDescriptor {
        api: "PMC OA",
        affects: Some("article fulltext resolution"),
        probe: ProbeKind::Get {
            url: "https://www.ncbi.nlm.nih.gov/pmc/utils/oa/oa.fcgi?id=PMC9984800",
        },
    },
    SourceDescriptor {
        api: "NCBI ID Converter",
        affects: Some("article fulltext resolution and identifier bridging"),
        probe: ProbeKind::Get {
            url: "https://pmc.ncbi.nlm.nih.gov/tools/idconv/api/v1/articles/?format=json&idtype=pmid&ids=22663011",
        },
    },
    SourceDescriptor {
        api: "ClinicalTrials.gov",
        affects: Some("search/get trial and trial helper commands"),
        probe: ProbeKind::Get {
            url: "https://clinicaltrials.gov/api/v2/studies?query.term=cancer&pageSize=1",
        },
    },
    SourceDescriptor {
        api: "NCI CTS",
        affects: Some("trial --source nci"),
        probe: ProbeKind::AuthGet {
            url: "https://clinicaltrialsapi.cancer.gov/api/v2/trials?size=1&keyword=melanoma",
            env_var: "NCI_API_KEY",
            header_name: "X-API-KEY",
            header_value_prefix: "",
        },
    },
    SourceDescriptor {
        api: "Enrichr",
        affects: Some("gene/pathway enrichment sections"),
        probe: ProbeKind::Get {
            url: "https://maayanlab.cloud/Enrichr/datasetStatistics",
        },
    },
    SourceDescriptor {
        api: "OpenFDA",
        affects: Some("adverse-event search"),
        probe: ProbeKind::Get {
            url: "https://api.fda.gov/drug/event.json?limit=1",
        },
    },
    SourceDescriptor {
        api: "CDC WONDER VAERS",
        affects: Some("vaccine adverse-event search for --source vaers|all"),
        probe: ProbeKind::VaersQuery,
    },
    SourceDescriptor {
        api: "OncoKB",
        affects: Some("variant oncokb command and variant evidence section"),
        probe: ProbeKind::AuthGet {
            url: "https://www.oncokb.org/api/v1/annotate/mutations/byProteinChange?hugoSymbol=BRAF&alteration=V600E",
            env_var: "ONCOKB_TOKEN",
            header_name: "Authorization",
            header_value_prefix: "Bearer ",
        },
    },
    SourceDescriptor {
        api: "DisGeNET",
        affects: Some("gene and disease disgenet sections"),
        probe: ProbeKind::AuthGet {
            url: "https://api.disgenet.com/api/v1/gda/summary?gene_ncbi_id=7157&page_number=0",
            env_var: "DISGENET_API_KEY",
            header_name: "Authorization",
            header_value_prefix: "",
        },
    },
    SourceDescriptor {
        api: "AlphaGenome",
        affects: Some("variant predict section"),
        probe: ProbeKind::AlphaGenomeConnect {
            env_var: "ALPHAGENOME_API_KEY",
        },
    },
    SourceDescriptor {
        api: "Semantic Scholar",
        affects: Some("Semantic Scholar features"),
        probe: ProbeKind::OptionalAuthGet {
            url: "https://api.semanticscholar.org/graph/v1/paper/search?query=BRAF&fields=paperId,title&limit=1",
            env_var: "S2_API_KEY",
            header_name: "x-api-key",
            header_value_prefix: "",
            unauthenticated_ok_status: "available (unauthenticated, shared rate limit)",
            authenticated_ok_status: "configured (authenticated)",
            unauthenticated_rate_limited_status: Some(
                "unavailable (set S2_API_KEY for reliable access)",
            ),
        },
    },
    SourceDescriptor {
        api: "CPIC",
        affects: Some("pgx recommendations and annotations"),
        probe: ProbeKind::Get {
            url: "https://api.cpicpgx.org/v1/pair_view?select=pairid&limit=1",
        },
    },
    SourceDescriptor {
        api: "PharmGKB",
        affects: Some("pgx recommendations and annotations"),
        probe: ProbeKind::Get {
            url: "https://api.pharmgkb.org/v1/data/labelAnnotation?relatedChemicals.name=warfarin&view=min",
        },
    },
    SourceDescriptor {
        api: "Monarch",
        affects: Some("disease genes, phenotypes, and models"),
        probe: ProbeKind::Get {
            url: "https://api-v3.monarchinitiative.org/v3/api/association?object=MONDO:0007739&subject_category=biolink:Gene&limit=1",
        },
    },
    SourceDescriptor {
        api: "HPO",
        affects: Some("phenotype search and disease ranking"),
        probe: ProbeKind::Get {
            url: "https://ontology.jax.org/api/hp/terms/HP:0001250",
        },
    },
    SourceDescriptor {
        api: "MyDisease",
        affects: Some("disease search and normalization"),
        probe: ProbeKind::Get {
            url: "https://mydisease.info/v1/query?q=melanoma&size=1&fields=disease_ontology.name,mondo.label",
        },
    },
    SourceDescriptor {
        api: "SEER Explorer",
        affects: Some("disease survival section"),
        probe: ProbeKind::Get {
            url: "https://seer.cancer.gov/statistics-network/explorer/source/content_writers/get_var_formats.php",
        },
    },
    SourceDescriptor {
        api: "NIH Reporter",
        affects: Some("gene and disease funding sections"),
        probe: ProbeKind::PostJson {
            url: "https://api.reporter.nih.gov/v2/projects/search",
            payload: r#"{"criteria":{"advanced_text_search":{"operator":"and","search_field":"projecttitle,abstracttext","search_text":"\"ERBB2\""},"fiscal_years":[2022,2023,2024,2025,2026]},"include_fields":["ProjectNum"],"offset":0,"limit":1,"sort_field":"award_amount","sort_order":"desc"}"#,
        },
    },
    SourceDescriptor {
        api: "CIViC",
        affects: Some("disease genes and variants sections"),
        probe: ProbeKind::PostJson {
            url: "https://civicdb.org/api/graphql",
            payload: r#"{"query":"query { evidenceItems(first: 1) { totalCount } }"}"#,
        },
    },
    SourceDescriptor {
        api: "GWAS Catalog",
        affects: Some("gwas search and variant gwas context"),
        probe: ProbeKind::Get {
            url: "https://www.ebi.ac.uk/gwas/rest/api/singleNucleotidePolymorphisms/rs7903146",
        },
    },
    SourceDescriptor {
        api: "GTEx",
        affects: Some("gene expression section"),
        probe: ProbeKind::Get {
            url: "https://gtexportal.org/api/v2/",
        },
    },
    SourceDescriptor {
        api: "DGIdb",
        affects: Some("gene druggability section"),
        probe: ProbeKind::PostJson {
            url: "https://dgidb.org/api/graphql",
            payload: r#"{"query":"query { __typename }"}"#,
        },
    },
    SourceDescriptor {
        api: "ClinGen",
        affects: Some("gene clingen section"),
        probe: ProbeKind::Get {
            url: "https://search.clinicalgenome.org/api/genes/look/BRAF",
        },
    },
    SourceDescriptor {
        api: "gnomAD",
        affects: Some("gene constraint section"),
        probe: ProbeKind::PostJson {
            url: "https://gnomad.broadinstitute.org/api",
            payload: r#"{"query":"query { __typename }"}"#,
        },
    },
    SourceDescriptor {
        api: "UniProt",
        affects: Some("gene protein summary and protein detail sections"),
        probe: ProbeKind::Get {
            url: "https://rest.uniprot.org/uniprotkb/P15056.json",
        },
    },
    SourceDescriptor {
        api: "QuickGO",
        affects: Some("gene go terms and protein annotation sections"),
        probe: ProbeKind::Get {
            url: "https://www.ebi.ac.uk/QuickGO/services/annotation/search?geneProductId=P15056&limit=5",
        },
    },
    SourceDescriptor {
        api: "STRING",
        affects: Some("gene interactions and protein interaction sections"),
        probe: ProbeKind::Get {
            url: "https://string-db.org/api/json/network?identifiers=BRAF&species=9606&limit=5",
        },
    },
    SourceDescriptor {
        api: "Reactome",
        affects: Some("pathway search and disease pathway sections"),
        probe: ProbeKind::Get {
            url: "https://reactome.org/ContentService/search/query?query=MAPK&species=Homo%20sapiens&pageSize=1",
        },
    },
    SourceDescriptor {
        api: "KEGG",
        affects: Some("pathway search and detail sections"),
        probe: ProbeKind::Get {
            url: "https://rest.kegg.jp/find/pathway/MAPK",
        },
    },
    SourceDescriptor {
        api: "WikiPathways",
        affects: Some("pathway search and WikiPathways detail/genes sections"),
        probe: ProbeKind::Get {
            url: "https://www.wikipathways.org/json/findPathwaysByText.json",
        },
    },
    SourceDescriptor {
        api: "g:Profiler",
        affects: Some("gene enrichment (biomcp enrich)"),
        probe: ProbeKind::PostJson {
            url: "https://biit.cs.ut.ee/gprofiler/api/gost/profile/",
            payload: r#"{"organism":"hsapiens","query":["BRAF"]}"#,
        },
    },
    SourceDescriptor {
        api: "OpenTargets",
        affects: Some("gene druggability, drug target, and disease association sections"),
        probe: ProbeKind::PostJson {
            url: "https://api.platform.opentargets.org/api/v4/graphql",
            payload: r#"{"query":"query { drug(chemblId: \"CHEMBL25\") { id name } }"}"#,
        },
    },
    SourceDescriptor {
        api: "ChEMBL",
        affects: Some("drug targets and indications sections"),
        probe: ProbeKind::Get {
            url: "https://www.ebi.ac.uk/chembl/api/data/molecule/CHEMBL25.json",
        },
    },
    SourceDescriptor {
        api: "HPA",
        affects: Some("gene protein tissue expression and localization section"),
        probe: ProbeKind::Get {
            url: "https://www.proteinatlas.org/ENSG00000157764.xml",
        },
    },
    SourceDescriptor {
        api: "InterPro",
        affects: Some("protein domains section"),
        probe: ProbeKind::Get {
            url: "https://www.ebi.ac.uk/interpro/api/entry/interpro/protein/uniprot/P15056/?page_size=5",
        },
    },
    SourceDescriptor {
        api: "ComplexPortal",
        affects: Some("protein complex membership section"),
        probe: ProbeKind::Get {
            url: "https://www.ebi.ac.uk/intact/complex-ws/search/P15056?number=25&filters=species_f:(%22Homo%20sapiens%22)",
        },
    },
    SourceDescriptor {
        api: "OLS4",
        affects: Some("discover command concept resolution"),
        probe: ProbeKind::Get {
            url: "https://www.ebi.ac.uk/ols4/api/search?q=BRCA1&rows=1&groupField=iri&ontology=hgnc",
        },
    },
    SourceDescriptor {
        api: "UMLS",
        affects: Some("discover command clinical crosswalk enrichment"),
        probe: ProbeKind::AuthQueryParam {
            url: "https://uts-ws.nlm.nih.gov/rest/search/current?string=BRCA1&pageSize=1",
            env_var: "UMLS_API_KEY",
            param_name: "apiKey",
        },
    },
    SourceDescriptor {
        api: "MedlinePlus",
        affects: Some("discover command plain-language disease and symptom context"),
        probe: ProbeKind::Get {
            url: "https://wsearch.nlm.nih.gov/ws/query?db=healthTopics&term=chest+pain&retmax=1",
        },
    },
    SourceDescriptor {
        api: "cBioPortal",
        affects: Some("cohort frequency section"),
        probe: ProbeKind::Get {
            url: "https://www.cbioportal.org/api/studies?projection=SUMMARY&pageSize=1",
        },
    },
];

pub(in crate::cli::health) const EMA_LOCAL_DATA_AFFECTS: &str = "default plain-name drug search plus search/get drug --region eu|all and EU regulatory/safety/shortage sections";
pub(in crate::cli::health) const CVX_LOCAL_DATA_AFFECTS: &str =
    "EMA vaccine identity bridge for plain-name drug search";
pub(in crate::cli::health) const WHO_LOCAL_DATA_AFFECTS: &str = "default plain-name drug search plus search/get drug --region who|all and WHO regulatory sections";
pub(in crate::cli::health) const DDINTER_LOCAL_DATA_AFFECTS: &str =
    "drug interactions helper plus get drug interactions section";
pub(in crate::cli::health) const GTR_LOCAL_DATA_AFFECTS: &str =
    "search/get diagnostic and local GTR-backed diagnostic routing";
pub(in crate::cli::health) const WHO_IVD_LOCAL_DATA_AFFECTS: &str =
    "search/get diagnostic and local WHO IVD-backed infectious-disease diagnostic routing";

pub(in crate::cli::health) fn health_sources() -> &'static [SourceDescriptor] {
    HEALTH_SOURCES
}

#[cfg_attr(not(test), allow(dead_code))]
pub(in crate::cli::health) fn affects_for_api(api: &str) -> Option<&'static str> {
    health_sources()
        .iter()
        .find(|source| source.api == api)
        .and_then(|source| source.affects)
}

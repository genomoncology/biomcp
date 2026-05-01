from __future__ import annotations

import re
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]

OVERVIEW_FILE = "index.md"
SOURCE_PAGE_SECTION_HEADINGS = (
    "## What BioMCP exposes",
    "## Example commands",
    "## API access",
    "## Official source",
    "## Related docs",
)

SOURCE_PAGE_SPECS = {
    "pubmed.md": {
        "title": "PubMed MCP Tool for AI Agents | BioMCP",
        "description": "Search PubMed in BioMCP with PubTator3 annotations, article summaries, and PMC full-text handoff so AI agents can review literature faster.",
        "api_access": "Optional `NCBI_API_KEY` for higher NCBI throughput.",
        "provider_key_url": "https://www.ncbi.nlm.nih.gov/account/settings/",
        "official_url": "https://pubmed.ncbi.nlm.nih.gov/",
        "required_intro_phrases": [
            '"PubMed" is an umbrella label',
            "PubTator3",
            "Europe PMC",
            "NCBI E-utilities",
            "PMC OA",
            "NCBI ID Converter",
            "Semantic Scholar",
            "question-format unfielded article terms",
        ],
        "exposes": [
            "search article",
            "get article <id>",
            "get article <id> annotations",
            "get article <id> fulltext",
            "article entities <pmid>",
        ],
        "example_commands": [
            "biomcp search article -g BRAF --limit 3",
            "biomcp get article 22663011",
            "biomcp get article 22663011 annotations",
            "biomcp article entities 22663011",
            "biomcp get article 27083046 fulltext",
        ],
    },
    "clinicaltrials-gov.md": {
        "title": "ClinicalTrials.gov MCP Tool for AI Agents | BioMCP",
        "description": "Search ClinicalTrials.gov from BioMCP for recruiting studies, eligibility criteria, locations, and trial details without learning the native API.",
        "api_access": "No BioMCP API key required.",
        "official_url": "https://clinicaltrials.gov/",
        "required_intro_phrases": [
            "default trial backend",
            "`--source nci`",
        ],
        "exposes": [
            "search trial",
            "get trial <nct_id>",
            "get trial <nct_id> eligibility",
            "get trial <nct_id> locations",
            "get trial <nct_id> outcomes",
            "get trial <nct_id> arms",
            "get trial <nct_id> references",
        ],
        "example_commands": [
            'biomcp search trial -c melanoma --status recruiting --limit 3',
            'biomcp search trial -c melanoma --mutation "BRAF V600E" --limit 3',
            "biomcp get trial NCT02576665",
            "biomcp get trial NCT02576665 eligibility",
            "biomcp get trial NCT02576665 --limit 3 locations",
        ],
    },
    "clinvar.md": {
        "title": "ClinVar MCP Tool for Variant Interpretation | BioMCP",
        "description": "Use BioMCP to pull ClinVar clinical significance, review status, and disease context for human variants through one variant lookup workflow.",
        "api_access": "No standalone BioMCP key path; ClinVar content is surfaced indirectly via MyVariant.info.",
        "official_url": "https://www.ncbi.nlm.nih.gov/clinvar/",
        "required_intro_phrases": [
            "indirect-only",
            "MyVariant.info",
            "does not act as a direct ClinVar API client",
        ],
        "exposes": [
            "get variant <id>",
            "get variant <id> clinvar",
            "search variant -g <gene> --significance <value>",
        ],
        "example_commands": [
            "biomcp get variant rs113488022",
            "biomcp get variant rs113488022 clinvar",
            'biomcp get variant "BRAF V600E" clinvar',
            "biomcp search variant -g BRCA1 --significance pathogenic --limit 5",
        ],
    },
    "openfda.md": {
        "title": "OpenFDA MCP Tool for Drug Safety Workflows | BioMCP",
        "description": "Use BioMCP to query OpenFDA adverse events, recalls, device reports, labels, and approval context for drug safety and surveillance workflows.",
        "api_access": "Optional `OPENFDA_API_KEY` for higher quota headroom.",
        "provider_key_url": "https://open.fda.gov/apis/authentication/",
        "official_url": "https://open.fda.gov/",
        "required_intro_phrases": [
            "FAERS",
            "MAUDE",
            "shortages",
            "Drugs@FDA-derived",
        ],
        "exposes": [
            "search adverse-event --drug <name>",
            "search adverse-event --type recall --drug <name>",
            "search adverse-event --type device --device <name>",
            "get adverse-event <report_id>",
            "get drug <name> label",
            "get drug <name> shortage",
            "get drug <name> approvals",
            "get drug <name> interactions",
            "get drug <name> safety --region us",
        ],
        "example_commands": [
            "biomcp search adverse-event --drug pembrolizumab --limit 3",
            "biomcp search adverse-event --type recall --drug metformin --limit 3",
            'biomcp search adverse-event --type device --device "insulin pump" --limit 3',
            "biomcp get drug vemurafenib label",
            "biomcp get drug dabrafenib approvals",
        ],
    },
    "ddinter.md": {
        "title": "DDInter MCP Tool for Drug-Drug Interactions | BioMCP",
        "description": "Use BioMCP to query DDInter-backed drug-drug interactions, severity levels, and class summaries through the local DDInter CSV bundle.",
        "api_access": "No BioMCP API key required.",
        "official_url": "https://ddinter.scbdd.com/download/",
        "required_intro_phrases": [
            "structured answer to a drug-drug interaction",
            "local-runtime source",
            "`BIOMCP_DDINTER_DIR`",
            "`biomcp ddinter sync`",
            "severity levels",
        ],
        "exposes": [
            "drug interactions <name>",
            "get drug <name> interactions",
            "health",
        ],
        "example_commands": [
            "biomcp drug interactions warfarin",
            "biomcp drug interactions imatinib",
            "biomcp get drug warfarin interactions",
            "biomcp ddinter sync",
        ],
    },
    "vaers.md": {
        "title": "CDC WONDER VAERS MCP Tool for Vaccine Safety Signals | BioMCP",
        "description": "Use BioMCP to query CDC WONDER VAERS aggregate vaccine adverse-event summaries in BioMCP with reaction counts, seriousness breakdowns, and age distribution.",
        "api_access": "No BioMCP API key required. BioMCP calls the CDC WONDER D8 XML POST endpoint and includes the required agreement to the CDC WONDER data use restrictions.",
        "official_url": "https://wonder.cdc.gov/vaers.html",
        "required_intro_phrases": [
            "aggregate-only",
            "`search adverse-event --source vaers`",
            "`--source all`",
            "CDC WONDER D8",
            "CDC CVX/MVX bridge",
        ],
        "exposes": [
            "search adverse-event <vaccine_query> --source vaers",
            "search adverse-event <vaccine_query> --source all",
            "search adverse-event <vaccine_query>",
            "biomcp health",
        ],
        "example_commands": [
            'biomcp search adverse-event "MMR vaccine" --source vaers --limit 5',
            'biomcp search adverse-event "COVID-19 vaccine" --source all --limit 5',
            'biomcp search adverse-event "influenza vaccine" --source all --limit 5',
            'biomcp search adverse-event "COVID-19 vaccine" --source faers --limit 5',
            "biomcp health --apis-only",
        ],
    },
    "uniprot.md": {
        "title": "UniProt MCP Tool for Protein Data | BioMCP",
        "description": "Use BioMCP to search UniProt proteins, fetch canonical protein cards, and surface structure-linked context for AI agents and research workflows.",
        "api_access": "No BioMCP API key required.",
        "official_url": "https://www.uniprot.org/",
        "required_intro_phrases": [
            "gene `protein` section",
            "Domains, interactions, and complexes are separate provider sections",
            "PDB",
            "AlphaFold",
        ],
        "exposes": [
            "search protein",
            "get protein <accession_or_symbol>",
            "get gene <symbol> protein",
            "get protein <accession> structures",
        ],
        "example_commands": [
            "biomcp search protein BRAF --limit 3",
            "biomcp get protein P15056",
            "biomcp get gene BRAF protein",
            "biomcp get protein P15056 structures",
        ],
    },
    "gnomad.md": {
        "title": "gnomAD MCP Tool for Variant Frequency Analysis | BioMCP",
        "description": "Use BioMCP to pull gnomAD population frequencies and gene constraint metrics for variant interpretation, rarity checks, and gene-level context.",
        "api_access": "No BioMCP API key required.",
        "official_url": "https://gnomad.broadinstitute.org/",
        "required_intro_phrases": [
            "gene constraint comes from the gnomAD source path directly",
            "MyVariant.info payloads",
        ],
        "exposes": [
            "get gene <symbol> constraint",
            "get variant <id> population",
            "search variant -g <gene> --max-frequency <value>",
        ],
        "example_commands": [
            "biomcp get gene BRAF constraint",
            "biomcp get variant rs113488022 population",
            'biomcp get variant "chr7:g.140453136A>T" population',
            "biomcp search variant -g BRCA1 --max-frequency 0.01 --limit 5",
        ],
    },
    "reactome.md": {
        "title": "Reactome MCP Tool for Pathway Analysis | BioMCP",
        "description": "Use BioMCP to search Reactome pathways, inspect pathway genes and events, and connect pathway context to downstream trial and article workflows.",
        "api_access": "No BioMCP API key required.",
        "official_url": "https://reactome.org/",
        "required_intro_phrases": [
            "multi-source across Reactome, KEGG, and WikiPathways",
            "Top-level `biomcp enrich` is a g:Profiler workflow",
            "Reactome-gated enrichment",
        ],
        "exposes": [
            "search pathway",
            "get pathway <id>",
            "get pathway <id> genes",
            "get pathway <id> events",
            "get pathway <id> enrichment",
            "get gene <symbol> pathways",
        ],
        "example_commands": [
            'biomcp search pathway "MAPK signaling" --limit 5',
            "biomcp get pathway R-HSA-5673001",
            "biomcp get pathway R-HSA-5673001 genes",
            "biomcp get pathway R-HSA-5673001 events",
            "biomcp get gene BRAF pathways",
        ],
    },
    "semantic-scholar.md": {
        "title": "Semantic Scholar MCP Tool for Citation Graphs | BioMCP",
        "description": "Use BioMCP to add Semantic Scholar TLDRs, citations, references, and recommendations to literature-review workflows for AI agents.",
        "api_access": "Optional `S2_API_KEY` for dedicated quota and higher reliability.",
        "provider_key_url": "https://www.semanticscholar.org/product/api",
        "official_url": "https://www.semanticscholar.org/",
        "required_intro_phrases": [
            "`search article` does not expose `--source semantic-scholar`",
            "automatic optional search leg",
            "shared-pool mode at 1 req/2sec",
            "authenticated mode at 1 req/sec",
            "article citations",
            "article references",
            "article recommendations",
        ],
        "exposes": [
            "search article",
            "get article <id> tldr",
            "article citations <id>",
            "article references <id>",
            "article recommendations <id>",
        ],
        "example_commands": [
            "biomcp get article 22663011 tldr",
            "biomcp article citations 22663011 --limit 3",
            "biomcp article references 22663011 --limit 3",
            "biomcp article recommendations 22663011 --limit 3",
        ],
    },
    "chembl.md": {
        "title": "ChEMBL MCP Tool for Drug Target Enrichment | BioMCP",
        "description": "Use BioMCP to pull ChEMBL drug-target activity and indication context for drug lookups without working directly with the ChEMBL API.",
        "api_access": "No BioMCP API key required.",
        "official_url": "https://www.ebi.ac.uk/chembl/",
        "required_intro_phrases": [
            "drug `targets` section",
            "drug `indications` section",
            "OpenTargets",
            "activity and mechanism context",
        ],
        "exposes": [
            "get drug <name> targets",
            "get drug <name> indications",
        ],
        "example_commands": [
            "biomcp get drug pembrolizumab targets",
            "biomcp get drug pembrolizumab indications",
            "biomcp get drug dabrafenib targets",
        ],
    },
    "opentargets.md": {
        "title": "OpenTargets MCP Tool for Target and Disease Context | BioMCP",
        "description": "Use BioMCP to surface OpenTargets disease associations, drug-target context, and druggability signals in BioMCP gene, disease, and drug workflows.",
        "api_access": "No BioMCP API key required.",
        "official_url": "https://platform.opentargets.org/",
        "required_intro_phrases": [
            "gene `druggability` section",
            "`get gene <symbol> diseases`",
            "`get disease <id> genes`",
            "OpenTargets scores",
        ],
        "exposes": [
            "get gene <symbol> diseases",
            "get gene <symbol> druggability",
            "get drug <name> targets",
            "get drug <name> indications",
            "get disease <id> genes",
            "get disease <id> prevalence",
        ],
        "example_commands": [
            "biomcp get gene BRAF diseases",
            "biomcp get gene BRAF druggability",
            "biomcp get disease MONDO:0005105 genes",
            "biomcp get disease MONDO:0005105 prevalence",
            "biomcp get drug pembrolizumab targets",
        ],
    },
    "seer.md": {
        "title": "SEER Explorer MCP Tool for Cancer Survival | BioMCP",
        "description": "Use BioMCP to surface SEER Explorer 5-year relative survival summaries for mapped cancers in the BioMCP disease survival section.",
        "api_access": "No BioMCP API key required.",
        "official_url": "https://seer.cancer.gov/statistics-network/explorer/",
        "required_intro_phrases": [
            "disease `survival` section",
            "5-year relative survival",
            "all-ages and all-races",
            "sex-split",
            "undocumented SEER Explorer UI endpoints",
        ],
        "exposes": [
            "get disease <id> survival",
            "get disease <id> all",
        ],
        "example_commands": [
            'biomcp get disease "chronic myeloid leukemia" survival',
            'biomcp get disease "Hodgkin lymphoma" survival',
            'biomcp get disease "Marfan syndrome" survival',
            'biomcp get disease "chronic myeloid leukemia" all',
        ],
    },
    "civic.md": {
        "title": "CIViC MCP Tool for Clinical Variant Evidence | BioMCP",
        "description": "Use BioMCP to surface CIViC evidence, disease-associated variants, and therapy context across BioMCP variant, gene, drug, and disease workflows.",
        "api_access": "No BioMCP API key required.",
        "official_url": "https://civicdb.org/home",
        "required_intro_phrases": [
            "section-gated enrichment",
            "`get variant <id> civic`",
            "`get gene <symbol> civic`",
            "`get disease <id> variants`",
            "`get drug <name> civic`",
        ],
        "exposes": [
            "get variant <id> civic",
            "get gene <symbol> civic",
            "get drug <name> civic",
            "get disease <id> civic",
            "get disease <id> variants",
        ],
        "example_commands": [
            'biomcp get variant "BRAF V600E" civic',
            "biomcp get gene BRAF civic",
            "biomcp get drug vemurafenib civic",
            "biomcp get disease MONDO:0005105 civic",
            "biomcp get disease MONDO:0005105 variants",
        ],
    },
    "oncokb.md": {
        "title": "OncoKB MCP Tool for Oncology Variant Annotation | BioMCP",
        "description": "Use BioMCP to run the explicit OncoKB variant helper for oncogenicity, evidence levels, and treatment implications on actionable variants.",
        "api_access": "Requires `ONCOKB_TOKEN` for the production OncoKB API. Configure it with the [API Keys](../getting-started/api-keys.md) guide and register at [OncoKB](https://www.oncokb.org/account/register).",
        "provider_key_url": "https://www.oncokb.org/account/register",
        "official_url": "https://www.oncokb.org/",
        "required_intro_phrases": [
            "explicit helper",
            "`variant oncokb`",
            "`ONCOKB_TOKEN`",
            "registration-gated",
        ],
        "exposes": [
            "variant oncokb <id>",
        ],
        "example_commands": [
            'biomcp variant oncokb "BRAF V600E"',
            'biomcp variant oncokb "EGFR L858R"',
            'biomcp variant oncokb "KRAS G12C"',
        ],
    },
    "cbioportal.md": {
        "title": "cBioPortal MCP Tool for Cohort Variant Context | BioMCP",
        "description": "Use BioMCP to add cBioPortal cohort-frequency context to variants and download local study datasets for BioMCP study analytics.",
        "api_access": "No BioMCP API key required. Local study analytics use downloaded datasets in the default study root or `BIOMCP_STUDY_DIR`.",
        "official_url": "https://www.cbioportal.org/",
        "required_intro_phrases": [
            "variant `cbioportal` section",
            "`study` is BioMCP's local cBioPortal analytics family",
            "`study download`",
            "`study query`",
        ],
        "exposes": [
            "get variant <id> cbioportal",
            "study download --list",
            "study download <study_id>",
            "study query --study <id> --gene <symbol> --type <mutations|cna|expression>",
        ],
        "example_commands": [
            'biomcp get variant "BRAF V600E" cbioportal',
            "biomcp study download --list",
            "biomcp study download msk_impact_2017",
            "biomcp study query --study msk_impact_2017 --gene TP53 --type mutations",
        ],
    },
    "ema.md": {
        "title": "EMA MCP Tool for EU Drug Regulatory Data | BioMCP",
        "description": "Use BioMCP to search EMA-backed EU drug records in BioMCP and retrieve regulatory, safety, and shortage context through the local EMA batch.",
        "api_access": "No BioMCP API key required. BioMCP auto-downloads the EMA human-medicines JSON batch into `BIOMCP_EMA_DIR` or the default data directory on first use.",
        "official_url": "https://www.ema.europa.eu/en/about-us/about-website/download-website-data-json-data-format",
        "required_intro_phrases": [
            "`BIOMCP_EMA_DIR`",
            "auto-downloads the six EMA human-medicines JSON feeds",
            "`biomcp ema sync`",
            "`--region eu|all`",
            "name/alias lookups",
        ],
        "exposes": [
            "search drug <name> --region eu",
            "search drug <name> --region all",
            "get drug <name> regulatory --region eu|all",
            "get drug <name> safety --region eu|all",
            "get drug <name> shortage --region eu|all",
        ],
        "example_commands": [
            "biomcp search drug Keytruda --region eu --limit 3",
            "biomcp get drug Keytruda regulatory --region eu",
            "biomcp get drug Dupixent regulatory --region ema",
            "biomcp get drug Ozempic safety --region eu",
            "biomcp ema sync",
        ],
    },
    "who-prequalification.md": {
        "title": "WHO Prequalification MCP Tool for Global Drug Access | BioMCP",
        "description": "Use BioMCP to search WHO Prequalification-backed drug and vaccine records in BioMCP and retrieve global regulatory context through the local WHO finished-pharma, API, and vaccine exports.",
        "api_access": "No BioMCP API key required. BioMCP auto-downloads the WHO finished-pharmaceutical-products CSV, active-pharmaceutical-ingredients CSV, and vaccine CSV into `BIOMCP_WHO_DIR` or the default data directory on first use.",
        "official_url": "https://extranet.who.int/prequal/medicines/prequalified/finished-pharmaceutical-products/export?page&_format=csv",
        "required_intro_phrases": [
            "`BIOMCP_WHO_DIR`",
            "finished-pharmaceutical-products CSV",
            "active-pharmaceutical-ingredients CSV",
            "vaccine export",
            "`biomcp who sync`",
            "`--region who|all`",
            "`--product-type <finished_pharma|api|vaccine>`",
            "structured U.S. hits through WHO prequalification",
        ],
        "exposes": [
            "search drug <name> --region who",
            "search drug <name> --region all",
            "search drug --indication <disease> --region who",
            "search drug <name> --region who --product-type <finished_pharma|api|vaccine>",
            "get drug <name> regulatory --region who|all",
        ],
        "example_commands": [
            "biomcp search drug trastuzumab --region who --limit 5",
            "biomcp search drug artesunate --region who --product-type api --limit 5",
            "biomcp search drug BCG --region who --product-type vaccine --limit 5",
            "biomcp search drug --indication malaria --region who --limit 5",
            "biomcp get drug trastuzumab regulatory --region who",
        ],
    },
    "gtr.md": {
        "title": "NCBI Genetic Testing Registry MCP Tool for Diagnostics | BioMCP",
        "description": "Use BioMCP to search GTR-backed genetic tests, fetch source-native diagnostic cards, and manage the local NCBI Genetic Testing Registry bundle.",
        "api_access": "No BioMCP API key required.",
        "official_url": "https://www.ncbi.nlm.nih.gov/gtr/",
        "required_intro_phrases": [
            "gene-centric genetic tests",
            "local-runtime backbone for the multi-source `diagnostic` entity",
            "`--source all` route can merge GTR rows with WHO IVD rows",
            "`BIOMCP_GTR_DIR`",
            "`biomcp gtr sync`",
        ],
        "exposes": [
            "search diagnostic --gene <symbol> --source gtr",
            "search diagnostic --disease <name> --source gtr",
            "search diagnostic --type <method> --source gtr",
            "search diagnostic --manufacturer <name> --source gtr",
            "get diagnostic GTR000006692.3",
            "get diagnostic GTR000006692.3 genes",
            "get diagnostic GTR000006692.3 conditions",
            "get diagnostic GTR000006692.3 methods",
            "biomcp health",
            "biomcp gtr sync",
        ],
        "example_commands": [
            "biomcp search diagnostic --gene BRCA1 --source gtr --limit 5",
            "biomcp search diagnostic --disease \"hereditary breast cancer\" --source gtr --limit 5",
            "biomcp get diagnostic GTR000006692.3 genes conditions methods",
            "biomcp gtr sync",
            "biomcp health",
        ],
    },
    "who-ivd.md": {
        "title": "WHO Prequalified IVD MCP Tool for Infectious Disease Diagnostics | BioMCP",
        "description": "Use BioMCP to search WHO prequalified infectious-disease diagnostics, fetch source-native WHO IVD product cards, and manage the local WHO IVD CSV lifecycle without learning the raw export format.",
        "api_access": "No BioMCP API key required. BioMCP downloads the WHO IVD CSV export on first",
        "official_url": "https://extranet.who.int/prequal/vitro-diagnostics/prequalified/in-vitro-diagnostics",
        "required_intro_phrases": [
            "gene-centric genetic tests",
            "WHO-backed branch of the multi-source `diagnostic` entity",
            "`--source all` route merges WHO IVD with GTR",
            "WHO IVD product codes can contain spaces",
            "quotes follow-up `get diagnostic` commands automatically",
        ],
        "exposes": [
            "search diagnostic --disease <name> --source who-ivd",
            "search diagnostic --type <assay_format> --source who-ivd",
            "search diagnostic --manufacturer <name> --source who-ivd",
            "get diagnostic \"<product_code>\"",
            "get diagnostic \"<product_code>\" conditions",
            "biomcp health",
            "biomcp who-ivd sync",
        ],
        "example_commands": [
            "biomcp search diagnostic --disease HIV --source who-ivd --limit 5",
            "biomcp get diagnostic \"<product_code>\"",
            "biomcp get diagnostic \"<product_code>\" conditions",
            "biomcp who-ivd sync",
            "biomcp health",
        ],
    },
    "cdc-cvx.md": {
        "title": "CDC CVX/MVX MCP Tool for Vaccine Identity Bridge | BioMCP",
        "description": "Use BioMCP to bridge vaccine brand names into EMA-backed drug matches and explicit WHO vaccine searches with the local CDC CVX, trade-name, and MVX bundle.",
        "api_access": "No BioMCP API key required. BioMCP auto-downloads `cvx.txt`, `TRADENAME.txt`, and `mvx.txt` into `BIOMCP_CVX_DIR` or the default data directory on first use.",
        "official_url": "https://www2.cdc.gov/vaccines/iis/iisstandards/downloads/cvx.txt",
        "required_intro_phrases": [
            "`BIOMCP_CVX_DIR`",
            "`cvx.txt`",
            "`TRADENAME.txt`",
            "`mvx.txt`",
            "`biomcp cvx sync`",
            "MyChem identity resolution fails",
            "`--region who --product-type vaccine`",
            "`--region us`",
        ],
        "exposes": [
            "search drug <vaccine_brand> --region eu",
            "search drug <vaccine_brand> --region all",
            "search drug <vaccine_brand> --region who --product-type vaccine",
            "search drug <vaccine_brand>",
            "biomcp health",
            "biomcp cvx sync",
        ],
        "example_commands": [
            "biomcp search drug gardasil --region eu --limit 5",
            "biomcp search drug prevnar --region eu --limit 5",
            "biomcp search drug fluzone --region eu --limit 5",
            "biomcp search drug gardasil --region who --product-type vaccine --limit 5",
            "biomcp health",
        ],
    },
    "medlineplus.md": {
        "title": "MedlinePlus MCP Tool for Plain-Language Disease Context | BioMCP",
        "description": "Use BioMCP to add MedlinePlus plain-language context to discover results and opt-in disease clinical-feature summaries.",
        "api_access": "No BioMCP API key required.",
        "official_url": "https://medlineplus.gov/",
        "required_intro_phrases": [
            "plain-language disease or symptom context",
            "supplements `biomcp discover`",
            "suppressed for gene, drug, pathway",
            "`get disease <name_or_id> clinical_features`",
            "embedded reviewed fixtures",
        ],
        "exposes": [
            "biomcp discover <query>",
            "get disease <name_or_id> clinical_features",
        ],
        "example_commands": [
            "biomcp discover \"symptoms of Marfan syndrome\"",
            "biomcp get disease \"uterine leiomyoma\" clinical_features",
            "biomcp get disease MONDO:0007947 clinical_features",
        ],
    },
    "kegg.md": {
        "title": "KEGG MCP Tool for Pathway Search | BioMCP",
        "description": "Use BioMCP to search KEGG pathways, fetch KEGG pathway summaries, and expand to pathway genes without learning KEGG's flat-file API.",
        "api_access": "No BioMCP API key required.",
        "official_url": "https://www.kegg.jp/",
        "required_intro_phrases": [
            "multi-source across Reactome, KEGG, and WikiPathways",
            "KEGG base cards stay summary-only unless you explicitly request `genes`",
            "`events` and pathway `enrichment` are Reactome-only",
            "3 requests / second",
        ],
        "exposes": [
            "search pathway",
            "get pathway <id>",
            "get pathway <id> genes",
            "get gene <symbol> pathways",
        ],
        "example_commands": [
            'biomcp search pathway "MAPK signaling" --limit 5',
            "biomcp get pathway hsa05200",
            "biomcp get pathway hsa05200 genes",
            "biomcp get gene BRAF pathways",
        ],
    },
    "pharmgkb.md": {
        "title": "PharmGKB and CPIC MCP Tool for Pharmacogenomics | BioMCP",
        "description": "Use BioMCP to search CPIC-backed PGx guidance and add PharmGKB clinical annotations to gene-drug pharmacogenomic workflows.",
        "api_access": "No BioMCP API key required.",
        "official_url": "https://www.pharmgkb.org/",
        "required_intro_phrases": [
            "CPIC",
            "PharmGKB",
            "`get pgx <gene_or_drug>`",
            "`annotations`",
            "recommendations",
        ],
        "exposes": [
            "search pgx",
            "get pgx <gene_or_drug>",
            "get pgx <gene_or_drug> recommendations",
            "get pgx <gene_or_drug> frequencies",
            "get pgx <gene_or_drug> guidelines",
            "get pgx <gene_or_drug> annotations",
        ],
        "example_commands": [
            "biomcp search pgx -g CYP2D6 --limit 5",
            "biomcp get pgx CYP2D6",
            "biomcp get pgx CYP2D6 recommendations",
            "biomcp get pgx CYP2D6 annotations",
            "biomcp get pgx codeine recommendations frequencies",
        ],
    },
    "human-protein-atlas.md": {
        "title": "Human Protein Atlas MCP Tool for Tissue Expression | BioMCP",
        "description": "Use BioMCP to surface Human Protein Atlas tissue expression and localization data through the BioMCP gene hpa section.",
        "api_access": "No BioMCP API key required.",
        "official_url": "https://www.proteinatlas.org/",
        "required_intro_phrases": [
            "gene `hpa` section",
            "tissue expression",
            "subcellular localization",
            "cancer expression",
        ],
        "exposes": [
            "get gene <symbol> hpa",
        ],
        "example_commands": [
            "biomcp get gene BRAF hpa",
            "biomcp get gene EGFR hpa",
            "biomcp get gene TP53 hpa",
        ],
    },
    "monarch-initiative.md": {
        "title": "Monarch Initiative MCP Tool for Phenotype and Disease Matching | BioMCP",
        "description": "Use BioMCP to query Monarch-backed disease genes, phenotype matches, and model evidence in BioMCP disease and phenotype workflows.",
        "api_access": "No BioMCP API key required.",
        "official_url": "https://monarchinitiative.org/",
        "required_intro_phrases": [
            "disease `genes` section",
            "disease `models` section",
            "`search phenotype`",
            "There is no `get phenotype` subcommand",
        ],
        "exposes": [
            "get disease <id> genes",
            "get disease <id> phenotypes",
            "get disease <id> models",
            "search phenotype",
        ],
        "example_commands": [
            "biomcp get disease MONDO:0005105 genes",
            "biomcp get disease MONDO:0005105 phenotypes",
            "biomcp get disease MONDO:0005105 models",
            'biomcp search phenotype "HP:0001250 HP:0001263" --limit 10',
        ],
    },
}

EXPECTED_SOURCE_FILES = [OVERVIEW_FILE, *SOURCE_PAGE_SPECS]

EXPECTED_NAV_BLOCK = """  - Sources:
      - Overview: sources/index.md
      - PubMed: sources/pubmed.md
      - ClinicalTrials.gov: sources/clinicaltrials-gov.md
      - ClinVar: sources/clinvar.md
      - OpenFDA: sources/openfda.md
      - UniProt: sources/uniprot.md
      - gnomAD: sources/gnomad.md
      - Reactome: sources/reactome.md
      - Semantic Scholar: sources/semantic-scholar.md
      - ChEMBL: sources/chembl.md
      - OpenTargets: sources/opentargets.md
      - SEER Explorer: sources/seer.md
      - CIViC: sources/civic.md
      - OncoKB: sources/oncokb.md
      - cBioPortal: sources/cbioportal.md
      - DDInter: sources/ddinter.md
      - EMA: sources/ema.md
      - WHO Prequalification: sources/who-prequalification.md
      - NCBI Genetic Testing Registry: sources/gtr.md
      - WHO Prequalified IVD: sources/who-ivd.md
      - CDC CVX/MVX: sources/cdc-cvx.md
      - MedlinePlus: sources/medlineplus.md
      - KEGG: sources/kegg.md
      - PharmGKB / CPIC: sources/pharmgkb.md
      - Human Protein Atlas: sources/human-protein-atlas.md
      - Monarch Initiative: sources/monarch-initiative.md
"""


def _read(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def _read_source_page(filename: str) -> str:
    return _read(f"docs/sources/{filename}")


def _front_matter_value(text: str, field: str) -> str:
    match = re.search(rf"(?ms)\A---\n.*?^{field}: \"([^\"]+)\"\n.*?^---\n", text)
    assert match, f"missing front matter field {field!r}"
    return match.group(1)


def _markdown_section_block(text: str, heading: str, next_heading: str) -> str:
    start = text.index(heading)
    remainder = text[start + len(heading) :]
    if not next_heading:
        return remainder
    end = remainder.find(next_heading)
    if end == -1:
        return remainder
    return remainder[:end]


def _source_intro_block(text: str) -> str:
    title_end = text.index("\n", text.index("\n# ") + 1)
    return text[title_end + 1 : text.index("\n## What BioMCP exposes")]


def _source_table_block(text: str) -> str:
    return _markdown_section_block(
        text,
        "## What BioMCP exposes\n",
        "\n## Example commands",
    )


def _markdown_blocks(text: str) -> list[str]:
    return [block.strip() for block in re.split(r"\n\s*\n", text.strip()) if block.strip()]


def _example_blocks(text: str) -> list[tuple[str, str]]:
    blocks = []
    for chunk in text.split("```bash\n")[1:]:
        command_block, remainder = chunk.split("\n```", 1)
        description = next(
            (line.strip() for line in remainder.splitlines() if line.strip()),
            "",
        )
        blocks.append((command_block.strip(), description))
    return blocks


def _markdown_links(text: str) -> list[str]:
    return re.findall(r"\[[^\]]+\]\(([^)#]+)\)", text)


def test_docs_sources_directory_has_expected_file_set() -> None:
    actual = sorted(path.name for path in (REPO_ROOT / "docs" / "sources").glob("*.md"))
    assert actual == sorted(EXPECTED_SOURCE_FILES)


def test_mkdocs_nav_contains_sources_section_between_user_guide_and_how_to() -> None:
    mkdocs = _read("mkdocs.yml")
    section = _markdown_section_block(
        mkdocs,
        "  - User Guide:\n",
        "  - How-To:\n",
    )

    assert EXPECTED_NAV_BLOCK in mkdocs
    assert "  - Sources:\n" in section


def test_sources_overview_page_has_required_metadata_and_links() -> None:
    overview = _read_source_page(OVERVIEW_FILE)

    assert (
        _front_matter_value(overview, "title")
        == "Biomedical Data Sources for AI Agents | BioMCP"
    )
    assert (
        _front_matter_value(overview, "description")
        == "Explore BioMCP source guides for PubMed, ClinicalTrials.gov, ClinVar, OpenFDA, CDC WONDER VAERS, UniProt, gnomAD, Reactome, Semantic Scholar, ChEMBL, OpenTargets, SEER Explorer, CIViC, OncoKB, cBioPortal, DDInter, EMA, WHO Prequalification, NCBI Genetic Testing Registry, WHO Prequalified IVD, CDC CVX/MVX, MedlinePlus, KEGG, PharmGKB / CPIC, Human Protein Atlas, and Monarch Initiative."
    )

    assert "# Biomedical Data Sources for AI Agents" in overview
    assert "User Guide" in overview
    assert "Sources" in overview
    assert "[Data Sources](../reference/data-sources.md)" in overview
    assert "[Source Licensing and Terms](../reference/source-licensing.md)" in overview
    assert "[API Keys](../getting-started/api-keys.md)" in overview
    assert "## Example commands" not in overview
    assert "```bash" not in overview

    for filename in SOURCE_PAGE_SPECS:
        assert f"]({filename})" in overview


def test_each_source_page_has_required_front_matter_headings_intro_and_examples() -> None:
    for filename, spec in SOURCE_PAGE_SPECS.items():
        page = _read_source_page(filename)

        assert _front_matter_value(page, "title") == spec["title"]
        assert _front_matter_value(page, "description") == spec["description"]

        heading_positions = []
        for heading in SOURCE_PAGE_SECTION_HEADINGS:
            assert heading in page, f"{filename} missing heading {heading!r}"
            heading_positions.append(page.index(heading))
        assert heading_positions == sorted(heading_positions), (
            f"{filename} headings are out of order"
        )

        intro = _source_intro_block(page)
        assert len(_markdown_blocks(intro)) == 2, (
            f"{filename} intro should be exactly two paragraphs"
        )
        for phrase in spec["required_intro_phrases"]:
            assert phrase in intro, f"{filename} missing intro phrase {phrase!r}"

        examples = _markdown_section_block(page, "## Example commands\n", "\n## API access")
        example_blocks = _example_blocks(examples)
        command_count = len(example_blocks)
        assert command_count == len(spec["example_commands"])
        assert 3 <= command_count <= 5, f"{filename} has {command_count} example commands"
        assert examples.count("```bash\n") == command_count

        for command in spec["example_commands"]:
            assert command in examples, f"{filename} missing example {command!r}"

        for _, description in example_blocks:
            assert description, f"{filename} has an example without a description"
            assert not description.startswith(("```", "## ")), (
                f"{filename} example is missing prose after a bash block"
            )
            assert description.endswith("."), (
                f"{filename} example description should be a sentence"
            )


def test_each_source_page_includes_expected_surface_auth_and_official_link() -> None:
    for filename, spec in SOURCE_PAGE_SPECS.items():
        page = _read_source_page(filename)
        source_table = _source_table_block(page)

        assert spec["api_access"] in page
        assert spec["official_url"] in page

        for command in spec["exposes"]:
            assert command in source_table, f"{filename} missing source surface {command!r}"

        if filename == "who-ivd.md":
            assert (
                "Minimum-length word/phrase boundary match over `Pathogen/Disease/Marker`"
                in source_table
            )


def test_pubmed_source_page_fulltext_contract_matches_current_ladder() -> None:
    page = _read_source_page("pubmed.md")
    source_table = _source_table_block(page)
    examples = _markdown_section_block(page, "## Example commands\n", "\n## API access")

    for phrase in (
        "Europe PMC",
        "NCBI E-utilities",
        "NCBI ID Converter",
        "PMC OA",
        "PMC HTML",
        "opt-in Semantic Scholar PDF",
    ):
        assert phrase in source_table

    assert (
        "NCBI ID Converter bridges PMID/DOI identifiers to PMCID before the "
        "PMCID-dependent source attempts"
    ) in page
    assert "Semantic Scholar PDF is attempted only when the caller passes `--pdf`" in page
    assert "biomcp get article 27083046 fulltext" in examples
    assert "`--pdf`" in examples
    assert "`Saved to:`" in examples
    assert "full text and PDFs remain governed by article-level licenses" in page


def test_semantic_scholar_source_page_documents_auth_status_and_backoff_contract() -> None:
    page = _read_source_page("semantic-scholar.md")
    source_table = _source_table_block(page)

    for phrase in (
        "`--source semantic-scholar` is not a public source switch",
        "shared unauthenticated pool at\n1 req/2sec",
        "authenticated requests at 1 req/sec",
        "honors authenticated\n`Retry-After` responses",
        "never print the\nsecret key or key prefix",
        "`_meta.source_status[]`",
        "`--debug-plan` mirrors that redacted status",
        "`ok`, `degraded`, and `unavailable`",
    ):
        assert phrase in page

    assert "source status" in source_table


def test_each_source_page_has_three_to_four_related_doc_links_that_resolve() -> None:
    for filename in SOURCE_PAGE_SPECS:
        page = _read_source_page(filename)
        related_docs = _markdown_section_block(page, "## Related docs\n", "")
        links = _markdown_links(related_docs)

        assert 3 <= len(links) <= 4, f"{filename} should link 3-4 related docs"

        page_dir = REPO_ROOT / "docs" / "sources"
        for link in links:
            assert "://" not in link, f"{filename} related docs should be internal links"
            assert link.endswith(".md"), f"{filename} related docs should point to markdown"
            resolved = (page_dir / link).resolve()
            assert resolved.exists(), f"{filename} related doc does not exist: {link}"


def test_optional_key_pages_link_api_keys_guide() -> None:
    for filename in ("pubmed.md", "openfda.md", "semantic-scholar.md"):
        page = _read_source_page(filename)
        assert "[API Keys](../getting-started/api-keys.md)" in page


def test_optional_key_pages_link_their_provider_key_pages() -> None:
    for filename, spec in SOURCE_PAGE_SPECS.items():
        provider_key_url = spec.get("provider_key_url")
        if provider_key_url is None:
            continue

        page = _read_source_page(filename)
        assert provider_key_url in page

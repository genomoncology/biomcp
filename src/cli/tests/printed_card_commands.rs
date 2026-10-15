//! Round-trip contract for commands printed by detail-card markdown.
//!
//! Adding a detail-card family that prints commands requires adding its fixture to
//! `detail_card_commands_parse`.

use crate::cli::Cli;
use clap::Parser;

fn printed_commands(markdown: &str) -> Vec<String> {
    let mut commands = std::collections::BTreeSet::new();
    for line in markdown.lines() {
        for code in markdown_code_spans(line) {
            if let Some(start) = code.find("biomcp ") {
                commands.insert(code[start..].trim().to_string());
            }
        }
        let plain = line.trim();
        if let Some(command) = plain.strip_prefix("biomcp ") {
            let command = command
                .split_once("   - ")
                .map_or(command, |(command, _)| command);
            commands.insert(format!("biomcp {}", command.trim()));
        }
    }
    commands.into_iter().collect()
}

fn markdown_code_spans(line: &str) -> Vec<&str> {
    let bytes = line.as_bytes();
    let mut spans = Vec::new();
    let mut cursor = 0;
    while cursor < bytes.len() {
        let Some(relative_start) = bytes[cursor..].iter().position(|byte| *byte == b'`') else {
            break;
        };
        let start = cursor + relative_start;
        let delimiter_len = bytes[start..]
            .iter()
            .take_while(|byte| **byte == b'`')
            .count();
        let content_start = start + delimiter_len;
        let mut search = content_start;
        let mut close = None;
        while search < bytes.len() {
            let Some(relative_tick) = bytes[search..].iter().position(|byte| *byte == b'`') else {
                break;
            };
            let tick = search + relative_tick;
            let run = bytes[tick..]
                .iter()
                .take_while(|byte| **byte == b'`')
                .count();
            if run == delimiter_len {
                close = Some(tick);
                break;
            }
            search = tick + run;
        }
        let Some(end) = close else {
            break;
        };
        spans.push(&line[content_start..end]);
        cursor = end + delimiter_len;
    }
    spans
}

fn parse_printed_commands(markdown: &str) -> Result<usize, String> {
    let commands = printed_commands(markdown);
    if commands.is_empty() {
        return Err("card fixture did not print a biomcp command".to_string());
    }
    for command in &commands {
        let argv = shlex::split(command)
            .ok_or_else(|| format!("rendered command is not valid shell text: `{command}`"))?;
        Cli::try_parse_from(argv)
            .map_err(|error| format!("rendered command does not parse: `{command}`: {error}"))?;
    }
    Ok(commands.len())
}

fn render_fixtures() -> Vec<(&'static str, String)> {
    use crate::render::markdown::*;

    let variant = serde_json::from_value(serde_json::json!({
        "gene": "BRAF",
        "id": "chr7:g.140453136A>T",
        "hgvs_p": "p.Val600Glu",
        "rsid": "rs113488022"
    }))
    .expect("variant fixture");
    let gene = serde_json::from_value(serde_json::json!({
        "symbol": "BRAF",
        "name": "B-Raf proto-oncogene",
        "entrez_id": "673",
        "aliases": []
    }))
    .expect("gene fixture");
    let disease = serde_json::from_value(serde_json::json!({
        "id": "MONDO:0005105",
        "name": "melanoma"
    }))
    .expect("disease fixture");
    let drug = serde_json::from_value(serde_json::json!({
        "name": "drug with space"
    }))
    .expect("drug fixture");
    let trial = serde_json::from_value(serde_json::json!({
        "nct_id": "NCT01234567",
        "title": "Quoted intervention trial",
        "status": "Recruiting",
        "conditions": ["rare disease subtype"],
        "interventions": [{"id": 1, "name": "drug with space", "type": null, "description": null, "other_names": []}]
    }))
    .expect("trial fixture");
    let article = serde_json::from_value(serde_json::json!({
        "pmid": "22663011",
        "title": "BRAF article fixture",
        "author_count": 1,
        "author_completeness": "complete",
        "author_source": "pubmed"
    }))
    .expect("article fixture");
    let adverse_event = serde_json::from_value(serde_json::json!({
        "report_id": "10329882",
        "drug": "drug with space",
        "reactions": ["Cough"],
        "serious": false
    }))
    .expect("adverse-event fixture");
    let diagnostic = serde_json::from_value(serde_json::json!({
        "source": "gtr",
        "source_id": "GTR000000001.1",
        "accession": "GTR000000001.1",
        "name": "Diagnostic fixture"
    }))
    .expect("diagnostic fixture");
    let protein = serde_json::from_value(serde_json::json!({
        "accession": "P15056",
        "name": "BRAF protein",
        "gene_symbol": "BRAF"
    }))
    .expect("protein fixture");
    let pgx = serde_json::from_value(serde_json::json!({
        "query": "drug with space",
        "gene": "CYP2D6",
        "drug": "drug with space"
    }))
    .expect("PGx fixture");
    let pathway = serde_json::from_value(serde_json::json!({
        "source": "Reactome",
        "id": "R-HSA-5673001",
        "name": "RAF/MAP kinase cascade"
    }))
    .expect("pathway fixture");

    let author_id: crate::entities::author::ProviderAuthorId =
        "semanticscholar:1716151".parse().expect("author ID");
    let author = crate::entities::author::AuthorDetail {
        identity: crate::entities::author::AuthorIdentity::ExactProvider {
            id: author_id.clone(),
        },
        display_name: "A. Butte".into(),
        provider_records: vec![crate::entities::author::ProviderAuthorRecord {
            id: author_id,
            source: "semantic_scholar",
            status: crate::entities::author::ProviderStatus::Available,
        }],
        affiliations: vec![],
        paper_count: None,
        citation_count: None,
        h_index: None,
        conflicts: vec![],
        warnings: vec![],
        _meta: crate::entities::author::AuthorMeta {
            source_status: vec![],
            evidence_urls: vec![],
            next_commands: vec!["biomcp author papers semanticscholar:1716151".to_string()],
        },
    };

    vec![
        (
            "variant",
            variant_markdown(&variant, &[]).expect("variant card"),
        ),
        ("gene", gene_markdown(&gene, &[]).expect("gene card")),
        (
            "disease",
            disease_markdown(&disease, &[]).expect("disease card"),
        ),
        ("drug", drug_markdown(&drug, &[]).expect("drug card")),
        ("trial", trial_markdown(&trial, &[]).expect("trial card")),
        (
            "article",
            article_markdown(&article, &[]).expect("article card"),
        ),
        ("author", author_detail_markdown(&author)),
        (
            "adverse-event",
            adverse_event_markdown(&adverse_event, &[]).expect("adverse-event card"),
        ),
        (
            "diagnostic",
            diagnostic_markdown(&diagnostic, &[]).expect("diagnostic card"),
        ),
        (
            "protein",
            protein_markdown(&protein, &[]).expect("protein card"),
        ),
        ("PGx", pgx_markdown(&pgx, &[]).expect("PGx card")),
        (
            "pathway",
            pathway_markdown(&pathway, &[]).expect("pathway card"),
        ),
    ]
}

#[test]
fn detail_card_commands_parse() {
    for (family, markdown) in render_fixtures() {
        parse_printed_commands(&markdown)
            .unwrap_or_else(|error| panic!("{family} detail card: {error}\n{markdown}"));
    }
}

#[test]
fn detail_card_contract_rejects_malformed_command() {
    let malformed = "See also: `biomcp get --definitely-not-a-real-option value`";
    assert_eq!(
        printed_commands(malformed).len(),
        1,
        "the malformed command must first be extracted"
    );
    assert!(
        parse_printed_commands(malformed).is_err(),
        "the extracted malformed command must be rejected"
    );
}

#[test]
fn recovery_commands_round_trip_to_their_registered_routes() {
    use crate::entities::section_outcome::{SectionOutcome, SectionOutcomes};

    for row in crate::entities::source_state_registry::SOURCE_STATE_ROWS {
        let identity = "BR` AF;&";
        let mut outcomes = SectionOutcomes::with_keys(&[row.key]);
        let key = row.key;
        outcomes.complete(key, SectionOutcome::unavailable("provider unavailable"));
        commands =
            crate::render::markdown::section_recovery_commands(row.entity, identity, &outcomes);
        assert_eq!(commands.len(), 1, "{}/{}", row.entity, row.key);
        let command = &commands[0];
        let markdown = format!(
            "Retry: {}",
            crate::render::markdown::markdown_command_code_span(command)
        );
        let extracted = printed_commands(&markdown);
        assert_eq!(extracted, commands);
        let argv = shlex::split(command).expect("valid shell command");
        Cli::try_parse_from(argv).unwrap_or_else(|error| {
            panic!(
                "{}/{} recovery does not parse: {command}: {error}",
                row.entity, row.key
            )
        });
    }
}

/// The recorded BRAF V600E answers this proof replays: the MyVariant
/// gene+protein query and the GRCh37 coordinate lookup both carry the
/// cached CIViC evidence, while the GRCh38 coordinate lookup does not.
const BRAF_V600E_QUERY_REPLY: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/testdata/sources/myvariant/query_braf_v600e_20261009.json"
));
const BRAF_V600E_GRCH37_REPLY: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/testdata/sources/myvariant/get_braf_v600e_grch37_20261007.json"
));
const BRAF_V600E_GRCH38_REPLY: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/testdata/sources/myvariant/get_braf_v600e_grch38_20260806.json"
));
/// The direct ClinVar record for VariationID 13961 (BRAF V600E), captured
/// 2026-10-15: 38 RCV rows, each carrying `SubmissionCount`, never a
/// per-row `NumberOfSubmitters`.
const BRAF_V600E_CLINVAR_XML: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/testdata/sources/ncbi_efetch/clinvar_13961_20261015.xml"
));
/// The RefSNP record for rs113488022, whose single GRCh38 chromosome
/// placement resolves the card's population section to
/// chr7:g.140753336A>T.
const DBSNP_113488022_REPLY: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/testdata/sources/dbsnp/refsnp_113488022_20261015.json"
));

/// The live CIViC GraphQL answer for the profile name the card spells from
/// its protein change ("BRAF Val600Glu"): CIViC knows the variant as
/// "BRAF V600E", so the accepted-evidence count the section prints is
/// zero and the cached MyVariant copy is the evidence the card counts.
const BRAF_V600E_CIVIC_GRAPHQL_REPLY: &str =
    r#"{"data":{"evidenceItems":{"totalCount":0,"nodes":[]},"assertions":{"totalCount":0,"nodes":[]}}}"#;

const BRAF_V600E_GNOMAD_REPLY: &str = r#"{"data":{"variant":{"variant_id":"7-140753336-A-T","exome":{"ac":2,"an":2000,"homozygote_count":0,"hemizygote_count":0,"filters":[],"faf95":null,"populations":[]},"genome":null}}}"#;

const HEADLINE_COORDINATE: &str = "chr7:g.140453136A>T";
const RESOLVED_GRCH38_COORDINATE: &str = "chr7:g.140753336A>T";

struct RecordedReplyEnv(Vec<(&'static str, Option<std::ffi::OsString>)>);

impl RecordedReplyEnv {
    fn set(&mut self, name: &'static str, value: &str) {
        self.0.push((name, std::env::var_os(name)));
        // SAFETY: this test holds the serial-test process-wide environment lock.
        unsafe { std::env::set_var(name, value) };
    }
}

impl Drop for RecordedReplyEnv {
    fn drop(&mut self) {
        for (name, prior) in self.0.drain(..).rev() {
            // SAFETY: this test holds the serial-test process-wide environment lock.
            unsafe {
                if let Some(value) = prior {
                    std::env::set_var(name, value);
                } else {
                    std::env::remove_var(name);
                }
            }
        }
    }
}

fn braf_v600e_recorded_reply(request: &str) -> (&'static str, &'static str, Vec<u8>) {
    let target = request.split_whitespace().nth(1).unwrap_or_default();
    if target.starts_with("/v1/query?") && target.contains("Val600Glu") {
        return (
            "200 OK",
            "application/json",
            BRAF_V600E_QUERY_REPLY.to_vec(),
        );
    }
    if target.starts_with("/v1/variant/chr7:g.140453136A%3ET") {
        return if target.contains("assembly=hg19") {
            ("200 OK", "application/json", BRAF_V600E_GRCH37_REPLY.to_vec())
        } else {
            (
                "404 Not Found",
                "application/json",
                br#"{"code":404,"success":false,"error":"Not Found."}"#.to_vec(),
            )
        };
    }
    if target.starts_with("/v1/variant/chr7:g.140753336A%3ET") {
        return if target.contains("assembly=hg38") {
            ("200 OK", "application/json", BRAF_V600E_GRCH38_REPLY.to_vec())
        } else {
            (
                "404 Not Found",
                "application/json",
                br#"{"code":404,"success":false,"error":"Not Found."}"#.to_vec(),
            )
        };
    }
    if target.starts_with("/efetch.fcgi?")
        && target.contains("db=clinvar")
        && target.contains("id=13961")
    {
        return (
            "200 OK",
            "application/xml",
            BRAF_V600E_CLINVAR_XML.to_vec(),
        );
    }
    if target.starts_with("/refsnp/113488022") {
        return (
            "200 OK",
            "application/json",
            DBSNP_113488022_REPLY.to_vec(),
        );
    }
    if request.starts_with("POST /graphql") {
        return (
            "200 OK",
            "application/json",
            BRAF_V600E_CIVIC_GRAPHQL_REPLY.as_bytes().to_vec(),
        );
    }
    if request.starts_with("POST /") && request.contains("7-140753336-A-T") {
        return (
            "200 OK",
            "application/json",
            BRAF_V600E_GNOMAD_REPLY.as_bytes().to_vec(),
        );
    }
    (
        "404 Not Found",
        "application/json",
        br#"{"error":"fixture path not found"}"#.to_vec(),
    )
}

async fn braf_v600e_recorded_reply_server() -> (String, tokio::task::JoinHandle<()>) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind BRAF V600E recorded-reply fixture");
    let base = format!("http://{}", listener.local_addr().expect("fixture address"));
    let task = tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut request = vec![0_u8; 64 * 1024];
                let len = stream
                    .read(&mut request)
                    .await
                    .expect("read fixture request");
                let request = String::from_utf8_lossy(&request[..len]).into_owned();
                let (status, content_type, body) = braf_v600e_recorded_reply(&request);
                let response = format!(
                    "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                stream
                    .write_all(response.as_bytes())
                    .await
                    .expect("write fixture response head");
                stream
                    .write_all(&body)
                    .await
                    .expect("write fixture response body");
            });
        }
    });
    (base, task)
}

fn quoted_coordinate(command: &str) -> Option<&str> {
    let start = command.find('"')? + 1;
    let end = start + command[start..].find('"')?;
    Some(&command[start..end])
}

/// The recorded-reply proof for the ticket: replaying the live BRAF V600E
/// answers for `get variant "BRAF V600E" all` keeps every printed
/// follow-up on the headline coordinate (not the dbSNP-resolved GRCh38
/// coordinate the population section stores), and following the printed
/// civic command reports the evidence items the card counted.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn braf_v600e_card_prints_follow_ups_on_the_headline_coordinate() {
    let (base, server) = braf_v600e_recorded_reply_server().await;
    let cache_dir = tempfile::tempdir().expect("temp cache dir");
    let mut env = RecordedReplyEnv(Vec::new());
    env.set("BIOMCP_CACHE_MODE", "off");
    env.set(
        "BIOMCP_CACHE_DIR",
        cache_dir.path().to_str().expect("utf-8 temp path"),
    );
    env.set("BIOMCP_MYVARIANT_BASE", &format!("{base}/v1"));
    env.set("BIOMCP_DBSNP_BASE", &base);
    env.set("BIOMCP_GNOMAD_BASE", &base);
    env.set("BIOMCP_CLINVAR_BASE", &base);
    env.set("BIOMCP_CIVIC_BASE", &base);
    env.set("BIOMCP_CBIOPORTAL_BASE", &base);
    env.set("BIOMCP_CANCERHOTSPOTS_BASE", &base);
    env.set("BIOMCP_GWAS_BASE", &format!("{base}/gwas/rest/api"));

    let variant = crate::entities::variant::get("BRAF V600E", &["all".to_string()])
        .await
        .expect("recorded BRAF V600E answer");
    let markdown = crate::render::markdown::variant_markdown(&variant, &["all".to_string()])
        .expect("rendered BRAF V600E card");

    // The population section still resolves and keeps the GRCh38
    // coordinate on its own line; gnomAD answered on it.
    assert!(
        markdown.contains(&format!(
            "Resolved GRCh38 coordinate: {RESOLVED_GRCH38_COORDINATE} (dbSNP)"
        )),
        "{markdown}"
    );
    assert!(markdown.contains("Exome overall frequency:"), "{markdown}");

    // The direct NCBI ClinVar rows print their submission counts: the
    // record-level submitter count, and the RCV rows' `SubmissionCount`.
    assert!(markdown.contains("VCV submitters: 22"), "{markdown}");
    assert!(
        markdown.contains("evaluated 2014-09-04; 1 submission(s)"),
        "{markdown}"
    );

    // Every printed command that names the variant carries the headline
    // coordinate, and no printed command names the resolved one.
    for command in printed_commands(&markdown) {
        assert!(
            !command.contains(RESOLVED_GRCH38_COORDINATE),
            "printed command leaves the headline coordinate: {command}"
        );
        let names_the_variant = command.contains("get variant ")
            || command.contains(" variant trials")
            || command.contains(" variant articles")
            || command.contains(" variant oncokb");
        if names_the_variant {
            assert_eq!(
                quoted_coordinate(&command),
                Some(HEADLINE_COORDINATE),
                "printed command leaves the headline coordinate: {command}"
            );
        }
    }

    // The headline pointer counts the cached CIViC evidence and prints the
    // civic follow-up on the headline coordinate.
    let pointer = markdown
        .lines()
        .find(|line| line.starts_with("Therapeutic evidence: "))
        .expect("the card prints its CIViC pointer");
    let civic_pointer_command = pointer
        .rsplit_once("see `")
        .and_then(|(_, rest)| rest.strip_suffix('`'))
        .expect("the pointer names a command");
    assert_eq!(
        civic_pointer_command,
        format!("get variant \"{HEADLINE_COORDINATE}\" civic")
    );
    let counted_predictive = pointer
        .strip_prefix("Therapeutic evidence: ")
        .and_then(|rest| rest.split(' ').next())
        .and_then(|count| count.parse::<usize>().ok())
        .expect("the pointer counts its predictive evidence");
    assert_eq!(counted_predictive, 13);

    // Following the printed civic command reports the counted evidence.
    let argv = shlex::split(&format!("biomcp {civic_pointer_command}"))
        .expect("printed civic command parses");
    Cli::try_parse_from(argv.iter().cloned()).expect("printed civic command routes");
    let follow_up_id = quoted_coordinate(civic_pointer_command).expect("quoted coordinate");
    let follow_up = crate::entities::variant::get(follow_up_id, &["civic".to_string()])
        .await
        .expect("recorded civic follow-up answer");
    let follow_up_markdown =
        crate::render::markdown::variant_markdown(&follow_up, &["civic".to_string()])
            .expect("rendered civic follow-up card");
    assert!(
        follow_up_markdown.contains("### Cached Evidence (MyVariant)"),
        "{follow_up_markdown}"
    );
    assert_eq!(
        follow_up_markdown.matches("| PREDICTIVE |").count(),
        counted_predictive,
        "the civic follow-up reports the evidence the card counted:\n{follow_up_markdown}"
    );
    server.abort();
}

use std::process::{Command, Output};

use biomcp_mcp_contract_client::{
    article_fulltext_fixture_env, provision_article_fulltext_fixture,
};
use serde_json::json;

const COMPACT_ORDERED_JSON: &str = r###"{
  "items": [
    {
      "input": "22663011",
      "result": {
        "author_completeness": "complete",
        "author_count": 6,
        "author_source": "pubtator",
        "authors": [
          "Ada First",
          "Ben Second",
          "Cyra Middle",
          "Dev Fourth",
          "Eli Fifth",
          "Fay Last"
        ],
        "citation_count": 12,
        "influential_citation_count": 3,
        "journal": "Journal One",
        "pmcid": "PMC123456",
        "pmid": "22663011",
        "requested_id": "22663011",
        "title": "Europe full text winner",
        "tldr": "Fixture compact summary",
        "year": 2025
      },
      "status": "ok"
    },
    {
      "input": "22663012",
      "result": {
        "author_completeness": "unavailable",
        "author_count": 0,
        "author_source": "pubtator",
        "authors": [],
        "journal": "Journal One",
        "pmcid": "PMC123457",
        "pmid": "22663012",
        "requested_id": "22663012",
        "title": "PMC HTML fallback winner",
        "year": 2025
      },
      "status": "ok"
    }
  ],
  "summary": {
    "failed": 0,
    "succeeded": 2,
    "total": 2
  }
}
"###;

const COMPACT_DUPLICATE_JSON: &str = r###"{
  "items": [
    {
      "input": "22663011",
      "result": {
        "author_completeness": "complete",
        "author_count": 6,
        "author_source": "pubtator",
        "authors": [
          "Ada First",
          "Ben Second",
          "Cyra Middle",
          "Dev Fourth",
          "Eli Fifth",
          "Fay Last"
        ],
        "citation_count": 12,
        "influential_citation_count": 3,
        "journal": "Journal One",
        "pmcid": "PMC123456",
        "pmid": "22663011",
        "requested_id": "22663011",
        "title": "Europe full text winner",
        "tldr": "Fixture compact summary",
        "year": 2025
      },
      "status": "ok"
    },
    {
      "input": "22663011",
      "result": {
        "author_completeness": "complete",
        "author_count": 6,
        "author_source": "pubtator",
        "authors": [
          "Ada First",
          "Ben Second",
          "Cyra Middle",
          "Dev Fourth",
          "Eli Fifth",
          "Fay Last"
        ],
        "citation_count": 12,
        "influential_citation_count": 3,
        "journal": "Journal One",
        "pmcid": "PMC123456",
        "pmid": "22663011",
        "requested_id": "22663011",
        "title": "Europe full text winner",
        "tldr": "Fixture compact summary",
        "year": 2025
      },
      "status": "ok"
    }
  ],
  "summary": {
    "failed": 0,
    "succeeded": 2,
    "total": 2
  }
}
"###;

const COMPACT_MIXED_JSON: &str = r###"{
  "items": [
    {
      "input": "22663011",
      "result": {
        "author_completeness": "complete",
        "author_count": 6,
        "author_source": "pubtator",
        "authors": [
          "Ada First",
          "Ben Second",
          "Cyra Middle",
          "Dev Fourth",
          "Eli Fifth",
          "Fay Last"
        ],
        "citation_count": 12,
        "influential_citation_count": 3,
        "journal": "Journal One",
        "pmcid": "PMC123456",
        "pmid": "22663011",
        "requested_id": "22663011",
        "title": "Europe full text winner",
        "tldr": "Fixture compact summary",
        "year": 2025
      },
      "status": "ok"
    },
    {
      "error": {
        "code": "invalid_argument",
        "message": "Invalid argument: Unsupported identifier format. BioMCP resolves PMID (digits only, e.g., 22663011), PMCID (starts with PMC, e.g., PMC9984800), and DOI (starts with 10., e.g., 10.1056/NEJMoa1203421). publisher PIIs (e.g., S1535610826000103) are not indexed by PubMed or Europe PMC and cannot be resolved."
      },
      "input": "not-an-article-id",
      "status": "error"
    },
    {
      "input": "22663011",
      "result": {
        "author_completeness": "complete",
        "author_count": 6,
        "author_source": "pubtator",
        "authors": [
          "Ada First",
          "Ben Second",
          "Cyra Middle",
          "Dev Fourth",
          "Eli Fifth",
          "Fay Last"
        ],
        "citation_count": 12,
        "influential_citation_count": 3,
        "journal": "Journal One",
        "pmcid": "PMC123456",
        "pmid": "22663011",
        "requested_id": "22663011",
        "title": "Europe full text winner",
        "tldr": "Fixture compact summary",
        "year": 2025
      },
      "status": "ok"
    }
  ],
  "summary": {
    "failed": 1,
    "succeeded": 2,
    "total": 3
  }
}
"###;

const S2_CONSTRUCTION_FAILURE_JSON: &str = r###"{
  "items": [
    {
      "error": {
        "code": "api",
        "message": "API request to Semantic Scholar failed.",
        "recovery": "Retry the remote source.",
        "source": "Semantic Scholar"
      },
      "input": "22663011",
      "status": "error"
    }
  ],
  "summary": {
    "failed": 1,
    "succeeded": 0,
    "total": 1
  }
}
"###;

const S2_SUCCESS_JSON: &str = r###"{
  "items": [
    {
      "input": "22663011",
      "result": {
        "author_completeness": "complete",
        "author_count": 6,
        "author_source": "pubtator",
        "authors": [
          "Ada First",
          "Ben Second",
          "Cyra Middle",
          "Dev Fourth",
          "Eli Fifth",
          "Fay Last"
        ],
        "citation_count": 12,
        "influential_citation_count": 3,
        "journal": "Journal One",
        "pmcid": "PMC123456",
        "pmid": "22663011",
        "requested_id": "22663011",
        "title": "Europe full text winner",
        "tldr": "Fixture compact summary",
        "year": 2025
      },
      "status": "ok"
    }
  ],
  "summary": {
    "failed": 0,
    "succeeded": 1,
    "total": 1
  }
}
"###;

const S2_FAIL_OPEN_JSON: &str = r###"{
  "items": [
    {
      "input": "22663012",
      "result": {
        "author_completeness": "unavailable",
        "author_count": 0,
        "author_source": "pubtator",
        "authors": [],
        "journal": "Journal One",
        "pmcid": "PMC123457",
        "pmid": "22663012",
        "requested_id": "22663012",
        "title": "PMC HTML fallback winner",
        "year": 2025
      },
      "status": "ok"
    }
  ],
  "summary": {
    "failed": 0,
    "succeeded": 1,
    "total": 1
  }
}
"###;

const DETAIL_JSON: &str = r###"{
  "items": [
    {
      "input": "22663011",
      "result": {
        "_meta": {
          "evidence_urls": [
            {
              "label": "PubMed",
              "url": "https://pubmed.ncbi.nlm.nih.gov/22663011/"
            },
            {
              "label": "PMC",
              "url": "https://pmc.ncbi.nlm.nih.gov/articles/PMC123456/"
            }
          ],
          "next_commands": [
            "biomcp article references 22663011 --limit 3",
            "biomcp article citations 22663011 --limit 3",
            "biomcp article recommendations 22663011 --limit 3"
          ],
          "section_sources": [
            {
              "key": "bibliography",
              "label": "Bibliography",
              "outcome": "data",
              "sources": [
                "PubMed",
                "Europe PMC"
              ]
            },
            {
              "key": "authors",
              "label": "Authors",
              "outcome": "data",
              "sources": [
                "PubTator3"
              ]
            },
            {
              "key": "abstract",
              "label": "Abstract",
              "outcome": "data",
              "sources": [
                "PubMed",
                "Europe PMC"
              ]
            },
            {
              "key": "tldr",
              "label": "Semantic Scholar",
              "outcome": "data",
              "sources": [
                "Semantic Scholar"
              ]
            }
          ]
        },
        "abstract_text": "Abstract text.",
        "author_completeness": "complete",
        "author_count": 6,
        "author_source": "pubtator",
        "authors": [
          "Ada First",
          "Ben Second",
          "Cyra Middle",
          "Dev Fourth",
          "Eli Fifth",
          "Fay Last"
        ],
        "date": "2025-01-01",
        "journal": "Journal One",
        "open_access": true,
        "pmcid": "PMC123456",
        "pmid": "22663011",
        "pubtator_fallback": false,
        "section_outcomes": {
          "fulltext": {
            "outcome": "not_requested",
            "sources": []
          },
          "indexing": {
            "outcome": "not_requested",
            "sources": []
          },
          "tldr": {
            "outcome": "data",
            "sources": [
              "Semantic Scholar"
            ]
          }
        },
        "semantic_scholar": {
          "citation_count": 12,
          "influential_citation_count": 3,
          "paper_id": "paper-1",
          "tldr": "Fixture detail summary"
        },
        "title": "Europe full text winner"
      },
      "status": "ok"
    }
  ],
  "summary": {
    "failed": 0,
    "succeeded": 1,
    "total": 1
  }
}
"###;

fn run_compatibility(json: bool) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_biomcp"));
    command.env("RUST_LOG", "off");
    command.args(["--no-cache"]);
    if json {
        command.arg("--json");
    }
    command
        .args(["article", "batch", "not-an-article-id", "also-invalid"])
        .output()
        .expect("run article batch")
}

fn run_canonical(json: bool) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_biomcp"));
    command.env("RUST_LOG", "off");
    command.args(["--no-cache"]);
    if json {
        command.arg("--json");
    }
    command
        .args([
            "batch",
            "article",
            "not-an-article-id,also-invalid",
            "--mode",
            "compact",
        ])
        .output()
        .expect("run canonical compact article batch")
}

fn run_compatibility_ids(ids: &[&str], json: bool) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_biomcp"));
    command.env("RUST_LOG", "off");
    command.args(["--no-cache"]);
    if json {
        command.arg("--json");
    }
    command
        .args(["article", "batch"])
        .args(ids)
        .output()
        .unwrap()
}

fn run_canonical_ids(ids: &str, json: bool) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_biomcp"));
    command.env("RUST_LOG", "off");
    command.args(["--no-cache"]);
    if json {
        command.arg("--json");
    }
    command
        .args(["batch", "article", ids, "--mode", "compact"])
        .output()
        .unwrap()
}

fn run_fixture(
    fixture: &biomcp_mcp_contract_client::ArticleFulltextFixture,
    args: &[&str],
) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_biomcp"));
    command.env("RUST_LOG", "off");
    command.args(args);
    for (name, value) in article_fulltext_fixture_env(fixture) {
        command.env(name, value);
    }
    command.output().expect("run fixture-backed article batch")
}

fn run_fixture_with_warnings(
    fixture: &biomcp_mcp_contract_client::ArticleFulltextFixture,
    args: &[&str],
) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_biomcp"));
    command.args(args).env("RUST_LOG", "warn");
    for (name, value) in article_fulltext_fixture_env(fixture) {
        command.env(name, value);
    }
    command.output().expect("run traced article batch fixture")
}

fn run_preflight_sentinel(
    fixture: &biomcp_mcp_contract_client::ArticleFulltextFixture,
    args: &[String],
) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_biomcp"));
    command
        .args(args)
        .env("RUST_LOG", "off")
        .env("BIOMCP_CACHE_DIR", &fixture.cache_dir);
    for name in [
        "BIOMCP_PUBTATOR_BASE",
        "BIOMCP_EUROPEPMC_BASE",
        "BIOMCP_S2_BASE",
    ] {
        command.env(name, "not a provider URL");
    }
    command
        .output()
        .expect("run invalid batch at real CLI boundary")
}

fn compact_ok_22663011() -> serde_json::Value {
    json!({
        "requested_id":"22663011","pmid":"22663011","pmcid":"PMC123456",
        "title":"Europe full text winner","authors":["Ada First","Ben Second","Cyra Middle","Dev Fourth","Eli Fifth","Fay Last"],
        "author_count":6,"author_completeness":"complete","author_source":"pubtator",
        "journal":"Journal One","year":2025,"tldr":"Fixture compact summary",
        "citation_count":12,"influential_citation_count":3
    })
}

fn assert_s2_fail_open_warning(stderr: &[u8]) {
    let warning = std::str::from_utf8(stderr).expect("UTF-8 warning");
    let diagnostic = warning
        .split_once("  WARN ")
        .map_or(warning, |(_, suffix)| suffix);
    assert_eq!(
        diagnostic,
        "biomcp_cli::error: External provider operation failed provider=\"Semantic Scholar\" operation=\"compact article batch enrichment\" class=\"internal\" internal failure\n"
    );
}

const COMPACT_SUCCESS_MARKDOWN: &str = r#"# Batch: article (2)

---

## 22663011 — ok

# Article Batch (1)

## 1. Europe full text winner
PMID: 22663011
Journal: Journal One
Year: 2025
Authors: Ada First, Ben Second, Cyra Middle, Dev Fourth, Eli Fifth, Fay Last
Authorship: complete (6 returned; PubTator3)
TLDR: Fixture compact summary
Citations: 12 (influential: 3)


---

## 22663012 — ok

# Article Batch (1)

## 1. PMC HTML fallback winner
PMID: 22663012
Journal: Journal One
Year: 2025
Authorship: unavailable (no author list supplied by PubTator3)


## Summary

Total: 2; succeeded: 2; failed: 0.

"#;

const DETAIL_MARKDOWN: &str = r#"# Batch: article (1)

---

## 22663011 — ok

# Europe full text winner

PMID: 22663011
PMCID: PMC123456

Journal: Journal One
Date: 2025-01-01


Open Access: Yes
[PubMed](https://pubmed.ncbi.nlm.nih.gov/22663011/)
Source: PubMed / Europe PMC

## Authors (PubTator3)

Ada First, Ben Second, Cyra Middle, Dev Fourth, Eli Fifth, Fay Last
Authorship: complete (6 returned; PubTator3)
## Abstract (PubMed / Europe PMC)

Abstract text.
## Semantic Scholar

Paper ID: paper-1
TLDR: Fixture detail summary
Citations: 12
Influential citations: 3
References:
Open access: No
More:
  biomcp get article 22663011 annotations   - PubTator normalized entity mentions
  biomcp get article 22663011 fulltext   - cached full text when available
  biomcp get article 22663011 tldr   - Semantic Scholar summary and influence
All:
  biomcp get article 22663011 all
See also:
  biomcp article references 22663011 --limit 3   - background evidence this paper builds on; use if the primary paper lacks context
  biomcp article citations 22663011 --limit 3   - later papers that cite this article; use only if the primary paper lacks your answer
  biomcp article recommendations 22663011 --limit 3   - related papers to broaden coverage; use only if the primary paper lacks your answer

[PubMed](https://pubmed.ncbi.nlm.nih.gov/22663011/) | [PMC](https://pmc.ncbi.nlm.nih.gov/articles/PMC123456/)

## Summary

Total: 1; succeeded: 1; failed: 0.

"#;

const MIXED_MARKDOWN: &str = r#"# Batch: article (3)

---

## 22663011 — ok

# Article Batch (1)

## 1. Europe full text winner
PMID: 22663011
Journal: Journal One
Year: 2025
Authors: Ada First, Ben Second, Cyra Middle, Dev Fourth, Eli Fifth, Fay Last
Authorship: complete (6 returned; PubTator3)
TLDR: Fixture compact summary
Citations: 12 (influential: 3)


---

## not-an-article-id — error

Invalid argument: Unsupported identifier format. BioMCP resolves PMID (digits only, e.g., 22663011), PMCID (starts with PMC, e.g., PMC9984800), and DOI (starts with 10., e.g., 10.1056/NEJMoa1203421). publisher PIIs (e.g., S1535610826000103) are not indexed by PubMed or Europe PMC and cannot be resolved.

---

## 22663011 — ok

# Article Batch (1)

## 1. Europe full text winner
PMID: 22663011
Journal: Journal One
Year: 2025
Authors: Ada First, Ben Second, Cyra Middle, Dev Fourth, Eli Fifth, Fay Last
Authorship: complete (6 returned; PubTator3)
TLDR: Fixture compact summary
Citations: 12 (influential: 3)


## Summary

Total: 3; succeeded: 2; failed: 1.

"#;

const JSON_GOLDEN: &str = r#"{
  "items": [
    {
      "error": {
        "code": "invalid_argument",
        "message": "Invalid argument: Unsupported identifier format. BioMCP resolves PMID (digits only, e.g., 22663011), PMCID (starts with PMC, e.g., PMC9984800), and DOI (starts with 10., e.g., 10.1056/NEJMoa1203421). publisher PIIs (e.g., S1535610826000103) are not indexed by PubMed or Europe PMC and cannot be resolved."
      },
      "input": "not-an-article-id",
      "status": "error"
    },
    {
      "error": {
        "code": "invalid_argument",
        "message": "Invalid argument: Unsupported identifier format. BioMCP resolves PMID (digits only, e.g., 22663011), PMCID (starts with PMC, e.g., PMC9984800), and DOI (starts with 10., e.g., 10.1056/NEJMoa1203421). publisher PIIs (e.g., S1535610826000103) are not indexed by PubMed or Europe PMC and cannot be resolved."
      },
      "input": "also-invalid",
      "status": "error"
    }
  ],
  "summary": {
    "failed": 2,
    "succeeded": 0,
    "total": 2
  }
}
"#;

const MARKDOWN_GOLDEN: &str = r#"# Batch: article (2)

---

## not-an-article-id — error

Invalid argument: Unsupported identifier format. BioMCP resolves PMID (digits only, e.g., 22663011), PMCID (starts with PMC, e.g., PMC9984800), and DOI (starts with 10., e.g., 10.1056/NEJMoa1203421). publisher PIIs (e.g., S1535610826000103) are not indexed by PubMed or Europe PMC and cannot be resolved.

---

## also-invalid — error

Invalid argument: Unsupported identifier format. BioMCP resolves PMID (digits only, e.g., 22663011), PMCID (starts with PMC, e.g., PMC9984800), and DOI (starts with 10., e.g., 10.1056/NEJMoa1203421). publisher PIIs (e.g., S1535610826000103) are not indexed by PubMed or Europe PMC and cannot be resolved.

## Summary

Total: 2; succeeded: 0; failed: 2.

"#;

fn detail_expected() -> serde_json::Value {
    json!({
        "items":[{"input":"22663011","status":"ok","result":{
            "pmid":"22663011","pmcid":"PMC123456","title":"Europe full text winner",
            "authors":["Ada First","Ben Second","Cyra Middle","Dev Fourth","Eli Fifth","Fay Last"],
            "author_count":6,"author_completeness":"complete","author_source":"pubtator",
            "journal":"Journal One","date":"2025-01-01","abstract_text":"Abstract text.",
            "open_access":true,"pubtator_fallback":false,
            "semantic_scholar":{"paper_id":"paper-1","tldr":"Fixture detail summary","citation_count":12,"influential_citation_count":3},
            "section_outcomes":{"fulltext":{"outcome":"not_requested","sources":[]},"indexing":{"outcome":"not_requested","sources":[]},"tldr":{"outcome":"data","sources":["Semantic Scholar"]}},
            "_meta":{
                "evidence_urls":[{"label":"PubMed","url":"https://pubmed.ncbi.nlm.nih.gov/22663011/"},{"label":"PMC","url":"https://pmc.ncbi.nlm.nih.gov/articles/PMC123456/"}],
                "next_commands":["biomcp article references 22663011 --limit 3","biomcp article citations 22663011 --limit 3","biomcp article recommendations 22663011 --limit 3"],
                "section_sources":[
                    {"key":"bibliography","label":"Bibliography","outcome":"data","sources":["PubMed","Europe PMC"]},
                    {"key":"authors","label":"Authors","outcome":"data","sources":["PubTator3"]},
                    {"key":"abstract","label":"Abstract","outcome":"data","sources":["PubMed","Europe PMC"]},
                    {"key":"tldr","label":"Semantic Scholar","outcome":"data","sources":["Semantic Scholar"]}
                ]
            }
        }}],"summary":{"total":1,"succeeded":1,"failed":0}
    })
}

#[test]
fn fixture_backed_batches_match_independent_literal_goldens() {
    let fixture = provision_article_fulltext_fixture(env!("CARGO_MANIFEST_DIR"))
        .expect("provision article fixture");
    let second = json!({
        "requested_id":"22663012","pmid":"22663012","pmcid":"PMC123457",
        "title":"PMC HTML fallback winner","authors":[],"author_count":0,
        "author_completeness":"unavailable","author_source":"pubtator",
        "journal":"Journal One","year":2025
    });
    let compact_expected = json!({
        "items":[
            {"input":"22663011","status":"ok","result":compact_ok_22663011()},
            {"input":"22663012","status":"ok","result":second}
        ],
        "summary":{"total":2,"succeeded":2,"failed":0}
    });

    let compact_json = run_fixture(
        &fixture,
        &[
            "--json",
            "batch",
            "article",
            "22663011,22663012",
            "--mode",
            "compact",
        ],
    );
    assert_eq!(compact_json.status.code(), Some(0));
    assert!(compact_json.stderr.is_empty());
    assert_eq!(compact_json.stdout, COMPACT_ORDERED_JSON.as_bytes());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&compact_json.stdout).unwrap(),
        compact_expected
    );
    let compact_markdown = run_fixture(
        &fixture,
        &["batch", "article", "22663011,22663012", "--mode", "compact"],
    );
    assert_eq!(compact_markdown.status.code(), Some(0));
    assert!(compact_markdown.stderr.is_empty());
    assert_eq!(
        String::from_utf8(compact_markdown.stdout).unwrap(),
        COMPACT_SUCCESS_MARKDOWN
    );
    let compatibility_json = run_fixture(
        &fixture,
        &["--json", "article", "batch", "22663011", "22663012"],
    );
    assert_eq!(compatibility_json.status.code(), Some(0));
    assert!(compatibility_json.stderr.is_empty());
    assert_eq!(compatibility_json.stdout, COMPACT_ORDERED_JSON.as_bytes());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&compatibility_json.stdout).unwrap(),
        compact_expected
    );
    let compatibility_markdown =
        run_fixture(&fixture, &["article", "batch", "22663011", "22663012"]);
    assert_eq!(compatibility_markdown.status.code(), Some(0));
    assert!(compatibility_markdown.stderr.is_empty());
    assert_eq!(
        String::from_utf8(compatibility_markdown.stdout).unwrap(),
        COMPACT_SUCCESS_MARKDOWN
    );

    let duplicate = run_fixture(
        &fixture,
        &[
            "--json",
            "batch",
            "article",
            "22663011,22663011",
            "--mode",
            "compact",
        ],
    );
    assert_eq!(duplicate.status.code(), Some(0));
    assert!(duplicate.stderr.is_empty());
    assert_eq!(duplicate.stdout, COMPACT_DUPLICATE_JSON.as_bytes());
    let duplicate_value: serde_json::Value = serde_json::from_slice(&duplicate.stdout).unwrap();
    assert_eq!(duplicate_value["items"][0], duplicate_value["items"][1]);

    let mixed_expected = json!({
        "items":[
            {"input":"22663011","status":"ok","result":compact_ok_22663011()},
            {"input":"not-an-article-id","status":"error","error":{
                "code":"invalid_argument",
                "message":"Invalid argument: Unsupported identifier format. BioMCP resolves PMID (digits only, e.g., 22663011), PMCID (starts with PMC, e.g., PMC9984800), and DOI (starts with 10., e.g., 10.1056/NEJMoa1203421). publisher PIIs (e.g., S1535610826000103) are not indexed by PubMed or Europe PMC and cannot be resolved."
            }},
            {"input":"22663011","status":"ok","result":compact_ok_22663011()}
        ],
        "summary":{"total":3,"succeeded":2,"failed":1}
    });
    let mixed = run_fixture(
        &fixture,
        &[
            "--json",
            "batch",
            "article",
            "22663011,not-an-article-id,22663011",
            "--mode",
            "compact",
        ],
    );
    assert_eq!(mixed.status.code(), Some(1));
    assert!(mixed.stderr.is_empty());
    assert_eq!(mixed.stdout, COMPACT_MIXED_JSON.as_bytes());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&mixed.stdout).unwrap(),
        mixed_expected
    );
    let mixed_markdown = run_fixture(
        &fixture,
        &[
            "batch",
            "article",
            "22663011,not-an-article-id,22663011",
            "--mode",
            "compact",
        ],
    );
    assert_eq!(mixed_markdown.status.code(), Some(1));
    assert!(mixed_markdown.stderr.is_empty());
    assert_eq!(
        String::from_utf8(mixed_markdown.stdout).unwrap(),
        MIXED_MARKDOWN
    );

    let s2_success = run_fixture(
        &fixture,
        &[
            "--json", "batch", "article", "22663011", "--mode", "compact",
        ],
    );
    assert_eq!(s2_success.status.code(), Some(0));
    assert!(s2_success.stderr.is_empty());
    assert_eq!(s2_success.stdout, S2_SUCCESS_JSON.as_bytes());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&s2_success.stdout).unwrap(),
        json!({"items":[{"input":"22663011","status":"ok","result":compact_ok_22663011()}],"summary":{"total":1,"succeeded":1,"failed":0}})
    );
    let s2_fail_open = run_fixture(
        &fixture,
        &[
            "--json", "batch", "article", "22663012", "--mode", "compact",
        ],
    );
    assert_eq!(s2_fail_open.status.code(), Some(0));
    assert_eq!(s2_fail_open.stdout, S2_FAIL_OPEN_JSON.as_bytes());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&s2_fail_open.stdout).unwrap(),
        json!({"items":[{"input":"22663012","status":"ok","result":second}],"summary":{"total":1,"succeeded":1,"failed":0}})
    );
    assert!(s2_fail_open.stderr.is_empty());

    let warned = run_fixture_with_warnings(
        &fixture,
        &[
            "--json", "batch", "article", "22663012", "--mode", "compact",
        ],
    );
    assert_eq!(warned.status.code(), Some(0));
    assert_eq!(warned.stdout, S2_FAIL_OPEN_JSON.as_bytes());
    assert_s2_fail_open_warning(&warned.stderr);

    for args in [
        ["--json", "batch", "article", "22663011", "", ""],
        ["--json", "batch", "article", "22663011", "--mode", "detail"],
    ] {
        let args = args
            .into_iter()
            .filter(|arg| !arg.is_empty())
            .collect::<Vec<_>>();
        let output = run_fixture(&fixture, &args);
        assert_eq!(output.status.code(), Some(0));
        assert!(output.stderr.is_empty());
        assert_eq!(output.stdout, DETAIL_JSON.as_bytes());
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap(),
            detail_expected()
        );
    }
    for args in [
        ["batch", "article", "22663011", "", ""],
        ["batch", "article", "22663011", "--mode", "detail"],
    ] {
        let args = args
            .into_iter()
            .filter(|arg| !arg.is_empty())
            .collect::<Vec<_>>();
        let output = run_fixture(&fixture, &args);
        assert_eq!(output.status.code(), Some(0));
        assert!(output.stderr.is_empty());
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            DETAIL_MARKDOWN.replace("References:\n", "References: \n")
        );
    }
}

#[test]
fn semantic_scholar_client_construction_failure_remains_a_settled_item_error() {
    let fixture = provision_article_fulltext_fixture(env!("CARGO_MANIFEST_DIR"))
        .expect("provision article fixture");
    let run = |args: &[&str]| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_biomcp"));
        command.args(args).env("RUST_LOG", "off");
        for (name, value) in article_fulltext_fixture_env(&fixture) {
            command.env(name, value);
        }
        command
            .env("BIOMCP_S2_BASE", "not a provider URL")
            .output()
            .expect("run S2 construction failure")
    };
    let canonical = run(&[
        "--json", "batch", "article", "22663011", "--mode", "compact",
    ]);
    let compatibility = run(&["--json", "article", "batch", "22663011"]);
    assert_eq!(canonical.status.code(), Some(1));
    assert_eq!(canonical.stdout, compatibility.stdout);
    assert_eq!(canonical.stderr, compatibility.stderr);
    assert!(canonical.stderr.is_empty());
    assert_eq!(canonical.stdout, S2_CONSTRUCTION_FAILURE_JSON.as_bytes());
    let payload: serde_json::Value = serde_json::from_slice(&canonical.stdout).unwrap();
    assert_eq!(
        payload["summary"],
        json!({"total":1,"succeeded":0,"failed":1})
    );
    assert_eq!(payload["items"][0]["input"], "22663011");
    assert_eq!(payload["items"][0]["status"], "error");
    assert_eq!(payload["items"][0]["error"]["code"], "api");
}

#[test]
fn failed_article_batch_report_stays_on_stdout_in_json_and_markdown() {
    for json in [false, true] {
        let output = run_compatibility(json);
        let canonical = run_canonical(json);
        assert_eq!(canonical.status, output.status);
        assert_eq!(canonical.stdout, output.stdout);
        assert_eq!(canonical.stderr, output.stderr);
        assert_eq!(output.status.code(), Some(1));
        assert!(
            output.stderr.is_empty(),
            "stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).expect("UTF-8 report");
        assert_eq!(stdout, if json { JSON_GOLDEN } else { MARKDOWN_GOLDEN });
        if json {
            let value: serde_json::Value = serde_json::from_str(&stdout).expect("batch JSON");
            assert_eq!(value["summary"]["failed"], 2);
            assert_eq!(value["items"].as_array().map(Vec::len), Some(2));
        } else {
            assert!(stdout.contains("# Batch: article (2)"));
            assert!(stdout.contains("Total: 2; succeeded: 0; failed: 2."));
        }
    }
}

#[test]
fn compatibility_preserves_positional_bytes_through_512_and_breaks_only_above_it() {
    let boundary = "x".repeat(512);
    for id in ["", " ", "left,right", boundary.as_str()] {
        let output = run_compatibility_ids(&[id], true);
        assert_eq!(output.status.code(), Some(1), "id length={}", id.len());
        assert!(output.stderr.is_empty());
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["items"][0]["input"], id);
        assert_eq!(value["items"][0]["status"], "error");
    }

    let overlong = "x".repeat(513);
    let output = run_compatibility_ids(&[&overlong], true);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    let error: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(error["error"]["code"], "invalid_argument");
    assert!(
        error["error"]["message"]
            .as_str()
            .unwrap()
            .contains("limited to 512 UTF-8 bytes")
    );
}

#[test]
fn canonical_normalization_and_byte_boundary_are_route_specific() {
    let boundary = "x".repeat(512);
    let canonical = run_canonical_ids(&format!(" ,{boundary},, "), true);
    let compatibility = run_compatibility_ids(&[&boundary], true);
    assert_eq!(canonical.status, compatibility.status);
    assert_eq!(canonical.stdout, compatibility.stdout);
    assert_eq!(canonical.stderr, compatibility.stderr);

    for empty in [",,", " , , "] {
        let output = run_canonical_ids(empty, true);
        assert_eq!(output.status.code(), Some(2));
        let error: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(error["error"]["code"], "invalid_argument");
    }
    let overlong = format!("{}x", "é".repeat(256));
    let output = run_canonical_ids(&overlong, true);
    assert_eq!(output.status.code(), Some(2));
    let error: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(error["error"]["code"], "invalid_argument");
}

#[test]
fn rejected_batch_matrix_never_reaches_real_cli_client_cache_fs_or_item_work() {
    let fixture = provision_article_fulltext_fixture(env!("CARGO_MANIFEST_DIR"))
        .expect("provision article fixture");
    assert!(!fixture.cache_dir.exists());
    let request_bytes_before = std::fs::read(&fixture.request_log).expect("fixture request log");
    let comma_ids = |count: usize| vec!["1"; count].join(",");
    let positional_ids = |count: usize| vec!["1".to_string(); count];
    let ids_11 = comma_ids(11);
    let ids_21 = comma_ids(21);
    let mut cases = vec![
        (
            vec!["--json", "batch", "article", ",,"],
            "Batch IDs are required",
        ),
        (
            vec!["--json", "batch", "article", ids_11.as_str()],
            "Batch is limited to 10 IDs",
        ),
        (
            vec![
                "--json",
                "batch",
                "article",
                ids_21.as_str(),
                "--mode",
                "compact",
            ],
            "Batch is limited to 20 IDs",
        ),
        (
            vec!["--json", "batch", "article", "1", "--source", "nci"],
            "--source is only supported for trial batches",
        ),
        (
            vec!["--json", "batch", "trial", "NCT1", "--source", "unknown"],
            "Unknown --source 'unknown'",
        ),
        (
            vec!["--json", "batch", "gene", "BRAF", "--mode", "detail"],
            "--mode is only supported for article batches",
        ),
        (
            vec![
                "--json",
                "batch",
                "article",
                "1",
                "--mode",
                "compact",
                "--sections",
                "tldr",
            ],
            "--sections is not supported for compact article batches",
        ),
        (
            vec!["--json", "batch", "article", "1", "--sections", ""],
            "comma-separated list of nonempty section names",
        ),
        (
            vec![
                "--json",
                "batch",
                "article",
                "1",
                "--sections",
                "tldr,,annotations",
            ],
            "comma-separated list of nonempty section names",
        ),
        (
            vec!["--json", "batch", "article", "1", "--sections", "unknown"],
            "Unknown section",
        ),
        (
            vec!["--json", "batch", "unknown", "1"],
            "Unknown batch entity",
        ),
        (
            vec![
                "--json",
                "batch",
                "adverse-event",
                "1",
                "--sections",
                "tldr",
            ],
            "sections are not supported for adverse-event",
        ),
        (
            vec!["--json", "batch", "article", "1", "--offset", "1"],
            "unexpected argument '--offset'",
        ),
        (
            vec!["--json", "article", "batch"],
            "required arguments were not provided",
        ),
    ]
    .into_iter()
    .map(|(args, expected)| {
        (
            args.into_iter().map(str::to_string).collect::<Vec<_>>(),
            expected,
        )
    })
    .collect::<Vec<_>>();
    cases.push((
        vec![
            "--json".into(),
            "batch".into(),
            "article".into(),
            "x".repeat(513),
        ],
        "limited to 512 UTF-8 bytes",
    ));
    let mut compat_21 = vec!["--json".into(), "article".into(), "batch".into()];
    compat_21.extend(positional_ids(21));
    cases.push((compat_21, "Article batch is limited to 20 IDs"));
    cases.push((
        vec![
            "--json".into(),
            "article".into(),
            "batch".into(),
            "x".repeat(513),
        ],
        "limited to 512 UTF-8 bytes",
    ));

    for (args, expected) in cases {
        let output = run_preflight_sentinel(&fixture, &args);
        assert_eq!(output.status.code(), Some(2), "args={args:?}");
        assert!(output.stderr.is_empty(), "args={args:?}");
        assert!(
            String::from_utf8_lossy(&output.stdout).contains(expected),
            "args={args:?}, stdout={}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert!(
            !fixture.cache_dir.exists(),
            "invalid dispatch touched cache/fs for {args:?}"
        );
        assert_eq!(
            std::fs::read(&fixture.request_log).expect("fixture request log"),
            request_bytes_before,
            "invalid dispatch reached a provider request for {args:?}"
        );
    }
}

//! Original-byte identity adoption and narrow source-only enrichment.
use super::GenomicPosField;
use crate::error::{BioMcpError, SourceContext, SourceProvider};
use biodata::{
    MyGenePage, MyGeneProfile, MyGeneRow, MyGeneRowDisposition, parse_mygene_query,
    select_mygene_get,
};
use serde::Deserialize;
use std::sync::Arc;

/// Custody remains intact through product assembly, including rejected dispositions.
#[derive(Debug, Clone)]
pub struct MyGeneRecord {
    page: Arc<MyGenePage>,
    index: usize,
    pub enrichment: MyGeneEnrichment,
    pub conversion: GeneConversionReport,
}

/// Decodes only fields outside shared identity ownership.
#[derive(Debug, Clone, Deserialize)]
pub struct MyGeneEnrichment {
    pub summary: Option<String>,
    pub type_of_gene: Option<String>,
    pub genomic_pos: Option<GenomicPosField>,
    #[serde(rename = "MIM")]
    pub mim: Option<serde_json::Value>,
    pub uniprot: Option<serde_json::Value>,
    pub pathway: Option<serde_json::Value>,
    // Shape validation and source custody retain transcript/protein members.
    // dead-code reason: get transcript and protein shapes retain source-only validation
    #[allow(dead_code)]
    pub ensembl: Option<EnsemblMembers>,
}

#[derive(Deserialize)]
struct SearchEnrichment {
    type_of_gene: Option<String>,
    genomic_pos: Option<GenomicPosField>,
    #[serde(rename = "MIM")]
    mim: Option<serde_json::Value>,
    uniprot: Option<serde_json::Value>,
}

// These retained source-only members enforce the existing get shape contract.
// dead-code reason: get transcript and protein shapes retain source-only validation
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct EnsemblMember {
    pub protein: Option<Vec<String>>,
    pub transcript: Option<Vec<String>>,
}
// dead-code reason: get transcript and protein shapes retain source-only validation
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum EnsemblMembers {
    Single(EnsemblMember),
    Multiple(Vec<EnsemblMember>),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GeneConversionReport {
    pub losses: Vec<(&'static str, &'static str)>,
    pub ensembl_display: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MyGeneSearchResponse {
    pub total: usize,
    pub hits: Vec<MyGeneRecord>,
    // Kept through assembly for digest, total and disposition custody.
    // dead-code reason: the acquired page retains digest, totals and row dispositions until assembly
    #[allow(dead_code)]
    pub page: Arc<MyGenePage>,
}

pub(crate) fn failure(message: impl Into<String>) -> BioMcpError {
    BioMcpError::Api {
        api: "MyGene.info".into(),
        message: message.into(),
    }
    .with_source_context(SourceContext::retry(SourceProvider::MYGENE))
}
fn total(page: &MyGenePage) -> Result<usize, BioMcpError> {
    page.total()
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| failure("MyGene response requires a representable total"))
}

impl MyGeneRecord {
    pub fn row(&self) -> &MyGeneRow {
        match &self.page.rows()[self.index] {
            MyGeneRowDisposition::Projected(row) => row,
            MyGeneRowDisposition::Rejected { .. } => unreachable!("only accepted rows convert"),
        }
    }
    // Consumer proof inspects the acquired page; product keeps it until assembly.
    // dead-code reason: consumer controls inspect exact-byte page custody
    #[allow(dead_code)]
    pub fn page(&self) -> &MyGenePage {
        &self.page
    }
    pub fn symbol(&self) -> Option<&str> {
        self.row().identity().symbol()
    }
    pub fn name(&self) -> Option<&str> {
        self.row().identity().name()
    }
    pub fn code(&self, namespace: &str) -> Option<&str> {
        self.row()
            .identity()
            .codes()
            .iter()
            .find(|code| code.namespace().as_str() == namespace)
            .map(|code| code.value())
    }
    pub fn aliases(&self) -> Vec<String> {
        self.row()
            .identity()
            .aliases()
            .iter()
            .map(|term| term.text().to_owned())
            .collect()
    }
    pub(crate) fn hgnc_ids(&self) -> Result<Vec<String>, ()> {
        let mut ids = Vec::new();
        for code in self
            .row()
            .identity()
            .codes()
            .iter()
            .filter(|code| code.namespace().as_str() == "HGNC")
        {
            let canonical = canonical_hgnc(code.value())?;
            if !ids.contains(&canonical) {
                ids.push(canonical);
            }
        }
        Ok(ids)
    }
}

fn canonical_hgnc(value: &str) -> Result<String, ()> {
    let value = value.trim();
    let digits = if value
        .get(..5)
        .is_some_and(|p| p.eq_ignore_ascii_case("HGNC:"))
    {
        &value[5..]
    } else {
        value
    };
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(());
    }
    let number = digits
        .parse::<u32>()
        .ok()
        .filter(|number| *number > 0)
        .ok_or(())?;
    Ok(format!("HGNC:{number}"))
}

fn convert(
    page: Arc<MyGenePage>,
    index: usize,
    profile: MyGeneProfile,
) -> Result<MyGeneRecord, BioMcpError> {
    let MyGeneRowDisposition::Projected(row) = &page.rows()[index] else {
        return Err(failure("MyGene response failed identity conversion"));
    };
    let enrichment = match profile {
        MyGeneProfile::Get => serde_json::from_str(row.source().raw())
            .map_err(|_| failure("MyGene source-only enrichment shape failed"))?,
        MyGeneProfile::Search => {
            let source: SearchEnrichment = serde_json::from_str(row.source().raw())
                .map_err(|_| failure("MyGene source-only enrichment shape failed"))?;
            MyGeneEnrichment {
                summary: None,
                type_of_gene: source.type_of_gene,
                genomic_pos: source.genomic_pos,
                mim: source.mim,
                uniprot: source.uniprot,
                pathway: None,
                ensembl: None,
            }
        }
    };
    // Locate the first source member only after validation. It cannot add a code.
    let raw: serde_json::Value = serde_json::from_str(row.source().raw())
        .map_err(|_| failure("MyGene source row shape failed"))?;
    let first = raw
        .get("ensembl")
        .and_then(|value| match value {
            serde_json::Value::Array(values) => values.first(),
            value => Some(value),
        })
        .and_then(|member| member.get("gene"));
    let first_text = first.and_then(|value| match value {
        serde_json::Value::String(text) => Some(text.as_str()),
        serde_json::Value::Array(values) => values.first().and_then(|value| value.as_str()),
        _ => None,
    });
    let ensembl_display = first_text
        .filter(|text| !text.trim().is_empty())
        .and_then(|text| {
            row.identity()
                .codes()
                .iter()
                .find(|code| code.namespace().as_str() == "Ensembl gene" && code.value() == text)
        })
        .map(|code| code.value().to_owned());
    let mut conversion = GeneConversionReport {
        ensembl_display,
        losses: Vec::new(),
    };
    if first_text.is_some_and(|text| text.trim().is_empty()) {
        conversion
            .losses
            .push(("ensembl.gene", "blank source value omitted from display"));
    }
    if first.is_some_and(|value| value.as_array().is_some_and(Vec::is_empty)) {
        conversion.losses.push((
            "ensembl.gene",
            "empty first gene vector omitted from display",
        ));
    }
    for code in row.identity().codes() {
        if code.namespace().as_str() == "Ensembl gene"
            && Some(code.value()) != conversion.ensembl_display.as_deref()
        {
            conversion
                .losses
                .push(("ensembl.gene", "source claim omitted from singular display"));
        }
        if code.namespace().as_str() == "HGNC"
            && canonical_hgnc(code.value()).as_deref() != Ok(code.value())
        {
            conversion.losses.push((
                "HGNC",
                "source lexical form changed or inconclusive for join",
            ));
        }
    }
    Ok(MyGeneRecord {
        page,
        index,
        enrichment,
        conversion,
    })
}

pub(crate) fn decode_get(bytes: &[u8], symbol: &str) -> Result<MyGeneRecord, BioMcpError> {
    let page = Arc::new(
        parse_mygene_query(bytes, MyGeneProfile::Get)
            .map_err(|error| failure(error.to_string()))?,
    );
    let selected = select_mygene_get(&page, symbol).map_err(|error| failure(error.to_string()))?;
    total(&page)?;
    let index =
        selected
            .map(|row| row.source().ordinal())
            .ok_or_else(|| BioMcpError::NotFound {
                entity: "gene".into(),
                id: symbol.trim().into(),
                suggestion: format!("Try searching: biomcp search gene -q {}", symbol.trim()),
            })?;
    convert(page, index, MyGeneProfile::Get)
}

pub(crate) fn decode_search(bytes: &[u8]) -> Result<MyGeneSearchResponse, BioMcpError> {
    let page = Arc::new(
        parse_mygene_query(bytes, MyGeneProfile::Search)
            .map_err(|error| failure(error.to_string()))?,
    );
    for disposition in page.rows() {
        if let MyGeneRowDisposition::Rejected { error, .. } = disposition {
            return Err(failure(error.to_string()));
        }
    }
    let total = total(&page)?;
    let hits = (0..page.rows().len())
        .map(|index| convert(Arc::clone(&page), index, MyGeneProfile::Search))
        .collect::<Result<_, _>>()?;
    Ok(MyGeneSearchResponse { total, hits, page })
}

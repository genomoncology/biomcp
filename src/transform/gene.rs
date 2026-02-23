use crate::entities::gene::{Gene, GenePathway, GeneSearchResult};
use crate::sources::mygene::{MyGeneGetResponse, MyGeneHit};

fn normalize_summary(summary: Option<String>) -> Option<String> {
    let summary = summary?;
    let normalized_ws = summary.split_whitespace().collect::<Vec<_>>().join(" ");
    let normalized_ws = normalized_ws.trim();
    if normalized_ws.is_empty() {
        return None;
    }

    Some(first_two_sentences(normalized_ws).to_string())
}

fn first_two_sentences(value: &str) -> &str {
    let mut boundaries = 0usize;
    for (idx, ch) in value.char_indices() {
        if !matches!(ch, '.' | '!' | '?') {
            continue;
        }
        let after = idx + ch.len_utf8();
        let sentence_boundary = after == value.len()
            || value[after..]
                .chars()
                .next()
                .is_some_and(|c| c.is_whitespace());
        if sentence_boundary {
            boundaries += 1;
        }

        if boundaries >= 2 {
            return value[..after].trim_end();
        }
    }

    value.trim()
}

fn drop_lowercase_aliases(aliases: Vec<String>) -> Vec<String> {
    aliases
        .into_iter()
        .filter(|alias| !alias.chars().any(|c| c.is_ascii_lowercase()))
        .collect()
}

fn drop_trailing_hyphen_number_aliases(aliases: Vec<String>) -> Vec<String> {
    aliases
        .into_iter()
        .filter(|alias| {
            let mut parts = alias.rsplitn(2, '-');
            let suffix = parts.next().unwrap_or("");
            let has_dash = parts.next().is_some();
            !(has_dash && !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()))
        })
        .collect()
}

fn normalize_aliases(mut aliases: Vec<String>) -> Vec<String> {
    aliases = drop_lowercase_aliases(aliases);
    aliases = drop_trailing_hyphen_number_aliases(aliases);
    aliases.truncate(5);
    aliases
}

fn first_string(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(s) => Some(s.trim().to_string()),
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::Array(arr) => arr.iter().find_map(first_string),
        serde_json::Value::Object(map) => map.values().find_map(first_string),
        _ => None,
    }
    .filter(|s| !s.is_empty())
}

fn extract_uniprot_id(value: Option<&serde_json::Value>) -> Option<String> {
    let value = value?;
    let obj = value.as_object()?;
    let swiss = obj.get("Swiss-Prot")?;
    first_string(swiss)
}

fn extract_omim_id(value: Option<&serde_json::Value>) -> Option<String> {
    value.and_then(first_string)
}

fn extract_kegg_pathways(value: Option<&serde_json::Value>) -> Option<Vec<GenePathway>> {
    let pathway = value?;
    let pathway = pathway.as_object()?;
    let kegg = pathway.get("kegg")?;

    let mut out: Vec<GenePathway> = Vec::new();
    let push_item = |item: &serde_json::Value, out: &mut Vec<GenePathway>| {
        let Some(obj) = item.as_object() else { return };
        let Some(id) = obj
            .get("id")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
        else {
            return;
        };
        let Some(name) = obj
            .get("name")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
        else {
            return;
        };
        out.push(GenePathway {
            id: id.to_string(),
            name: name.to_string(),
        });
    };

    match kegg {
        serde_json::Value::Object(_) => push_item(kegg, &mut out),
        serde_json::Value::Array(arr) => {
            for item in arr {
                push_item(item, &mut out);
                if out.len() >= 20 {
                    break;
                }
            }
        }
        _ => {}
    }

    if out.is_empty() { None } else { Some(out) }
}

fn format_genomic_coordinates(resp: &MyGeneGetResponse) -> Option<String> {
    let pos = resp.genomic_pos.as_ref()?;
    let chr = pos.chr()?.trim();
    let start = pos.start()?;
    let end = pos.end()?;
    let strand = pos.strand()?;
    if chr.is_empty() {
        return None;
    }
    Some(format!("{chr}:{start}-{end} (strand: {strand})"))
}

pub fn from_mygene_get(resp: MyGeneGetResponse) -> Gene {
    let genomic_coordinates = format_genomic_coordinates(&resp);
    let omim_id = extract_omim_id(resp.mim.as_ref());
    let uniprot_id = extract_uniprot_id(resp.uniprot.as_ref());
    let pathways = extract_kegg_pathways(resp.pathway.as_ref());
    let aliases = normalize_aliases(resp.alias.into_vec());

    Gene {
        symbol: resp.symbol.unwrap_or_default(),
        name: resp.name.unwrap_or_default(),
        entrez_id: resp
            .entrezgene
            .as_ref()
            .map(|n| n.as_string())
            .unwrap_or_default(),
        ensembl_id: resp.ensembl.as_ref().and_then(|e| e.gene()).cloned(),
        location: resp.genomic_pos.as_ref().and_then(|g| g.chr()).cloned(),
        genomic_coordinates,
        omim_id,
        uniprot_id,
        summary: normalize_summary(resp.summary),
        gene_type: resp.type_of_gene,
        aliases,
        clinical_diseases: Vec::new(),
        clinical_drugs: Vec::new(),
        pathways,
        ontology: None,
        diseases: None,
        protein: None,
        go: None,
        interactions: None,
        civic: None,
    }
}

pub fn from_mygene_hit(hit: &MyGeneHit) -> GeneSearchResult {
    let genomic_coordinates = hit.genomic_pos.as_ref().and_then(|pos| {
        let chr = pos.chr()?.trim();
        let start = pos.start()?;
        let end = pos.end()?;
        if chr.is_empty() {
            return None;
        }
        Some(format!("{chr}:{start}-{end}"))
    });

    GeneSearchResult {
        symbol: hit.symbol.clone().unwrap_or_default(),
        name: hit.name.clone().unwrap_or_default(),
        entrez_id: hit
            .entrezgene
            .as_ref()
            .map(|n| n.as_string())
            .unwrap_or_default(),
        genomic_coordinates,
        uniprot_id: extract_uniprot_id(hit.uniprot.as_ref()),
        omim_id: extract_omim_id(hit.mim.as_ref()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::serde::StringOrVec;

    #[test]
    fn normalize_summary_keeps_summary() {
        assert_eq!(
            normalize_summary(Some("Sentence one. Sentence two.".into())),
            Some("Sentence one. Sentence two.".into())
        );
        assert_eq!(
            normalize_summary(Some("Sentence one. Sentence two. Sentence three.".into())),
            Some("Sentence one. Sentence two.".into())
        );
        assert_eq!(normalize_summary(Some("".into())), None);
        assert_eq!(normalize_summary(Some("   ".into())), None);
    }

    #[test]
    fn normalize_summary_preserves_utf8_without_ellipsis() {
        let normalized = normalize_summary(Some("β-catenin regulates growth.".into()))
            .expect("expected summary");
        assert_eq!(normalized, "β-catenin regulates growth.");
        assert!(!normalized.contains("..."));
    }

    #[test]
    fn normalize_aliases_drops_lowercase_and_trailing_number_hyphen() {
        let aliases = normalize_aliases(vec![
            "B-RAF1".to_string(),
            "B-raf".to_string(),
            "BRAF-1".to_string(),
            "BRAF1".to_string(),
            "NS7".to_string(),
            "RAFB1".to_string(),
        ]);
        assert_eq!(aliases, vec!["B-RAF1", "BRAF1", "NS7", "RAFB1"]);
    }

    #[test]
    fn string_or_vec_into_vec() {
        assert_eq!(StringOrVec::None.into_vec(), Vec::<String>::new());
        assert_eq!(StringOrVec::Single("X".into()).into_vec(), vec!["X"]);
        assert_eq!(
            StringOrVec::Multiple(vec!["A".into(), "B".into()]).into_vec(),
            vec!["A", "B"]
        );
    }

    #[test]
    fn extract_kegg_pathways_handles_array() {
        let value = serde_json::json!({
            "kegg": [
                {"id": "hsa04010", "name": "MAPK signaling"},
                {"id": "hsa04650", "name": "NK signaling"}
            ]
        });
        let pathways = extract_kegg_pathways(Some(&value)).expect("pathways");
        assert_eq!(pathways.len(), 2);
        assert_eq!(pathways[0].id, "hsa04010");
    }

    #[test]
    fn gene_sections_maps_egfr_fields() {
        let resp: MyGeneGetResponse = serde_json::from_value(serde_json::json!({
            "symbol": "EGFR",
            "name": "epidermal growth factor receptor",
            "entrezgene": 1956,
            "summary": "Receptor tyrosine kinase.",
            "type_of_gene": "protein-coding",
            "ensembl": {"gene": "ENSG00000146648"},
            "genomic_pos": {"chr": "7", "start": 55086714, "end": 55275875, "strand": 1},
            "MIM": "131550",
            "uniprot": {"Swiss-Prot": "P00533"},
            "pathway": {"kegg": [{"id": "hsa04012", "name": "ErbB signaling pathway"}]}
        }))
        .expect("valid MyGene response");

        let gene = from_mygene_get(resp);
        assert_eq!(gene.symbol, "EGFR");
        assert_eq!(gene.entrez_id, "1956");
        assert_eq!(gene.uniprot_id.as_deref(), Some("P00533"));
        assert_eq!(gene.location.as_deref(), Some("7"));
    }

    #[test]
    fn gene_sections_maps_brca1_fields() {
        let resp: MyGeneGetResponse = serde_json::from_value(serde_json::json!({
            "symbol": "BRCA1",
            "name": "BRCA1 DNA repair associated",
            "entrezgene": 672,
            "summary": "Tumor suppressor and DNA repair gene.",
            "type_of_gene": "protein-coding",
            "ensembl": {"gene": "ENSG00000012048"},
            "genomic_pos": {"chr": "17", "start": 43044295, "end": 43125482, "strand": -1},
            "MIM": "113705",
            "uniprot": {"Swiss-Prot": "P38398"}
        }))
        .expect("valid MyGene response");

        let gene = from_mygene_get(resp);
        assert_eq!(gene.symbol, "BRCA1");
        assert_eq!(gene.ensembl_id.as_deref(), Some("ENSG00000012048"));
        assert_eq!(gene.omim_id.as_deref(), Some("113705"));
        assert_eq!(gene.uniprot_id.as_deref(), Some("P38398"));
    }

    #[test]
    fn gene_sections_maps_tp53_fields() {
        let resp: MyGeneGetResponse = serde_json::from_value(serde_json::json!({
            "symbol": "TP53",
            "name": "tumor protein p53",
            "entrezgene": 7157,
            "summary": "Acts as a tumor suppressor.",
            "type_of_gene": "protein-coding",
            "alias": ["P53", "BCC7", "Trp53", "LFS1"],
            "ensembl": {"gene": "ENSG00000141510"},
            "genomic_pos": {"chr": "17", "start": 7661779, "end": 7687550, "strand": -1},
            "uniprot": {"Swiss-Prot": "P04637"}
        }))
        .expect("valid MyGene response");

        let gene = from_mygene_get(resp);
        assert_eq!(gene.symbol, "TP53");
        assert_eq!(gene.aliases, vec!["P53", "BCC7", "LFS1"]);
        assert_eq!(gene.uniprot_id.as_deref(), Some("P04637"));
    }
}

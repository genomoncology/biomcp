#![deny(clippy::dbg_macro)]
#![deny(clippy::print_stderr)]
#![deny(clippy::print_stdout)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]

pub mod cli;
pub mod error;
mod gene;
pub mod mcp;
mod next_command;

#[cfg_attr(not(test), allow(dead_code))]
mod cache;
mod entities;
mod render;
mod skill_assets;
mod sources;
#[cfg(test)]
pub(crate) mod test_support;
mod transform;
mod utils;
pub(crate) mod workflow_ladders;

#[cfg(test)]
#[tokio::test]
async fn augment_genes_with_opentargets_merges_sources_without_duplicates() {
    crate::entities::disease::tests::proof_augment_genes_with_opentargets_merges_sources_without_duplicates().await;
}

#[cfg(test)]
#[tokio::test]
async fn augment_genes_with_opentargets_respects_twenty_gene_cap() {
    crate::entities::disease::tests::proof_augment_genes_with_opentargets_respects_twenty_gene_cap(
    )
    .await;
}

#[cfg(test)]
#[tokio::test]
async fn enrich_sparse_disease_identity_prefers_exact_ols4_match() {
    crate::entities::disease::tests::proof_enrich_sparse_disease_identity_prefers_exact_ols4_match(
    )
    .await;
}

#[cfg(test)]
#[test]
fn disease_markdown_renders_ot_only_gene_association_table() {
    crate::render::markdown::tests::proof_disease_markdown_renders_ot_only_gene_association_table();
}

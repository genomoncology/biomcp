//! Exact molecule synonyms used to resolve drug development codes.

use serde::Deserialize;

use super::ChemblClient;
use crate::error::BioMcpError;
use crate::sources::RequestPlan;

#[derive(Debug, Deserialize)]
pub(crate) struct ChemblMoleculeIdentity {
    pub molecule_chembl_id: String,
    pub pref_name: String,
    #[serde(default)]
    molecule_synonyms: Vec<MoleculeSynonym>,
}

#[derive(Debug, Deserialize)]
struct MoleculeSynonym {
    molecule_synonym: String,
}

#[derive(Deserialize)]
struct MoleculePage {
    molecules: Vec<ChemblMoleculeIdentity>,
    page_meta: MoleculePageMeta,
}

#[derive(Deserialize)]
struct MoleculePageMeta {
    total_count: usize,
    next: Option<String>,
}

impl ChemblClient {
    /// A complete exact-synonym page must identify one molecule. The returned
    /// row itself must carry the synonym; a server-side text match is no proof.
    pub(crate) async fn molecule_by_exact_synonym(
        &self,
        synonym: &str,
    ) -> Result<Option<ChemblMoleculeIdentity>, BioMcpError> {
        let plan = RequestPlan::get("molecule.json")
            .query("molecule_synonyms__molecule_synonym__iexact", synonym)
            .query("limit", "100");
        let page: MoleculePage = self.get_plan(&plan).await?;
        if page.page_meta.next.is_some() || page.page_meta.total_count != page.molecules.len() {
            return Err(BioMcpError::Api {
                api: "chembl".into(),
                message: "ChEMBL returned an incomplete exact molecule synonym page; drug identity cannot be confirmed.".into(),
            });
        }
        let mut matches = page.molecules.into_iter().filter(|molecule| {
            molecule.molecule_synonyms.iter().any(|row| {
                row.molecule_synonym
                    .trim()
                    .eq_ignore_ascii_case(synonym.trim())
            })
        });
        let Some(first) = matches.next() else {
            return Ok(None);
        };
        if matches.any(|row| {
            row.molecule_chembl_id != first.molecule_chembl_id
                || !row.pref_name.eq_ignore_ascii_case(&first.pref_name)
        }) {
            return Err(BioMcpError::InvalidArgument(format!(
                "Ambiguous drug development code \"{synonym}\": ChEMBL lists several matching molecules. Use a preferred drug name."
            )));
        }
        if first.pref_name.trim().is_empty() || first.molecule_chembl_id.trim().is_empty() {
            return Ok(None);
        }
        Ok(Some(first))
    }
}

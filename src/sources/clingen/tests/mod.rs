mod acquisition;
mod parsing;

mod construction {
    //! Tier 2 — request construction. Pure: builds request plans and asserts the
    //! exact method/path/query that would be sent.

    use super::super::*;
    use crate::error::BioMcpError;
    use crate::sources::HttpMethod;

    #[test]
    fn clingen_plans_set_lookup_and_download_paths() {
        let lookup = ClinGenClient::gene_lookup_plan(" braf ").unwrap();
        assert_eq!(lookup.method, HttpMethod::Get);
        assert_eq!(lookup.path, "api/genes/look/BRAF");
        assert!(lookup.query.is_empty());

        let validity = ClinGenClient::validity_download_plan();
        assert_eq!(validity.method, HttpMethod::Get);
        assert_eq!(validity.path, "kb/gene-validity/download");

        let dosage = ClinGenClient::dosage_download_plan();
        assert_eq!(dosage.method, HttpMethod::Get);
        assert_eq!(dosage.path, "kb/gene-dosage/download");
    }

    #[test]
    fn lookup_plan_rejects_invalid_gene_symbols() {
        for gene in ["", "BR AF", "BRAF/ALK"] {
            assert!(
                matches!(
                    ClinGenClient::gene_lookup_plan(gene),
                    Err(BioMcpError::InvalidArgument(_))
                ),
                "expected invalid argument for {gene:?}"
            );
        }
    }
}

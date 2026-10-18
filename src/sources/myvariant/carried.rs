//! Apply source policy while decoding carried annotations, before any caller
//! can use them for display, search ranking, fallback, or enrichment.

use serde::{Deserialize, Deserializer};

macro_rules! carried_field {
    ($name:ident, $id:literal) => {
        pub(super) fn $name<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
        where
            D: Deserializer<'de>,
            T: Deserialize<'de>,
        {
            let value = Option::<T>::deserialize(deserializer)?;
            Ok(if crate::sources::disable::is_disabled($id) {
                None
            } else {
                value
            })
        }
    };
}

carried_field!(de_clinvar, "clinvar");
carried_field!(de_civic, "civic");
carried_field!(de_gnomad, "gnomad");
carried_field!(de_gnomad_exome, "gnomad");
carried_field!(de_cosmic, "cosmic");
carried_field!(de_cgi, "cancer-genome-interpreter");
carried_field!(de_dbsnp, "dbsnp");

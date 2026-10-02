//! Finite source-reviewed authored stage/action/reason vocabulary.
pub(crate) fn known(stage: &str, action: &str, reason: &str) -> bool {
    matches!(
        (stage, action, reason),
        ("", "ascii_lowercase", "anchor display")
            | ("", "ascii_lowercase", "display")
            | ("", "ascii_lowercase", "product display")
            | ("", "ascii_lowercase", "salt-form name matches requested")
            | (
                "",
                "ascii_lowercase",
                "selected display source; original lexical claim retained"
            )
            | ("", "ascii_lowercase", "therapy query display")
            | (
                "",
                "ascii_lowercase_retry_label",
                "one different nonblank search name selects detail request"
            )
            | ("", "cap", "32 DDInter cap")
            | ("", "deduplicate", "case-insensitive synonym collision")
            | ("", "deduplicate", "equal synonym occurrence")
            | ("", "deduplicate", "query term already inserted")
            | (
                "",
                "discard_get_row",
                "unmatched row while a name match exists"
            )
            | (
                "",
                "discard_mechanism_filter",
                "Synthetic activator does not contain inhibitor"
            )
            | ("", "discard_search_row", "empty display")
            | (
                "",
                "discard_sparse_product",
                "ambiguous canonical discovery refuses successful sparse result"
            )
            | (
                "",
                "discard_target_filter",
                "GENEC does not match requested geneb"
            )
            | ("", "exclude", "combination hit is not the named anchor")
            | ("", "exclude_alias_row", "conflicting populated identifiers")
            | (
                "",
                "exclude_alias_row",
                "later UNII conflict despite earlier matching code"
            )
            | (
                "",
                "exclude_ema_field",
                "CheBI labels are excluded from EMA allowed identity fields"
            )
            | (
                "",
                "exclude_ema_field",
                "DrugBank synonyms are excluded from EMA allowed identity fields"
            )
            | (
                "",
                "exclude_ema_field",
                "GtoPdb labels are excluded from EMA allowed identity fields"
            )
            | (
                "",
                "exclude_ema_field",
                "UNII display labels are excluded from EMA allowed identity fields"
            )
            | (
                "",
                "exclude_ema_field",
                "later CheBI labels remain excluded from EMA allowed identity fields"
            )
            | (
                "",
                "exclude_ema_field",
                "later UNII display labels remain excluded from EMA allowed identity fields"
            )
            | ("", "fallback", "label canonical signal")
            | ("", "fallback", "one different nonblank search result")
            | ("", "fallback", "unique discover canonical")
            | (
                "",
                "include_match_candidate",
                "ChEMBL preferred name; lower display priority"
            )
            | (
                "",
                "include_match_candidate",
                "DrugBank name; lower display priority"
            )
            | (
                "",
                "include_match_candidate",
                "GtoPdb name; lower display priority"
            )
            | (
                "",
                "include_match_candidate",
                "first CheBI name; lower display priority"
            )
            | (
                "",
                "include_match_candidate",
                "first OpenFDA brand; lower display priority"
            )
            | (
                "",
                "include_match_candidate",
                "first OpenFDA generic; lower display priority"
            )
            | (
                "",
                "include_match_candidate",
                "first UNII display; lower display priority"
            )
            | ("", "insert_requested", "")
            | (
                "",
                "match_and_uppercase_requested_target",
                "trimmed case-insensitive target match; display requested target GENEB"
            )
            | (
                "",
                "normalize_approval_date",
                "YYYYMMDD normalizes to YYYY-MM-DD and named calendar display"
            )
            | (
                "",
                "normalize_molecule_type",
                "existing molecule-type map; identity adapter omits enrichment"
            )
            | ("", "normalize_to_empty", "edge dots")
            | (
                "",
                "omit_candidate_identifier",
                "search result establishes lookup label; final GET alone supplies product code"
            )
            | ("", "omit_card_keep_ddinter", "card three-brand cap")
            | ("", "omit_card_keep_ddinter", "three-card cap")
            | ("", "omit_code", "first ChEMBL code already selected")
            | ("", "omit_code", "first DrugBank code already selected")
            | ("", "omit_code", "first UNII code already selected")
            | (
                "",
                "omit_code",
                "first code already selected; conflict retained"
            )
            | (
                "",
                "omit_code",
                "first code already selected; equal value separate occurrence"
            )
            | ("", "omit_code", "first identifier already selected")
            | ("", "omit_display", "anchor already selected")
            | ("", "omit_display", "later UNII display")
            | ("", "omit_display", "lower-priority source")
            | (
                "",
                "omit_display_candidate",
                "OpenFDA generic precedes DrugBank name"
            )
            | (
                "",
                "omit_get_atc",
                "ATC remains source-only; Get profile does not request it and no merged Drug field consumes it"
            )
            | (
                "",
                "omit_initial_target",
                "first target is overridden by actual requested-target match"
            )
            | ("", "omit_match_candidate", "later CheBI name")
            | ("", "omit_match_candidate", "later NDC element")
            | (
                "",
                "omit_match_candidate",
                "later OpenFDA brand array member"
            )
            | ("", "omit_match_candidate", "later OpenFDA brand member")
            | ("", "omit_match_candidate", "later OpenFDA generic member")
            | ("", "omit_match_candidate", "later UNII display")
            | (
                "",
                "omit_synthesized_action",
                "explicit mechanism text present"
            )
            | (
                "",
                "omit_synthesized_target",
                "explicit mechanism text present; get target list is GtoPdb-owned"
            )
            | (
                "",
                "refuse_ambiguous_canonical",
                "competing Exact CanonicalId Drug with different normalized label"
            )
            | (
                "",
                "replace_lookup_display",
                "accepted sparse initial name is replaced by accepted final candidate"
            )
            | ("", "requested_name_fallback", "no source name")
            | ("", "requested_name_fallback", "no supplied name")
            | ("", "select", "EMA ChEMBL authority")
            | ("", "select", "EMA DrugBank authority")
            | ("", "select", "EMA NDC authority")
            | ("", "select", "EMA brand authority")
            | ("", "select", "EMA generic authority")
            | ("", "select", "OpenFDA generic display before brand")
            | ("", "select", "anchor card and DDInter")
            | ("", "select", "anchor synonym")
            | ("", "select", "first ChEMBL code")
            | ("", "select", "first DrugBank code")
            | ("", "select", "first DrugBank identifier")
            | ("", "select", "first UNII code")
            | ("", "select", "first code")
            | ("", "select", "first identifier")
            | ("", "select", "first named anchor")
            | ("", "select_and_ascii_lowercase", "display precedence")
            | (
                "",
                "select_and_ascii_lowercase",
                "first supplied NDC name and display precedence"
            )
            | (
                "",
                "select_approval_agency",
                "FDA agency makes date eligible"
            )
            | (
                "",
                "select_canonical_retry_label",
                "Exact and CanonicalId top Drug has no competing exact canonical Drug label"
            )
            | ("", "select_card_and_ddinter", "first anchor synonym")
            | (
                "",
                "select_card_and_ddinter",
                "req-100 differs from requested BrandTwo and canonicaldrug"
            )
            | (
                "",
                "select_card_and_ddinter",
                "second retained anchor synonym"
            )
            | ("", "select_card_and_ddinter", "within both caps")
            | (
                "",
                "select_card_and_ddinter_synonym",
                "anchor synonym distinct from requested and display"
            )
            | (
                "",
                "select_ddinter_omit_card",
                "card cap is three; DDInter cap is 32"
            )
            | (
                "",
                "select_ddinter_omit_card",
                "card cap three is already filled; DDInter cap 32"
            )
            | (
                "",
                "select_first_code",
                "first DrugBank identifier in selected lookup"
            )
            | ("", "select_first_code", "selected DrugBank identifier")
            | (
                "",
                "select_indication",
                "first unique trimmed indication name"
            )
            | (
                "",
                "select_interaction_description",
                "description owns interaction text"
            )
            | (
                "",
                "select_interaction_partner",
                "name owns interaction partner under existing field precedence"
            )
            | (
                "",
                "select_match_candidate",
                "first OpenFDA brand is admitted for matching"
            )
            | (
                "",
                "select_mechanism",
                "explicit mechanism takes precedence over synthesized action/target"
            )
            | (
                "",
                "select_mechanism_and_admit",
                "mechanism substring matches inhibitor; first mechanism supplies display"
            )
            | (
                "",
                "select_retry_label",
                "first generic label supplies different canonical lookup"
            )
            | ("", "select_target", "first unique trimmed GtoPdb symbol")
            | (
                "",
                "strip_moa_suffix",
                "pharmacologic class keeps text before [MoA]"
            )
            | ("", "trim_edge_dots_ascii_lowercase", "product display")
            | (
                "Search candidate projection",
                "ascii_lowercase",
                "SecondDrug becomes seconddrug before ranking and final slice"
            )
            | (
                "Search candidate projection",
                "ascii_lowercase_and_match_active_substance",
                "SampleDrug becomes sampledrug; exact DrugBank name matches active-substance tier"
            )
            | (
                "Search candidate projection",
                "select_first_code",
                "candidate identifier exists before final pagination"
            )
            | (
                "Search candidate projection",
                "select_first_code",
                "first DrugBank identifier populates selected ranked candidate"
            )
            | (
                "alias eligibility and insertion",
                "exclude",
                "free-base descriptor"
            )
            | (
                "alias eligibility and insertion",
                "exclude",
                "more than four words"
            )
            | (
                "alias eligibility and insertion",
                "omit_unvisited_alias_cap",
                "ALPHA and BrandTwo insert two provider aliases; ABT-100 inserts third; cap check stops at Beta before this candidate; no deduplication call"
            )
            | (
                "alias eligibility and insertion",
                "select",
                "OpenFDA brand priority"
            )
            | (
                "alias eligibility and insertion",
                "select",
                "OpenFDA brand priority and lexical tie-break"
            )
            | (
                "alias eligibility and insertion",
                "select",
                "eligible investigational code"
            )
            | (
                "alias eligibility and insertion",
                "trim_and_deduplicate",
                "lexical tie-break selects ALPHA"
            )
            | (
                "base identity contribution to alias resolution",
                "omit_display",
                "OpenFDA generic has precedence"
            )
            | (
                "base identity contribution to alias resolution",
                "select",
                "canonical display"
            )
            | (
                "base identity contribution to alias resolution",
                "select",
                "first identifier"
            )
            | (
                "base selection, merge and candidate extraction",
                "extract_candidate",
                "OpenFDA brands feed trial candidate extraction"
            )
            | (
                "base selection, merge and candidate extraction",
                "extract_candidate",
                "preserve spelling until alias sorting"
            )
            | (
                "base selection, merge and candidate extraction",
                "extract_candidate",
                "preserve whitespace until alias insertion"
            )
            | (
                "base selection, merge and candidate extraction",
                "omit_display",
                "OpenFDA generic precedes DrugBank name"
            )
            | (
                "base selection, merge and candidate extraction",
                "omit_requested_synonym",
                "equals requested name by ASCII lowercase; merge actually visits and skips it"
            )
            | (
                "base selection, merge and candidate extraction",
                "select_ddinter_omit_card",
                "fifth DDInter synonym; three card slots filled"
            )
            | (
                "base selection, merge and candidate extraction",
                "select_ddinter_omit_card",
                "fourth DDInter synonym; three card slots filled"
            )
            | (
                "base selection, merge and candidate extraction",
                "select_display_ascii_lowercase",
                "OpenFDA generic supplies normalized canonical display"
            )
            | (
                "base selection, merge and candidate extraction",
                "select_first_code",
                "first identifier"
            )
            | (
                "base selection, merge and candidate extraction",
                "select_synonym",
                "first card synonym; first DDInter synonym"
            )
            | (
                "base selection, merge and candidate extraction",
                "select_synonym",
                "second card synonym; second DDInter synonym"
            )
            | (
                "base selection, merge and candidate extraction",
                "select_synonym",
                "third card synonym; third DDInter synonym; alias eligibility is a later separate policy"
            )
            | (
                "candidate extraction",
                "extract_candidate",
                "all DrugBank synonyms are extracted in source order before alias eligibility/sorting/cap"
            )
            | (
                "final ranked slice",
                "omit_page",
                "limit after complete ranking"
            )
            | (
                "final ranked slice",
                "select",
                "active-substance tier before broad text"
            )
            | (
                "initial Get selection and merge",
                "ascii_lowercase",
                "no source name matches the 256-byte request; selection falls back to the sole hit; SampleDrug supplies sampledrug"
            )
            | (
                "initial Get selection and merge",
                "match_and_ascii_lowercase",
                "DrugBank name matches requested sampledrug and supplies normalized merged display"
            )
            | (
                "initial Get selection and merge",
                "select_first_code",
                "initial Get page selected row supplies merged DrugBank identifier"
            )
            | (
                "name deduplication",
                "deduplicate",
                "normalized sampledrug already inserted from ordinal 0; duplicate branch reached before limit 2"
            )
            | (
                "ranked deduplication",
                "tier_upgrade_keep_first_row",
                "later duplicate shares displayed name matchdrug; first row retained"
            )
            | (
                "ranked deduplication",
                "tier_upgrade_keep_first_row",
                "later product-name match for the same normalized displayed name matchdrug"
            )
            | (
                "ranked match classification",
                "select_alias_match",
                "DrugBank synonym equals normalized query"
            )
            | (
                "ranked match classification",
                "select_match_tier",
                "DrugBank name equals normalized query"
            )
            | (
                "ranked match classification",
                "select_match_tier",
                "OpenFDA brand equals normalized query and outranks active substance"
            )
            | (
                "ranked pagination",
                "omit_page",
                "limit after complete ranking"
            )
            | (
                "ranked pagination",
                "omit_page",
                "offset after complete ranking"
            )
            | (
                "result insertion",
                "select",
                "distinct normalized seconddrug accepted; requested limit 2 reached"
            )
            | (
                "result insertion",
                "select",
                "first normalized name inserted; requested limit reached"
            )
            | (
                "result insertion",
                "select",
                "first normalized sampledrug accepted"
            )
            | (
                "row projection",
                "ascii_lowercase",
                "SampleDrug becomes sampledrug before name deduplication and insertion"
            )
            | (
                "row projection",
                "ascii_lowercase",
                "source display normalized before name deduplication"
            )
            | (
                "row projection",
                "discard",
                "from_mychem_search_hit has no source name and returns None; branch reached before limit 2"
            )
            | (
                "row projection",
                "select_first_code",
                "first DrugBank identifier populates projected row before insertion"
            )
            | (
                "row projection",
                "select_first_code",
                "first DrugBank identifier selected in row projection before name deduplication"
            )
            | (
                "search projection",
                "ascii_lowercase",
                "OpenFDA brand precedes DrugBank name for display; original lexical claim retained"
            )
            | (
                "search projection",
                "ascii_lowercase",
                "selected display source; original lexical claim retained"
            )
            | (
                "search projection",
                "omit_display",
                "OpenFDA brand supplies higher priority displayed name; source claim retained"
            )
            | (
                "search projection",
                "select_first_code",
                "first DrugBank identifier in this projected search row"
            )
            | (
                "selection break",
                "omit_unvisited_limit",
                "loop breaks immediately after accepting ordinal 0 at limit 1; page admission precedes selection"
            )
            | (
                "selection break",
                "omit_unvisited_limit",
                "row never projected or deduplicated after limit break"
            )
    )
}

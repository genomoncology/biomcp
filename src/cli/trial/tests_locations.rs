//! Trial location pagination and query-summary tests.

use super::TrialGetArgs;
use super::dispatch::{
    LocationPaginationMeta, handle_get, paginate_trial_locations, parse_trial_location_paging,
    should_show_trial_zero_result_nickname_hint, trial_locations_json, trial_search_query_summary,
};

fn contact(
    level: &str,
    name: &str,
    facility: Option<&str>,
) -> crate::entities::trial::TrialContact {
    crate::entities::trial::TrialContact {
        level: level.to_string(),
        name: name.to_string(),
        role: Some("CONTACT".to_string()),
        phone: None,
        email: Some(format!(
            "{}@example.test",
            name.to_ascii_lowercase().replace(' ', "-")
        )),
        facility: facility.map(str::to_string),
        city: facility.map(|_| "Example City".to_string()),
        state: None,
        country: facility.map(|_| "United States".to_string()),
    }
}

fn location(index: usize) -> crate::entities::trial::TrialLocation {
    let facility = format!("Site {index:02}");
    let name = format!("Contact {index:02}");
    let email = format!(
        "{}@example.test",
        name.to_ascii_lowercase().replace(' ', "-")
    );
    crate::entities::trial::TrialLocation {
        facility: Some(facility),
        city: Some("Example City".to_string()),
        state: None,
        postal_code: None,
        country: Some("United States".to_string()),
        status: Some("RECRUITING".to_string()),
        contacts: vec![crate::entities::trial::TrialSiteContact {
            name: name.clone(),
            role: Some("CONTACT".to_string()),
            phone: None,
            email: Some(email.clone()),
        }],
        contact_name: Some(name),
        contact_role: Some("CONTACT".to_string()),
        contact_phone: None,
        contact_email: Some(email),
        latitude: None,
        longitude: None,
    }
}

fn paged_contact_trial() -> crate::entities::trial::Trial {
    let locations: Vec<_> = (0..25).map(location).collect();
    let mut contacts = vec![contact("central", "Central Coordinator", None)];
    contacts.extend((0..25).map(|index| {
        contact(
            "site",
            &format!("Contact {index:02}"),
            Some(&format!("Site {index:02}")),
        )
    }));
    crate::entities::trial::Trial {
        nct_id: "NCT00000001".to_string(),
        source: Some("ctgov".to_string()),
        title: "Example trial".to_string(),
        status: "Recruiting".to_string(),
        why_stopped: None,
        phase: None,
        study_type: None,
        age_range: None,
        conditions: vec![],
        interventions: vec![],
        intervention_details: vec![],
        sponsor: None,
        enrollment: None,
        summary: None,
        start_date: None,
        completion_date: None,
        eligibility_text: None,
        eligibility: None,
        eligibility_provenance: None,
        contacts: Some(contacts),
        locations: Some(locations),
        outcomes: None,
        arms: None,
        references: None,
    }
}

#[test]
fn paginate_trial_locations_aligns_site_contacts_to_the_page() {
    let mut trial = paged_contact_trial();

    let meta = paginate_trial_locations(&mut trial, 20, 3);

    assert_eq!(meta.total, 25);
    assert_eq!(meta.offset, 20);
    assert_eq!(meta.limit, 3);
    assert!(meta.has_more);
    assert_eq!(
        trial
            .locations
            .as_ref()
            .unwrap()
            .iter()
            .map(|location| location.facility.as_deref().unwrap())
            .collect::<Vec<_>>(),
        ["Site 20", "Site 21", "Site 22"]
    );
    assert_eq!(
        trial
            .contacts
            .as_ref()
            .unwrap()
            .iter()
            .map(|contact| contact.name.as_str())
            .collect::<Vec<_>>(),
        [
            "Central Coordinator",
            "Contact 20",
            "Contact 21",
            "Contact 22"
        ]
    );
}

#[test]
fn paginate_trial_locations_uses_counted_exact_membership_and_legacy_boundaries() {
    let duplicate = location(0);
    let mut second_duplicate = duplicate.clone();
    second_duplicate.status = Some("ACTIVE_NOT_RECRUITING".to_string());
    let mut optional = location(4);
    optional.facility = None;
    optional.city = None;
    optional.country = None;
    let mut legacy = location(5);
    legacy.contacts.clear();
    let mut blank_legacy = location(6);
    blank_legacy.contacts.clear();
    blank_legacy.contact_name = Some("   ".to_string());
    let mut authoritative = location(7);
    authoritative.contact_name = Some("Stale Alias".to_string());
    let mut omitted_same_place = location(8);
    omitted_same_place.facility = Some("Site 00".to_string());
    omitted_same_place.contacts[0].name = "Wrong Person".to_string();
    omitted_same_place.contacts[0].email = Some("wrong-person@example.test".to_string());
    omitted_same_place.contact_name = Some("Wrong Person".to_string());
    omitted_same_place.contact_email = Some("wrong-person@example.test".to_string());
    let locations = vec![
        duplicate,
        second_duplicate,
        optional.clone(),
        legacy.clone(),
        blank_legacy,
        authoritative.clone(),
        omitted_same_place,
    ];
    let mut trial = paged_contact_trial();
    trial.locations = Some(locations);
    let duplicate_contact = contact("site", "Contact 00", Some("Site 00"));
    let optional_contact = crate::entities::trial::TrialContact {
        level: "SITE".to_string(),
        facility: None,
        city: None,
        country: None,
        ..contact("site", "Contact 04", None)
    };
    let blank_legacy_contact = crate::entities::trial::TrialContact {
        name: "   ".to_string(),
        ..contact("site", "Contact 06", Some("Site 06"))
    };
    let stale_legacy_alias_contact = crate::entities::trial::TrialContact {
        name: "Stale Alias".to_string(),
        ..contact("site", "Contact 07", Some("Site 07"))
    };
    trial.contacts = Some(vec![
        duplicate_contact.clone(),
        contact("central", "Central", None),
        contact("site", "Wrong Person", Some("Site 00")),
        duplicate_contact.clone(),
        duplicate_contact,
        contact("future", "Unknown", None),
        optional_contact,
        contact("site", "Contact 05", Some("Site 05")),
        blank_legacy_contact,
        stale_legacy_alias_contact,
        contact("site", "Contact 07", Some("Site 07")),
    ]);

    paginate_trial_locations(&mut trial, 0, 6);

    assert!(
        !trial
            .contacts
            .as_ref()
            .unwrap()
            .iter()
            .any(|contact| contact.name == "   ")
    );
    assert!(
        !trial
            .contacts
            .as_ref()
            .unwrap()
            .iter()
            .any(|contact| contact.name == "Stale Alias")
    );
    assert_eq!(
        trial
            .contacts
            .as_ref()
            .unwrap()
            .iter()
            .map(|contact| contact.name.as_str())
            .collect::<Vec<_>>(),
        [
            "Contact 00",
            "Central",
            "Contact 00",
            "Unknown",
            "Contact 04",
            "Contact 05",
            "Contact 07"
        ]
    );
}

#[test]
fn paginate_trial_locations_empty_page_keeps_non_site_contacts_and_normalizes_empty() {
    let mut mixed = paged_contact_trial();
    mixed.contacts = Some(vec![
        contact("site", "Contact 00", Some("Site 00")),
        contact("central", "Central", None),
        contact("unknown", "Unknown", None),
    ]);
    paginate_trial_locations(&mut mixed, 99, 3);
    assert_eq!(
        mixed
            .contacts
            .as_ref()
            .unwrap()
            .iter()
            .map(|contact| contact.name.as_str())
            .collect::<Vec<_>>(),
        ["Central", "Unknown"]
    );

    let mut sites_only = paged_contact_trial();
    sites_only
        .contacts
        .as_mut()
        .unwrap()
        .retain(|contact| contact.level.eq_ignore_ascii_case("site"));
    paginate_trial_locations(&mut sites_only, 99, 3);
    assert!(sites_only.contacts.is_none());
}

#[test]
fn parse_trial_location_paging_extracts_offset_limit_flags() {
    let sections = vec![
        "locations".to_string(),
        "--offset".to_string(),
        "20".to_string(),
        "--limit=10".to_string(),
    ];
    let (cleaned, offset, limit) =
        parse_trial_location_paging(&sections).expect("valid pagination flags");
    assert_eq!(cleaned, vec!["locations".to_string()]);
    assert_eq!(offset, Some(20));
    assert_eq!(limit, Some(10));
}

#[tokio::test]
async fn handle_get_rejects_duplicate_declared_and_legacy_paging() {
    let err = handle_get(
        TrialGetArgs {
            nct_id: "NCT02576665".to_string(),
            sections: vec![
                "locations".to_string(),
                "--offset".to_string(),
                "20".to_string(),
            ],
            source: "ctgov".to_string(),
            offset: Some(10),
            limit: None,
        },
        false,
    )
    .await
    .expect_err("duplicate offset should fail fast");

    assert!(err.to_string().contains("--offset supplied twice"));
}

#[tokio::test]
async fn handle_get_rejects_declared_paging_without_locations() {
    let err = handle_get(
        TrialGetArgs {
            nct_id: "NCT02576665".to_string(),
            sections: vec!["eligibility".to_string()],
            source: "ctgov".to_string(),
            offset: Some(20),
            limit: None,
        },
        false,
    )
    .await
    .expect_err("location paging without locations should fail fast");

    assert!(
        err.to_string()
            .contains("--offset and --limit are only valid with the 'locations' section")
    );
}

#[tokio::test]
async fn handle_get_rejects_declared_limit_zero() {
    let err = handle_get(
        TrialGetArgs {
            nct_id: "NCT02576665".to_string(),
            sections: vec!["locations".to_string()],
            source: "ctgov".to_string(),
            offset: None,
            limit: Some(0),
        },
        false,
    )
    .await
    .expect_err("limit zero should fail fast");

    assert!(
        err.to_string()
            .contains("--limit must be >= 1 for trial location pagination")
    );
}

#[test]
fn parse_trial_location_paging_rejects_legacy_limit_zero() {
    let sections = vec![
        "locations".to_string(),
        "--limit".to_string(),
        "0".to_string(),
    ];
    let err = parse_trial_location_paging(&sections).expect_err("limit zero should fail");

    assert!(
        err.to_string()
            .contains("--limit must be >= 1 for trial location pagination")
    );
}

#[test]
fn trial_locations_json_preserves_location_pagination_and_section_sources() {
    let trial = crate::entities::trial::Trial {
        nct_id: "NCT00000001".to_string(),
        source: Some("ctgov".to_string()),
        title: "Example trial".to_string(),
        status: "Recruiting".to_string(),
        why_stopped: None,
        phase: Some("Phase 2".to_string()),
        study_type: Some("Interventional".to_string()),
        age_range: Some("18 Years and older".to_string()),
        conditions: vec!["melanoma".to_string()],
        interventions: vec!["osimertinib".to_string()],
        intervention_details: Vec::new(),
        sponsor: Some("Example Sponsor".to_string()),
        enrollment: Some(100),
        summary: Some("Example summary".to_string()),
        start_date: Some("2024-01-01".to_string()),
        completion_date: None,
        eligibility_text: None,
        eligibility: None,
        eligibility_provenance: None,
        contacts: None,
        locations: Some(vec![crate::entities::trial::TrialLocation {
            facility: Some("Example Hospital".to_string()),
            city: Some("Boston".to_string()),
            state: Some("MA".to_string()),
            postal_code: None,
            country: Some("United States".to_string()),
            status: Some("Recruiting".to_string()),
            contacts: Vec::new(),
            contact_name: None,
            contact_role: None,
            contact_phone: None,
            contact_email: None,
            latitude: None,
            longitude: None,
        }]),
        outcomes: None,
        arms: None,
        references: None,
    };

    let json = trial_locations_json(
        &trial,
        LocationPaginationMeta {
            total: 42,
            offset: 20,
            limit: 10,
            has_more: true,
        },
    )
    .expect("trial locations json");

    let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
    assert_eq!(value["nct_id"], "NCT00000001");
    assert_eq!(value["location_pagination"]["total"], 42);
    assert_eq!(value["location_pagination"]["offset"], 20);
    assert_eq!(value["location_pagination"]["limit"], 10);
    assert_eq!(value["location_pagination"]["has_more"], true);
    assert!(value.get("_meta").is_some());
    assert_eq!(value["_meta"]["section_sources"][0]["key"], "overview");
    assert_eq!(
        value["_meta"]["section_sources"][0]["sources"][0],
        "ClinicalTrials.gov"
    );
    assert!(
        value["_meta"]["section_sources"]
            .as_array()
            .expect("section sources array")
            .iter()
            .any(|entry| entry["key"] == "locations")
    );
}
#[test]
fn paginate_trial_locations_handles_missing_locations() {
    let mut trial = crate::entities::trial::Trial {
        nct_id: "NCT00000001".to_string(),
        source: Some("ctgov".to_string()),
        title: "Example trial".to_string(),
        status: "Recruiting".to_string(),
        why_stopped: None,
        phase: Some("Phase 2".to_string()),
        study_type: Some("Interventional".to_string()),
        age_range: Some("18 Years and older".to_string()),
        conditions: vec!["melanoma".to_string()],
        interventions: vec!["osimertinib".to_string()],
        intervention_details: Vec::new(),
        sponsor: Some("Example Sponsor".to_string()),
        enrollment: Some(100),
        summary: Some("Example summary".to_string()),
        start_date: Some("2024-01-01".to_string()),
        completion_date: None,
        eligibility_text: None,
        eligibility: None,
        eligibility_provenance: None,
        contacts: None,
        locations: None,
        outcomes: None,
        arms: None,
        references: None,
    };

    let meta = paginate_trial_locations(&mut trial, 20, 10);
    assert_eq!(meta.total, 0);
    assert_eq!(meta.offset, 20);
    assert_eq!(meta.limit, 10);
    assert!(!meta.has_more);
    assert!(trial.locations.is_some());
    assert_eq!(trial.locations.as_ref().map_or(usize::MAX, Vec::len), 0);
}
#[test]
fn trial_search_query_summary_includes_geo_filters() {
    let summary = trial_search_query_summary(
        &crate::entities::trial::TrialSearchFilters {
            condition: Some("melanoma".into()),
            facility: Some("MD Anderson".into()),
            age: Some(67.0),
            sex: Some("female".into()),
            criteria: Some("mismatch repair deficient".into()),
            sponsor_type: Some("nih".into()),
            lat: Some(40.7128),
            lon: Some(-74.006),
            distance: Some(50),
            ..Default::default()
        },
        None,
        0,
        None,
    );
    assert!(summary.contains("condition=melanoma"));
    assert!(summary.contains("facility=MD Anderson"));
    assert!(summary.contains("age=67"));
    assert!(summary.contains("sex=female"));
    assert!(summary.contains("criteria=mismatch repair deficient"));
    assert!(summary.contains("sponsor_type=nih"));
    assert!(summary.contains("lat=40.7128"));
    assert!(summary.contains("lon=-74.006"));
    assert!(summary.contains("distance=50"));
}

#[test]
fn trial_search_query_summary_includes_nci_source_marker() {
    let summary = trial_search_query_summary(
        &crate::entities::trial::TrialSearchFilters {
            condition: Some("melanoma".into()),
            source: crate::entities::trial::TrialSource::NciCts,
            ..Default::default()
        },
        None,
        0,
        None,
    );

    assert!(summary.contains("condition=melanoma"));
    assert!(summary.contains("source=nci"));
}

#[test]
fn trial_search_query_summary_includes_alias_opt_out_marker() {
    let summary = trial_search_query_summary(
        &crate::entities::trial::TrialSearchFilters {
            intervention: Some("daraxonrasib".into()),
            no_alias_expand: true,
            ..Default::default()
        },
        Some("daraxonrasib"),
        0,
        None,
    );

    assert!(summary.contains("intervention=daraxonrasib"));
    assert!(summary.contains("alias_expand=off"));
}

#[test]
fn trial_search_query_summary_omits_alias_opt_out_marker_when_not_applicable() {
    let no_intervention = trial_search_query_summary(
        &crate::entities::trial::TrialSearchFilters {
            condition: Some("melanoma".into()),
            no_alias_expand: true,
            ..Default::default()
        },
        None,
        0,
        None,
    );
    let nci = trial_search_query_summary(
        &crate::entities::trial::TrialSearchFilters {
            intervention: Some("daraxonrasib".into()),
            no_alias_expand: true,
            source: crate::entities::trial::TrialSource::NciCts,
            ..Default::default()
        },
        Some("daraxonrasib"),
        0,
        None,
    );

    assert!(!no_intervention.contains("alias_expand=off"));
    assert!(!nci.contains("alias_expand=off"));
}

#[test]
fn trial_search_query_summary_can_show_canonical_intervention() {
    let summary = trial_search_query_summary(
        &crate::entities::trial::TrialSearchFilters {
            intervention: Some("Keytruda".into()),
            ..Default::default()
        },
        Some("pembrolizumab"),
        0,
        None,
    );

    assert!(summary.contains("intervention=pembrolizumab"));
    assert!(!summary.contains("intervention=Keytruda"));
}

#[test]
fn trial_zero_result_nickname_hint_requires_positional_ctgov_query_with_zero_results() {
    use crate::entities::trial::TrialSource;

    assert!(should_show_trial_zero_result_nickname_hint(
        Some("CodeBreaK 300"),
        TrialSource::ClinicalTrialsGov,
        0
    ));
    assert!(!should_show_trial_zero_result_nickname_hint(
        None,
        TrialSource::ClinicalTrialsGov,
        0
    ));
    assert!(!should_show_trial_zero_result_nickname_hint(
        Some("CodeBreaK 300"),
        TrialSource::NciCts,
        0
    ));
    assert!(!should_show_trial_zero_result_nickname_hint(
        Some("CodeBreaK 300"),
        TrialSource::ClinicalTrialsGov,
        1
    ));
}

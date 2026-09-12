//! Ticket 1145 citation-evidence extractor tests.

use super::super::refs::JatsCitationExtraction;
use super::super::refs::JatsCitationTargetIds;
use super::super::refs::citation::JatsCitationPassage;
use super::super::refs::extract_citation_evidence;

fn target_ids(doi: Option<&str>, pmid: Option<&str>, pmcid: Option<&str>) -> JatsCitationTargetIds {
    JatsCitationTargetIds {
        doi: doi.map(str::to_string),
        pmid: pmid.map(str::to_string),
        pmcid: pmcid.map(str::to_string),
    }
}

fn article_with_body(body: &str) -> String {
    format!(
        r#"<article><front><article-meta><article-title>T</article-title></article-meta></front><body>{body}</body></article>"#
    )
}

fn body_with_refs(inner: &str, refs: &str) -> String {
    article_with_body(&format!(r#"{inner}<ref-list>{refs}</ref-list>"#))
}

fn refs_doc(refs: &str, body: &str) -> String {
    body_with_refs(
        &format!(
            r#"<sec><title>Results</title><p>Anchor <xref ref-type="bibr" rid="bib7">7</xref> text.</p></sec>{body}"#
        ),
        refs,
    )
}

fn doi_ref(id: &str, doi: &str) -> String {
    format!(
        r#"<ref id="{id}"><element-citation><pub-id pub-id-type="doi">{doi}</pub-id></element-citation></ref>"#
    )
}

#[test]
fn extractor_resolves_doi_with_prefix_and_case_normalization() {
    let xml = refs_doc(
        &doi_ref("bib7", "https://dx.doi.org/10.1038/NATURE10725"),
        "",
    );
    let out = extract_citation_evidence(&xml, &target_ids(Some("10.1038/nature10725"), None, None));
    assert_eq!(
        out,
        Ok(JatsCitationExtraction::Linked {
            ref_id: "bib7".into(),
            passages: vec![JatsCitationPassage {
                text: "Anchor 7 text.".into(),
                section_path: vec!["Results".into()],
                paragraph: 1,
                marker: "7".into(),
            }]
        })
    );
}

#[test]
fn extractor_rejects_doi_without_ten_prefix_or_slash() {
    let xml = refs_doc(
        &format!(
            "{}{}",
            doi_ref("bib7", "not-a-doi"),
            doi_ref("bib8", "10.1038-no-slash")
        ),
        "",
    );
    assert_eq!(
        extract_citation_evidence(&xml, &target_ids(Some("not-a-doi"), None, None)),
        Ok(JatsCitationExtraction::ReferenceUnresolved)
    );
}

#[test]
fn extractor_prefers_doi_over_pmid_when_both_match() {
    let xml = refs_doc(
        r#"<ref id="bib7"><element-citation><pub-id pub-id-type="doi">10.1/x</pub-id><pub-id pub-id-type="pmid">123</pub-id></element-citation></ref>"#,
        "",
    );
    let target = target_ids(Some("10.1/x"), Some("123"), None);
    assert!(matches!(
        extract_citation_evidence(&xml, &target),
        Ok(JatsCitationExtraction::Linked { .. })
    ));
}

#[test]
fn extractor_matches_pmid_by_decimal_value_with_prefix_and_zeros() {
    let xml = refs_doc(
        r#"<ref id="bib7"><element-citation><pub-id pub-id-type="pmid">pmid:0000123</pub-id></element-citation></ref>"#,
        "",
    );
    assert!(matches!(
        extract_citation_evidence(&xml, &target_ids(None, Some("123"), None)),
        Ok(JatsCitationExtraction::Linked { .. })
    ));
}

#[test]
fn extractor_matches_pmcid_case_insensitively_with_canonical_value() {
    let xml = refs_doc(
        r#"<ref id="bib7"><element-citation><pub-id pub-id-type="pmcid">pmcid:PMC0099</pub-id></element-citation></ref>"#,
        "",
    );
    assert!(matches!(
        extract_citation_evidence(&xml, &target_ids(None, None, Some("PMC99"))),
        Ok(JatsCitationExtraction::Linked { .. })
    ));
}

#[test]
fn extractor_rejects_lower_priority_match_with_conflicting_higher_identifier() {
    // The ref matches the target PMID but carries a different valid DOI, so
    // the PMID match is rejected and nothing else matches.
    let xml = refs_doc(
        r#"<ref id="bib7"><element-citation><pub-id pub-id-type="doi">10.9/other</pub-id><pub-id pub-id-type="pmid">123</pub-id></element-citation></ref>"#,
        "",
    );
    assert_eq!(
        extract_citation_evidence(&xml, &target_ids(None, Some("123"), None)),
        Ok(JatsCitationExtraction::ReferenceUnresolved)
    );
}

#[test]
fn extractor_fails_closed_on_two_distinct_dois_in_one_ref() {
    let xml = refs_doc(
        r#"<ref id="bib7"><element-citation><pub-id pub-id-type="doi">10.1/x</pub-id></element-citation><ext-link ext-link-type="doi">https://doi.org/10.2/y</ext-link></ref>"#,
        "",
    );
    assert_eq!(
        extract_citation_evidence(&xml, &target_ids(Some("10.1/x"), None, None)),
        Ok(JatsCitationExtraction::ReferenceUnresolved)
    );
}

#[test]
fn extractor_allows_identical_normalized_duplicates_inside_one_ref() {
    let xml = refs_doc(
        r#"<ref id="bib7"><element-citation><pub-id pub-id-type="doi">10.1/x</pub-id><pub-id pub-id-type="doi">https://doi.org/10.1/x</pub-id></element-citation></ref>"#,
        "",
    );
    assert!(matches!(
        extract_citation_evidence(&xml, &target_ids(Some("10.1/x"), None, None)),
        Ok(JatsCitationExtraction::Linked { .. })
    ));
}

#[test]
fn extractor_fails_closed_on_two_matching_refs() {
    let xml = refs_doc(
        &format!("{}{}", doi_ref("bib7", "10.1/x"), doi_ref("bib8", "10.1/x")),
        "",
    );
    assert_eq!(
        extract_citation_evidence(&xml, &target_ids(Some("10.1/x"), None, None)),
        Ok(JatsCitationExtraction::ReferenceUnresolved)
    );
}

#[test]
fn extractor_fails_closed_when_selected_id_duplicates_across_ref_lists() {
    let xml = article_with_body(&format!(
        r#"<p>Anchor <xref ref-type="bibr" rid="bib7">7</xref></p><ref-list>{}</ref-list><ref-list>{}</ref-list>"#,
        doi_ref("bib7", "10.1/x"),
        doi_ref("bib7", "10.9/unrelated")
    ));
    assert_eq!(
        extract_citation_evidence(&xml, &target_ids(Some("10.1/x"), None, None)),
        Ok(JatsCitationExtraction::ReferenceUnresolved)
    );
}

#[test]
fn extractor_fails_closed_when_selected_ref_lacks_a_nonblank_id() {
    let xml = article_with_body(
        r#"<p>Anchor <xref ref-type="bibr" rid="bib7">7</xref></p><ref-list><ref id="  "><element-citation><pub-id pub-id-type="doi">10.1/x</pub-id></element-citation></ref></ref-list>"#,
    );
    assert_eq!(
        extract_citation_evidence(&xml, &target_ids(Some("10.1/x"), None, None)),
        Ok(JatsCitationExtraction::ReferenceUnresolved)
    );
}

#[test]
fn extractor_treats_hostile_identifier_text_as_closed_not_matched() {
    let hostile = "10.1/x|`inj` $;; \"quote\"&amp;&lt;b&gt; café";
    let decoded = "10.1/x|`inj` $;; \"quote\"&<b> caf\u{e9}";
    let xml = refs_doc(&doi_ref("bib7", hostile), "");
    assert_eq!(
        extract_citation_evidence(&xml, &target_ids(Some(decoded), None, None)),
        Ok(JatsCitationExtraction::Linked {
            ref_id: "bib7".into(),
            passages: vec![JatsCitationPassage {
                text: "Anchor 7 text.".into(),
                section_path: vec!["Results".into()],
                paragraph: 1,
                marker: "7".into(),
            }]
        })
    );
    // The same hostile value stays closed for other targets.
    assert_eq!(
        extract_citation_evidence(&xml, &target_ids(Some("10.1/other"), None, None)),
        Ok(JatsCitationExtraction::ReferenceUnresolved)
    );
}

#[test]
fn extractor_ignores_grouped_markers_and_requires_exact_case_rid() {
    let xml = refs_doc(
        &doi_ref("bib7", "10.1/x"),
        r#"<p>Grouped <xref ref-type="bibr" rid="bib7 bib8">7, 8</xref> ignored.</p><p>Wrong case <xref ref-type="bibr" rid="BIB7">7</xref> ignored.</p>"#,
    );
    assert_eq!(
        extract_citation_evidence(&xml, &target_ids(Some("10.1/x"), None, None)),
        Ok(JatsCitationExtraction::Linked {
            ref_id: "bib7".into(),
            passages: vec![JatsCitationPassage {
                text: "Anchor 7 text.".into(),
                section_path: vec!["Results".into()],
                paragraph: 1,
                marker: "7".into(),
            }]
        })
    );
}

#[test]
fn extractor_links_adjacent_separate_markers_in_one_paragraph_once() {
    let xml = refs_doc(
        &format!("{}{}", doi_ref("bib7", "10.1/x"), doi_ref("bib8", "10.1/y")),
        r#"<p>Two adjacent <xref ref-type="bibr" rid="bib7">7</xref><xref ref-type="bibr" rid="bib8">8</xref> markers.</p>"#,
    );
    let out = extract_citation_evidence(&xml, &target_ids(Some("10.1/x"), None, None));
    match out {
        Ok(JatsCitationExtraction::Linked { passages, .. }) => {
            // The built-in anchor paragraph plus the adjacent-marker paragraph.
            assert_eq!(passages.len(), 2);
            assert_eq!(passages[0].text, "Anchor 7 text.");
            assert_eq!(passages[1].text, "Two adjacent 78 markers.");
            assert_eq!(passages[1].marker, "7");
            assert_eq!(passages[1].paragraph, 2);
        }
        other => panic!("expected linked, got {other:?}"),
    }
}

#[test]
fn extractor_excludes_paragraphs_in_table_figure_and_caption_scopes() {
    let xml = refs_doc(
        &doi_ref("bib7", "10.1/x"),
        r#"<p><table-wrap><table><tr><td>Inside <xref ref-type="bibr" rid="bib7">7</xref> table</td></tr></table></table-wrap></p><fig><caption>Fig <xref ref-type="bibr" rid="bib7">7</xref> caption</caption></fig><p>Good <xref ref-type="bibr" rid="bib7">7</xref> paragraph.</p>"#,
    );
    match extract_citation_evidence(&xml, &target_ids(Some("10.1/x"), None, None)) {
        Ok(JatsCitationExtraction::Linked { passages, .. }) => {
            // Only the anchor and the plain body paragraph survive.
            assert_eq!(passages.len(), 2);
            assert_eq!(passages[0].text, "Anchor 7 text.");
            assert_eq!(passages[1].text, "Good 7 paragraph.");
        }
        other => panic!("expected linked, got {other:?}"),
    }
}

#[test]
fn extractor_reports_nested_section_paths_and_paragraph_ordinals() {
    let xml = article_with_body(&format!(
        r#"<p>First paragraph without citation.</p><sec><title>Outer</title><p>Second.</p><sec><title>Inner Section</title><p>Deep <xref ref-type="bibr" rid="bib7">7</xref> paragraph.</p></sec></sec><ref-list>{}</ref-list>"#,
        doi_ref("bib7", "10.1/x")
    ));
    match extract_citation_evidence(&xml, &target_ids(Some("10.1/x"), None, None)) {
        Ok(JatsCitationExtraction::Linked { passages, .. }) => {
            assert_eq!(passages[0].section_path, ["Outer", "Inner Section"]);
            assert_eq!(passages[0].paragraph, 3);
        }
        other => panic!("expected linked, got {other:?}"),
    }
}

#[test]
fn extractor_keeps_document_order_dedupes_paragraphs_but_not_equal_text() {
    let xml = refs_doc(
        &doi_ref("bib7", "10.1/x"),
        r#"<p>Repeated <xref ref-type="bibr" rid="bib7">7</xref> twice <xref ref-type="bibr" rid="bib7">7</xref>.</p><p>Repeated <xref ref-type="bibr" rid="bib7">7</xref> by text only.</p><p>Later dropped by the cap <xref ref-type="bibr" rid="bib7">7</xref>.</p>"#,
    );
    match extract_citation_evidence(&xml, &target_ids(Some("10.1/x"), None, None)) {
        Ok(JatsCitationExtraction::Linked { passages, .. }) => {
            // Anchor, duplicated-marker paragraph, and an equal-text
            // paragraph carrying its own marker: equal text never dedupes.
            // The fourth paragraph is dropped by the three-passage cap.
            assert_eq!(passages.len(), 3);
            assert_eq!(passages[0].text, "Anchor 7 text.");
            assert_eq!(passages[1].text, "Repeated 7 twice 7.");
            assert_eq!(passages[1].paragraph, 2);
            assert_eq!(passages[2].text, "Repeated 7 by text only.");
        }
        other => panic!("expected linked, got {other:?}"),
    }
}

#[test]
fn extractor_truncates_long_paragraphs_around_first_marker_scalar() {
    let filler_a: String = "a".repeat(700);
    let filler_b: String = "b".repeat(700);
    let paragraph = format!(
        "<p>{} <xref ref-type=\"bibr\" rid=\"bib7\">7</xref> {}</p>",
        filler_a, filler_b
    );
    let xml = article_with_body(&format!(
        "{}<ref-list>{}</ref-list>",
        paragraph,
        doi_ref("bib7", "10.1/x")
    ));
    match extract_citation_evidence(&xml, &target_ids(Some("10.1/x"), None, None)) {
        Ok(JatsCitationExtraction::Linked { passages, .. }) => {
            let text = &passages[0].text;
            let scalars = text.chars().count();
            assert_eq!(scalars, 1_200, "passage must be exactly 1200 scalars");
            assert!(text.starts_with('\u{2026}'), "prefix ellipsis expected");
            assert!(text.ends_with('\u{2026}'), "suffix ellipsis expected");
            assert!(text.contains('7'), "marker must be retained");
        }
        other => panic!("expected linked, got {other:?}"),
    }
}

#[test]
fn extractor_passes_through_paragraphs_at_or_below_the_limit() {
    for (filler, expect_prefix, expect_suffix) in [(598, false, false), (599, false, false)] {
        let text: String = "x".repeat(filler);
        let paragraph = format!(
            "<p>{} <xref ref-type=\"bibr\" rid=\"bib7\">7</xref></p>",
            text
        );
        let xml = article_with_body(&format!(
            "{}<ref-list>{}</ref-list>",
            paragraph,
            doi_ref("bib7", "10.1/x")
        ));
        match extract_citation_evidence(&xml, &target_ids(Some("10.1/x"), None, None)) {
            Ok(JatsCitationExtraction::Linked { passages, .. }) => {
                let scalars = passages[0].text.chars().count();
                // filler + space + marker = filler + 2 scalars
                assert_eq!(scalars, filler + 2);
                assert_eq!(passages[0].text.starts_with('\u{2026}'), expect_prefix);
                assert_eq!(passages[0].text.ends_with('\u{2026}'), expect_suffix);
            }
            other => panic!("expected linked, got {other:?}"),
        }
    }
}

#[test]
fn extractor_skips_empty_marker_text_before_truncation() {
    let filler: String = "y".repeat(1_300);
    let paragraph = format!(
        "<p><xref ref-type=\"bibr\" rid=\"bib7\"></xref>{} <xref ref-type=\"bibr\" rid=\"bib7\">7</xref></p>",
        filler
    );
    let xml = article_with_body(&format!(
        "{}<ref-list>{}</ref-list>",
        paragraph,
        doi_ref("bib7", "10.1/x")
    ));
    match extract_citation_evidence(&xml, &target_ids(Some("10.1/x"), None, None)) {
        Ok(JatsCitationExtraction::Linked { passages, .. }) => {
            let text = &passages[0].text;
            assert_eq!(text.chars().count(), 1_199);
            assert!(
                text.contains('7'),
                "later nonblank marker anchors the slice"
            );
            assert!(text.ends_with('7'));
            assert!(text.starts_with('\u{2026}'));
        }
        other => panic!("expected linked, got {other:?}"),
    }
}

#[test]
fn extractor_caps_passages_at_three_in_document_order() {
    let mut paragraphs = String::new();
    for index in 0..4 {
        paragraphs.push_str(&format!(
            "<p>P{index} <xref ref-type=\"bibr\" rid=\"bib7\">7</xref></p>"
        ));
    }
    let xml = refs_doc(&doi_ref("bib7", "10.1/x"), &paragraphs);
    match extract_citation_evidence(&xml, &target_ids(Some("10.1/x"), None, None)) {
        Ok(JatsCitationExtraction::Linked { passages, .. }) => {
            assert_eq!(passages.len(), 3);
            assert_eq!(passages[0].text, "Anchor 7 text.");
            assert_eq!(passages[1].text, "P0 7");
            assert_eq!(passages[2].text, "P1 7");
        }
        other => panic!("expected linked, got {other:?}"),
    }
}

#[test]
fn extractor_returns_marker_unlinked_when_no_marker_reaches_a_paragraph() {
    let xml = body_with_refs(r#"<p>No marker here.</p>"#, &doi_ref("bib7", "10.1/x"));
    assert_eq!(
        extract_citation_evidence(&xml, &target_ids(Some("10.1/x"), None, None)),
        Ok(JatsCitationExtraction::MarkerUnlinked {
            ref_id: "bib7".into()
        })
    );
}

#[test]
fn extractor_returns_parsed_unusable_for_non_article_or_bodyless_documents() {
    assert_eq!(
        extract_citation_evidence(
            "<html><body>x</body></html>",
            &target_ids(Some("10.1/x"), None, None)
        ),
        Ok(JatsCitationExtraction::ParsedUnusable)
    );
    let bodyless = r#"<article><front><article-meta><article-title>T</article-title></article-meta></front><ref-list><ref id="bib7"><element-citation><pub-id pub-id-type="doi">10.1/x</pub-id></element-citation></ref></ref-list></article>"#;
    assert_eq!(
        extract_citation_evidence(bodyless, &target_ids(Some("10.1/x"), None, None)),
        Ok(JatsCitationExtraction::ParsedUnusable)
    );
}

#[test]
fn extractor_fails_on_malformed_xml() {
    assert!(
        extract_citation_evidence("<article><body>", &target_ids(Some("10.1/x"), None, None))
            .is_err()
    );
}

#[test]
fn extractor_excludes_paragraphs_inside_boxed_text() {
    let xml = article_with_body(
        r#"<boxed-text><p>Boxed callout citing <xref ref-type="bibr" rid="bib7">7</xref>.</p></boxed-text><ref-list><ref id="bib7"><element-citation><pub-id pub-id-type="doi">10.1/x</pub-id></element-citation></ref></ref-list>"#,
    );
    assert_eq!(
        extract_citation_evidence(&xml, &target_ids(Some("10.1/x"), None, None)),
        Ok(JatsCitationExtraction::MarkerUnlinked {
            ref_id: "bib7".into()
        })
    );
}

#[test]
fn extractor_anchors_truncation_on_the_marker_not_an_earlier_equal_string() {
    // The marker string "7" also occurs early in unrelated text. A naive
    // `text.find(marker)` anchor would clamp the window around that early
    // digit and drop the real marker near the end.
    let filler_a: String = "a".repeat(1_100);
    let filler_b: String = "b".repeat(700);
    let paragraph = format!(
        "<p>In study 7 of the cohort, {} <xref ref-type=\"bibr\" rid=\"bib7\">7</xref> {}</p>",
        filler_a, filler_b
    );
    let xml = article_with_body(&format!(
        "{}<ref-list>{}</ref-list>",
        paragraph,
        doi_ref("bib7", "10.1/x")
    ));
    match extract_citation_evidence(&xml, &target_ids(Some("10.1/x"), None, None)) {
        Ok(JatsCitationExtraction::Linked { passages, .. }) => {
            let text = &passages[0].text;
            assert_eq!(text.chars().count(), 1_200, "bounded passage length");
            // The slice must retain the real marker: its first scalar sits
            // exactly at start + 599 by the clamp rule, and the retained
            // window contains " a7" (space, last filler a, marker), not the
            // early digit.
            let chars: Vec<char> = text.chars().collect();
            assert_eq!(chars[599 + 1], '7', "marker first scalar inside window");
            // The early unrelated digit must NOT be the anchor: it sits far
            // before the window start.
            assert!(
                !text.starts_with("In study 7"),
                "window must not begin at the earlier unrelated digit"
            );
        }
        other => panic!("expected linked, got {other:?}"),
    }
}

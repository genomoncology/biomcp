//! JATS extraction regression tests.

use super::*;

fn extract_text_from_xml(xml: &str) -> String {
    classify_jats_document(xml)
        .ok()
        .and_then(|classified| classified.markdown)
        .unwrap_or_default()
}

#[test]
fn extract_text_from_jats_preserves_structure_and_renders_references() {
    let xml = r#"<?xml version="1.0"?>
<!DOCTYPE article PUBLIC
  "-//NLM//DTD JATS (Z39.96) Journal Archiving and Interchange DTD v1.4 20241031//EN"
  "https://example.invalid/JATS-archivearticle1-4.dtd">
<article xmlns:xlink="http://www.w3.org/1999/xlink">
  <front>
    <journal-meta>
      <journal-title-group>
        <journal-title>Noise Journal</journal-title>
      </journal-title-group>
      <issn>1234-5678</issn>
    </journal-meta>
    <article-meta>
      <title-group>
        <article-title>Precision oncology in melanoma</article-title>
      </title-group>
      <permissions>
        <license><license-p>Creative Commons text that should not leak.</license-p></license>
      </permissions>
      <abstract>
        <p>Abstract text with <xref ref-type="bibr" rid="ref1">1</xref> and <italic>signal</italic>.</p>
      </abstract>
    </article-meta>
  </front>
  <body>
    <sec>
      <title>Introduction</title>
      <p>Body paragraph with <bold>important</bold> findings at 70 &#181;m and <ext-link xlink:href="https://example.org/resource">external evidence</ext-link>.</p>
      <fig id="f1">
        <label>Figure 1</label>
        <caption>
          <title>Response overview</title>
          <p>Treatment response summary.</p>
        </caption>
      </fig>
      <table-wrap id="t1">
        <label>Table 1</label>
        <caption><title>Patient characteristics</title></caption>
        <table>
          <thead>
            <tr><th>Gene</th><th>Count</th></tr>
          </thead>
          <tbody>
            <tr><td>BRAF</td><td>12</td></tr>
            <tr><td>NRAS</td><td>4</td></tr>
          </tbody>
        </table>
      </table-wrap>
      <sec>
        <title>Methods</title>
        <list list-type="order">
          <list-item><p>Collect tumor samples</p></list-item>
          <list-item><p>Sequence genes</p></list-item>
        </list>
      </sec>
    </sec>
  </body>
  <back>
    <ref-list>
      <ref id="ref1"><label>1</label><mixed-citation>Reference one.</mixed-citation></ref>
      <ref id="ref2"><label>2</label><mixed-citation>Reference two.</mixed-citation></ref>
    </ref-list>
  </back>
</article>
"#;

    let out = extract_text_from_xml(xml);
    assert!(out.contains("# Precision oncology in melanoma"));
    assert!(out.contains("## Abstract"));
    assert!(out.contains("## Introduction"));
    assert!(out.contains("### Methods"));
    assert!(out.contains("Abstract text with [1] and *signal*."));
    assert!(out.contains("Body paragraph with **important** findings at 70 µm"));
    assert!(out.contains("[external evidence](https://example.org/resource)"));
    let quality = classify_jats_document(xml).expect("valid JATS").quality;
    assert!(quality.has_sections);
    assert!(quality.has_tables);
    assert!(quality.has_references);
    assert!(out.contains("> **Figure 1.** Response overview Treatment response summary."));
    assert!(out.contains("| Gene | Count |"));
    assert!(out.contains("| BRAF | 12 |"));
    assert!(out.contains("1. Collect tumor samples"));
    assert!(out.contains("## References"));
    assert!(out.contains("1. Reference one."));
    assert!(out.contains("2. Reference two."));
    assert!(!out.contains("references cited."));
    assert!(!out.contains("Noise Journal"));
    assert!(!out.contains("Creative Commons text that should not leak."));
}

#[test]
fn extract_text_from_jats_renders_element_citation_fields_and_ids() {
    let xml = r#"
<article xmlns:xlink="http://www.w3.org/1999/xlink">
  <front>
    <article-meta>
      <title-group><article-title>Element citation article</article-title></title-group>
    </article-meta>
  </front>
  <back>
    <ref-list>
      <ref id="ref1">
        <element-citation publication-type="journal">
          <person-group person-group-type="author">
            <name><surname>Doe</surname><given-names>JA</given-names></name>
            <name><surname>Roe</surname><given-names>R</given-names></name>
            <etal/>
          </person-group>
          <article-title>Structured reference title</article-title>
          <source>Journal of Tests</source>
          <year>2024</year>
          <volume>12</volume>
          <issue>3</issue>
          <elocation-id>e45</elocation-id>
          <comment>Online ahead of print</comment>
          <pub-id pub-id-type="doi">10.1000/test-doi</pub-id>
          <pub-id pub-id-type="pmid">123456</pub-id>
          <pub-id pub-id-type="pmcid">PMC123456</pub-id>
          <ext-link ext-link-type="uri" xlink:href="https://example.org/dataset">Dataset</ext-link>
        </element-citation>
      </ref>
    </ref-list>
  </back>
</article>
"#;

    let out = extract_text_from_xml(xml);
    assert!(out.contains(
        "1. Doe JA, Roe R, et al. Structured reference title. Journal of Tests. 2024;12(3):e45. Online ahead of print. [10.1000/test-doi](https://doi.org/10.1000/test-doi). PMID: 123456. PMCID: PMC123456. [Dataset](https://example.org/dataset)"
    ));
}

#[test]
fn extract_text_from_jats_renders_mixed_citation_doi_links() {
    let xml = r#"
<article xmlns:xlink="http://www.w3.org/1999/xlink">
  <front>
    <article-meta>
      <title-group><article-title>Mixed citation article</article-title></title-group>
    </article-meta>
  </front>
  <back>
    <ref-list>
      <ref id="ref1">
        <mixed-citation>Alpha study. <pub-id pub-id-type="doi">10.1000/alpha</pub-id></mixed-citation>
      </ref>
      <ref id="ref2">
        <mixed-citation>Beta study. <ext-link ext-link-type="doi" xlink:href="10.1000/beta">doi</ext-link></mixed-citation>
      </ref>
    </ref-list>
  </back>
</article>
"#;

    let out = extract_text_from_xml(xml);
    assert!(out.contains("1. Alpha study. [10.1000/alpha](https://doi.org/10.1000/alpha)"));
    assert!(out.contains("2. Beta study. [10.1000/beta](https://doi.org/10.1000/beta)"));
}

#[test]
fn extract_text_from_jats_reference_fallback_omits_duplicate_label() {
    let xml = r#"
<article>
  <front>
    <article-meta>
      <title-group><article-title>Fallback citation article</article-title></title-group>
    </article-meta>
  </front>
  <back>
    <ref-list>
      <ref id="ref1">
        <label>S1</label>
        <note><p>Supplemental dataset companion</p></note>
      </ref>
    </ref-list>
  </back>
</article>
"#;

    let out = extract_text_from_xml(xml);
    assert!(out.contains("1. [S1] Supplemental dataset companion"));
    assert!(!out.contains("1. [S1] S1 Supplemental dataset companion"));
}

#[test]
fn extract_text_from_jats_merges_multiple_ref_lists() {
    let xml = r#"
<article>
  <front>
    <article-meta>
      <title-group><article-title>Multiple ref-list article</article-title></title-group>
    </article-meta>
  </front>
  <back>
    <ref-list>
      <ref id="ref1"><mixed-citation>First reference.</mixed-citation></ref>
    </ref-list>
    <sec>
      <title>Supplement</title>
      <ref-list>
        <ref id="ref2"><mixed-citation>Second reference.</mixed-citation></ref>
      </ref-list>
    </sec>
  </back>
</article>
"#;

    let out = extract_text_from_xml(xml);
    let first = out.find("1. First reference.").expect("first ref present");
    let second = out
        .find("2. Second reference.")
        .expect("second ref present");
    assert!(first < second);
}

#[test]
fn extract_text_from_jats_preserves_complex_table_cells_and_spans() {
    let xml = r#"
<article>
  <front>
    <article-meta>
      <title-group><article-title>Irregular table article</article-title></title-group>
    </article-meta>
  </front>
  <body>
    <table-wrap>
      <label>Table 7</label>
      <caption><title>Irregular measurements</title></caption>
      <table>
        <tbody>
          <tr><th rowspan="2">Marker</th><th>Value</th></tr>
          <tr><td>42</td></tr>
        </tbody>
      </table>
    </table-wrap>
  </body>
</article>
"#;

    let out = extract_text_from_xml(xml);
    assert!(out.contains("Table 7"));
    assert!(out.contains("Irregular measurements"));
    assert!(out.contains("merged-cell layout may be lossy"));
    assert!(out.contains("Row 1: Marker [rowspan=2] | Value"));
    assert!(out.contains("Row 2: 42"));
    assert!(!out.contains("complex table omitted"));
}

#[test]
fn real_pmc6329583_capture_preserves_all_six_complex_tables() {
    let response = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/testdata/sources/ncbi_efetch/pmc6329583.xml"
    ));
    let xml = crate::sources::ncbi_efetch::normalize_article_xml(response)
        .unwrap()
        .unwrap();
    let out = extract_text_from_xml(&xml);
    for sentinel in [
        "Pathogenic Criteria",
        "Macrocephaly of >2 SD to <4 SD",
        "Supporting (PS4_P): 1-1.5 points",
        "Round 1 review –criteria applied",
        "Round 1 review – criteria applied",
        "ClinVar Status (as of 10.29.17)",
    ] {
        assert!(out.contains(sentinel), "missing cell: {sentinel}");
    }
    assert_eq!(out.matches("merged-cell layout may be lossy").count(), 6);
    assert!(!out.contains("complex table omitted"));
}

#[test]
fn extract_text_from_jats_renders_floats_group_after_body_before_references() {
    let xml = r#"
<article>
  <front><article-meta><title-group><article-title>Float order</article-title></title-group></article-meta></front>
  <body>
    <sec>
      <title>Results</title>
      <p>Body text.</p>
      <fig id="fig1"><label>Figure 1</label><caption><p>Body figure.</p></caption></fig>
    </sec>
  </body>
  <floats-group>
    <fig id="fig1"><label>Figure 1</label><caption><p>Duplicate body figure.</p></caption></fig>
    <fig id="fig2"><label>Figure 2</label><caption><p>Floats figure.</p></caption></fig>
  </floats-group>
  <back><ref-list><ref><mixed-citation>Reference one.</mixed-citation></ref></ref-list></back>
</article>
"#;

    let out = extract_text_from_xml(xml);
    assert_eq!(out.matches("> **Figure 1.**").count(), 1);
    assert!(out.contains("> **Figure 1.** Body figure."));
    assert!(!out.contains("Duplicate body figure"));
    let figure = out
        .find("> **Figure 2.** Floats figure.")
        .expect("float figure");
    let references = out.find("## References").expect("references");
    assert!(figure < references);
}

#[test]
fn extract_text_from_jats_renders_supplementary_material_metadata() {
    let xml = r#"
<article xmlns:xlink="http://www.w3.org/1999/xlink">
  <body>
    <supplementary-material id="s1" xlink:href="traces-s1.csv">
      <label>Supplementary Data S1</label>
      <caption><p>Measurement traces for the treatment cohort.</p></caption>
      <media xlink:href="traces-s1.csv" />
    </supplementary-material>
  </body>
</article>
"#;

    let out = extract_text_from_xml(xml);
    assert!(out.contains("**Supplementary Data S1.**"));
    assert!(out.contains("Measurement traces for the treatment cohort."));
    assert!(out.contains("File: traces-s1.csv"));
    assert_eq!(out.matches("traces-s1.csv").count(), 1);
}

#[test]
fn extract_text_from_jats_suppresses_source_parenthesized_xref_and_preserves_boundary_spacing() {
    let xml = r#"
<article>
  <body>
    <p>Europe PMC body text with callout (<xref ref-type="fig" rid="fig2">Figure 2</xref>) and B-RAF<sup>V600E</sup>.PLX4032 boundary text.</p>
  </body>
</article>
"#;

    let out = extract_text_from_xml(xml);
    assert!(out.contains(
        "Europe PMC body text with callout (Figure 2) and B-RAF^V600E^. PLX4032 boundary text."
    ));
    assert!(!out.contains("((Figure 2))"));
}

#[test]
fn extract_text_from_jats_wraps_unparenthesized_figure_xrefs() {
    let xml = r#"
<article>
  <body>
    <p>See <xref ref-type="fig" rid="fig2">Figure 2</xref> for details.</p>
  </body>
</article>
"#;

    let out = extract_text_from_xml(xml);
    assert!(out.contains("See (Figure 2) for details."));
}

#[test]
fn jats_classification_requires_meaningful_direct_body_content() {
    let fulltext_cases = [
        "<article><body><p>x</p></body></article>",
        "<article><body><sec><title>Results</title><list><list-item><p>item</p></list-item></list></sec></body></article>",
        "<article><body><table-wrap><table><tr><td>cell</td></tr></table></table-wrap></body></article>",
        "<article><body><fig><caption><p>caption</p></caption></fig></body></article>",
        "<article><body><disp-quote>quoted result</disp-quote></body></article>",
        "<article><body><preformat>result</preformat></body></article>",
    ];
    for xml in fulltext_cases {
        let classified = classify_jats_document(xml).expect("valid body fixture");
        assert_eq!(
            classified.coverage,
            ArticleDocumentCoverage::FullText,
            "fixture: {xml}"
        );
        assert!(classified.quality.has_fulltext_signal);
        assert!(classified.markdown.is_some());
    }

    let partial_cases = [
        (
            "<article><front><article-meta><abstract><p>first abstract shape</p></abstract></article-meta></front></article>",
            ArticleDocumentCoverage::AbstractOnly,
        ),
        (
            "<article><front><abstract><sec><p>second abstract shape</p></sec></abstract></front><body><sec><title>Heading only</title></sec></body></article>",
            ArticleDocumentCoverage::AbstractOnly,
        ),
        (
            "<article><front><article-meta><title-group><article-title>Title only</article-title></title-group></article-meta></front></article>",
            ArticleDocumentCoverage::MetadataOnly,
        ),
        (
            "<article><floats-group><fig><caption><p>float only</p></caption></fig></floats-group><back><ref-list><ref><mixed-citation>reference only</mixed-citation></ref></ref-list></back></article>",
            ArticleDocumentCoverage::MetadataOnly,
        ),
        (
            "<article><back><abstract><p>back-matter abstract</p></abstract></back></article>",
            ArticleDocumentCoverage::MetadataOnly,
        ),
        (
            "<article><body><supplementary-material><p>supplement only</p></supplementary-material></body></article>",
            ArticleDocumentCoverage::MetadataOnly,
        ),
        (
            "<article><body><unsupported><p>unsupported nested paragraph</p></unsupported></body></article>",
            ArticleDocumentCoverage::MetadataOnly,
        ),
    ];
    for (xml, expected) in partial_cases {
        let classified = classify_jats_document(xml).expect("valid partial fixture");
        assert_eq!(classified.coverage, expected);
        assert!(!classified.quality.has_fulltext_signal);
    }

    assert!(classify_jats_document("<article>").is_err());
    assert!(classify_jats_document("<metadata><title>wrong root</title></metadata>").is_err());
}

#[test]
fn entity_bearing_jats_is_not_rendered_through_fallback() {
    let xml = r#"<!DOCTYPE article [<!ENTITY unsafe "expanded">]><article><body><p>&unsafe;</p></body></article>"#;
    assert!(extract_text_from_xml(xml).is_empty());
    assert!(classify_jats_document(xml).is_err());
}

// --- Ticket 1145: citation-evidence extractor ---

use super::refs::{JatsCitationExtraction, JatsCitationTargetIds, extract_citation_evidence};

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
            passages: vec![super::refs::citation::JatsCitationPassage {
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
            passages: vec![super::refs::citation::JatsCitationPassage {
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
            passages: vec![super::refs::citation::JatsCitationPassage {
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

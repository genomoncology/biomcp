//! JATS extraction regression tests.

use super::*;

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
    let quality = jats_quality_flags(xml);
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
fn extract_text_from_jats_marks_complex_tables() {
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
    assert!(out.contains("*[complex table omitted: 2×2, merged cells]*"));
    assert!(!out.contains("| Marker | Value |"));
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
fn extract_text_from_xml_falls_back_for_non_jats_and_malformed_xml() {
    let non_jats = "<root><meta>ignored?</meta><p>Alpha</p><p>Beta</p></root>";
    let malformed = "<article><body><p>Broken";

    let non_jats_out = extract_text_from_xml(non_jats);
    let malformed_out = extract_text_from_xml(malformed);

    assert_eq!(non_jats_out, "ignored?AlphaBeta");
    assert_eq!(malformed_out, "Broken");
}

#[test]
fn entity_bearing_jats_is_not_rendered_through_fallback() {
    let xml = r#"<!DOCTYPE article [<!ENTITY unsafe "expanded">]><article><body><p>&unsafe;</p></body></article>"#;
    assert!(extract_text_from_xml(xml).is_empty());
    assert_eq!(jats_quality_flags(xml), ArticleFulltextQuality::default());
}

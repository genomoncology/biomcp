//! Supplement parser format and resource boundary cases.

use super::*;
use std::io::Write as _;

const BRCA1_SHEET1_XLSX: &[u8] =
    include_bytes!("../../../../tests/fixtures/article/supplements/30209399-brca1-sheet1.xlsx");
const AXIN2_ST2_XLSX: &[u8] =
    include_bytes!("../../../../tests/fixtures/article/supplements/28944238-axin2-st2.xlsx");
const LEGACY_XLS: &[u8] =
    include_bytes!("../../../../tests/fixtures/article/supplements/legacy-supplement.xls");
const METHOD_NOTES_DOCX: &[u8] =
    include_bytes!("../../../../tests/fixtures/article/supplements/method-notes.docx");
const SUPPLEMENT_NOTES_PDF: &[u8] =
    include_bytes!("../../../../tests/fixtures/article/supplements/supplement-notes.pdf");

fn budget() -> SupplementTextBudget {
    SupplementTextBudget {
        max_member_bytes: 64 * 1024 * 1024,
        max_aggregate_bytes: 256 * 1024 * 1024,
        max_pdf_pages: 500,
        archive_depth: 2,
    }
}

fn hit<'a>(extraction: &'a SupplementTextExtraction, needle: &str) -> &'a SupplementTextBlock {
    extraction
        .blocks
        .iter()
        .find(|block| block.text.contains(needle))
        .unwrap_or_else(|| panic!("no block holds {needle}"))
}

#[test]
fn xlsx_extract_locates_a_row_on_sheet1() {
    let extraction = extract_supplement_text(SupplementFormat::Xlsx, BRCA1_SHEET1_XLSX, &budget());
    assert!(extraction.problems.is_empty(), "{:?}", extraction.problems);
    let block = hit(&extraction, "F1761I");
    assert_eq!(block.worksheet.as_deref(), Some("Sheet1"));
    assert_eq!(block.row, Some(2780));
    assert!(block.text.contains("BRCA1"));
}

#[test]
fn xlsx_extract_locates_a_named_worksheet_row() {
    let extraction = extract_supplement_text(SupplementFormat::Xlsx, AXIN2_ST2_XLSX, &budget());
    assert!(extraction.problems.is_empty(), "{:?}", extraction.problems);
    let block = hit(&extraction, "P245S");
    assert_eq!(block.worksheet.as_deref(), Some("ST2"));
    assert_eq!(block.row, Some(85));
    assert!(block.text.contains("AXIN2"));
    // The decoy worksheets hold no match of their own.
    assert_eq!(
        extraction
            .blocks
            .iter()
            .filter(|block| block.worksheet.as_deref() == Some("ST1")
                || block.worksheet.as_deref() == Some("ST3"))
            .filter(|block| block.text.contains("P245S"))
            .count(),
        0
    );
}

#[test]
fn xls_extract_reads_the_legacy_binary_workbook() {
    let extraction = extract_supplement_text(SupplementFormat::Xls, LEGACY_XLS, &budget());
    assert!(extraction.problems.is_empty(), "{:?}", extraction.problems);
    let block = hit(&extraction, "R273H");
    assert_eq!(block.worksheet.as_deref(), Some("Sheet1"));
    assert_eq!(block.row, Some(12));
    assert!(block.text.contains("TP53"));
}

#[test]
fn docx_extract_locates_a_paragraph() {
    let extraction = extract_supplement_text(SupplementFormat::Docx, METHOD_NOTES_DOCX, &budget());
    assert!(extraction.problems.is_empty(), "{:?}", extraction.problems);
    let block = hit(&extraction, "G12D");
    assert_eq!(block.paragraph, Some(3));
    assert!(block.text.contains("KRAS"));
}

#[test]
fn csv_extract_locates_a_record_row() {
    let bytes = b"row,gene,change\n1,FILLER,F0000X\n2,EGFR,L858R\n".to_vec();
    let extraction = extract_supplement_text(SupplementFormat::Csv, &bytes, &budget());
    assert!(extraction.problems.is_empty(), "{:?}", extraction.problems);
    let block = hit(&extraction, "L858R");
    assert_eq!(block.row, Some(3));
}

#[test]
fn pdf_extract_locates_a_page() {
    let extraction =
        extract_supplement_text(SupplementFormat::Pdf, SUPPLEMENT_NOTES_PDF, &budget());
    assert!(extraction.problems.is_empty(), "{:?}", extraction.problems);
    let block = hit(&extraction, "Q61K");
    assert_eq!(block.page, Some(2));
    assert!(block.text.contains("NRAS"));
    assert_eq!(extraction.blocks.len(), 3);
}

#[test]
fn pdf_page_budget_marks_truncation() {
    let mut small = budget();
    small.max_pdf_pages = 1;
    let extraction = extract_supplement_text(SupplementFormat::Pdf, SUPPLEMENT_NOTES_PDF, &small);
    assert!(extraction.truncated);
    assert_eq!(extraction.blocks.len(), 1);
}

#[test]
fn zip_extract_locates_a_member_and_reports_its_siblings() {
    let mut out = std::io::Cursor::new(Vec::new());
    {
        let mut archive = zip::ZipWriter::new(&mut out);
        let options = zip::write::SimpleFileOptions::default();
        archive.start_file("tables/st2.csv", options).unwrap();
        archive
            .write_all(b"row,gene,change\n1,FILLER,F0000X\n2,AXIN2,P245S\n")
            .unwrap();
        archive.start_file("figure-1.gif", options).unwrap();
        archive.write_all(b"GIF89a figure bytes").unwrap();
        archive.start_file("empty-notes.docx", options).unwrap();
        archive.write_all(b"").unwrap();
        archive.finish().unwrap();
    }
    let bytes = out.into_inner();
    let extraction = extract_supplement_text(SupplementFormat::Zip, &bytes, &budget());
    let block = hit(&extraction, "P245S");
    assert_eq!(block.member.as_deref(), Some("tables/st2.csv"));
    assert_eq!(block.row, Some(3));
    let reasons: Vec<(Option<&str>, &str, &str)> = extraction
        .problems
        .iter()
        .map(|problem| {
            (
                problem.member.as_deref(),
                problem.format.as_str(),
                problem.reason.as_str(),
            )
        })
        .collect();
    assert!(
        reasons.contains(&(Some("figure-1.gif"), "zip", "unsupported member format")),
        "{reasons:?}"
    );
    assert!(
        reasons.contains(&(Some("empty-notes.docx"), "docx", "empty archive member")),
        "{reasons:?}"
    );
}

#[test]
fn zip_member_budget_reports_the_size_limit() {
    let mut out = std::io::Cursor::new(Vec::new());
    {
        let mut archive = zip::ZipWriter::new(&mut out);
        let options = zip::write::SimpleFileOptions::default();
        archive.start_file("big-table.csv", options).unwrap();
        archive.write_all(&vec![b'x'; 2048]).unwrap();
        archive.finish().unwrap();
    }
    let mut small = budget();
    small.max_member_bytes = 1024;
    let extraction = extract_supplement_text(SupplementFormat::Zip, &out.into_inner(), &small);
    assert!(extraction.truncated);
    assert_eq!(extraction.problems.len(), 1);
    assert_eq!(
        extraction.problems[0].member.as_deref(),
        Some("big-table.csv")
    );
    assert_eq!(
        extraction.problems[0].reason,
        "archive member exceeds the extraction size limit"
    );
}

fn archive_bytes(name: &str, bytes: &[u8]) -> Vec<u8> {
    let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
    archive
        .start_file(name, zip::write::SimpleFileOptions::default())
        .unwrap();
    archive.write_all(bytes).unwrap();
    archive.finish().unwrap().into_inner()
}

#[test]
fn extraction_limits_cover_nested_archives_and_document_packages() {
    let table = vec![b'x'; 1000];
    let inner = archive_bytes("table.csv", &table);
    let outer = archive_bytes("inner.zip", &inner);
    let mut small = budget();
    small.max_aggregate_bytes = inner.len() as u64 + 999;
    let extraction = extract_supplement_text(SupplementFormat::Zip, &outer, &small);
    assert!(
        extraction
            .problems
            .iter()
            .any(|p| p.reason == "extraction size limit exceeded"),
        "nested members share their source budget: {:?}",
        extraction.problems
    );
    for (format, bytes) in [
        (SupplementFormat::Docx, METHOD_NOTES_DOCX),
        (SupplementFormat::Xlsx, BRCA1_SHEET1_XLSX),
    ] {
        small.max_member_bytes = 16;
        small.max_aggregate_bytes = 32;
        let extraction = extract_supplement_text(format, bytes, &small);
        assert!(
            extraction.blocks.is_empty(),
            "oversized package must not be parsed: {format:?}"
        );
        assert!(
            extraction
                .problems
                .iter()
                .any(|p| p.reason.contains("extraction size limit")),
            "package bytes obey the same limits: {:?}",
            extraction.problems
        );
    }
}

#[test]
fn only_one_nested_archive_is_searched() {
    let table = archive_bytes("table.csv", b"TARGET");
    let nested = archive_bytes("nested.zip", &table);
    let source = archive_bytes("inner.zip", &nested);
    let mut limits = budget();
    limits.archive_depth = 1;
    let extraction = extract_supplement_text(SupplementFormat::Zip, &source, &limits);
    assert!(extraction.blocks.is_empty());
    assert!(
        extraction
            .problems
            .iter()
            .any(|p| p.reason.contains("extraction depth"))
    );
}

#[test]
fn workbook_package_limits_follow_bytes_when_extension_is_xls() {
    let mut limits = budget();
    // The compressed workbook and each rendered row fit, but a worksheet XML
    // member does not. Auto-detection must not bypass its expansion budget.
    limits.max_member_bytes = BRCA1_SHEET1_XLSX.len() as u64 + 1;
    let extraction = extract_supplement_text(SupplementFormat::Xls, BRCA1_SHEET1_XLSX, &limits);
    assert!(
        extraction.blocks.is_empty(),
        "oversized XML must not be parsed"
    );
    assert!(
        extraction
            .problems
            .iter()
            .any(|problem| problem.reason.contains("extraction size limit"))
    );
}

#[test]
fn unreadable_bytes_report_format_and_reason() {
    // The scrubbed two-route fixture shape: a .xlsx name over bytes that
    // are not a workbook.
    let bytes = b"PK\x03\x04\nscrubbed two-route supplement XLSX fixture bytes\n".to_vec();
    let extraction = extract_supplement_text(SupplementFormat::Xlsx, &bytes, &budget());
    assert!(extraction.blocks.is_empty());
    assert_eq!(extraction.problems.len(), 1);
    assert_eq!(extraction.problems[0].format, "xlsx");
    assert!(
        extraction.problems[0]
            .reason
            .contains("workbook could not be opened")
    );
}

#[test]
fn format_dispatch_reads_file_extensions_case_insensitively() {
    assert_eq!(
        supplement_format_for_filename("SUPPLEMENT.XLSX"),
        Some(SupplementFormat::Xlsx)
    );
    assert_eq!(
        supplement_format_for_filename("NIHMS265402-Supplementary_Tables.xls"),
        Some(SupplementFormat::Xls)
    );
    assert_eq!(supplement_format_for_filename("readme.txt"), None);
    assert_eq!(supplement_format_for_filename("no-extension"), None);
}

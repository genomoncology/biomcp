//! Supplement text extraction for the article supplement search (2152).
//!
//! Every extractor is pure: bytes in, text blocks with exact locators out.
//! A file or archive member that cannot be read becomes a problem entry
//! naming its format and reason, so an empty search stays distinguishable
//! from an unread file. Nothing here interprets content; callers match text.

use std::io::{Cursor, Read};

use calamine::{Data, Reader, open_workbook_auto_from_rs};
use serde::{Deserialize, Serialize};

/// A single extractable supplement format, decided by file extension.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SupplementFormat {
    Xlsx,
    Xls,
    Csv,
    Docx,
    Pdf,
    Zip,
}

impl SupplementFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            SupplementFormat::Xlsx => "xlsx",
            SupplementFormat::Xls => "xls",
            SupplementFormat::Csv => "csv",
            SupplementFormat::Docx => "docx",
            SupplementFormat::Pdf => "pdf",
            SupplementFormat::Zip => "zip",
        }
    }
}

/// Returns the extractable supplement format a file name names, if any.
pub fn supplement_format_for_filename(filename: &str) -> Option<SupplementFormat> {
    let lower = filename.to_ascii_lowercase();
    let extension = lower.rsplit_once('.')?.1;
    match extension {
        "xlsx" => Some(SupplementFormat::Xlsx),
        "xls" => Some(SupplementFormat::Xls),
        "csv" => Some(SupplementFormat::Csv),
        "docx" => Some(SupplementFormat::Docx),
        "pdf" => Some(SupplementFormat::Pdf),
        "zip" => Some(SupplementFormat::Zip),
        _ => None,
    }
}

/// Extraction limits, stated once by the caller so tests can shrink them.
#[derive(Clone, Copy, Debug)]
pub struct SupplementTextBudget {
    /// Largest decompressed archive member that is extracted.
    pub max_member_bytes: u64,
    /// Largest aggregate decompressed bytes extracted from one source file.
    pub max_aggregate_bytes: u64,
    /// PDF pages extracted per file; later pages are not searched.
    pub max_pdf_pages: u32,
    /// Archive members may nest this many more archive levels.
    pub archive_depth: usize,
}

/// One unit of extracted text with the locator that found it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupplementTextBlock {
    /// Archive member the text came from, when the source file is an archive.
    pub member: Option<String>,
    /// Worksheet name for spreadsheet text.
    pub worksheet: Option<String>,
    /// One-based worksheet or CSV record row.
    pub row: Option<u32>,
    /// One-based PDF page.
    pub page: Option<u32>,
    /// One-based document paragraph.
    pub paragraph: Option<u32>,
    pub text: String,
}

/// A file, or archive member, that produced no text and why.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupplementExtractionProblem {
    /// Archive member the problem is about, when there is one.
    pub member: Option<String>,
    pub format: String,
    pub reason: String,
    /// The bytes read fine but held no text: a text layer problem, not a
    /// parse failure.
    pub empty_text: bool,
}

impl SupplementExtractionProblem {
    /// Constructs a parse-failure report. Extraction failures are data, not
    /// control flow, so callers outside this module build them too.
    pub(crate) fn unread(member: Option<String>, format: &str, reason: String) -> Self {
        SupplementExtractionProblem {
            member,
            format: format.to_string(),
            reason,
            empty_text: false,
        }
    }

    /// The extraction outcome that is exactly this problem.
    pub fn into_extraction(self) -> SupplementTextExtraction {
        SupplementTextExtraction {
            blocks: Vec::new(),
            truncated: false,
            problems: vec![self],
        }
    }
}

/// Extraction result for one source file.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SupplementTextExtraction {
    pub blocks: Vec<SupplementTextBlock>,
    /// A limit stopped extraction early; the file's later content is unsearched.
    pub truncated: bool,
    pub problems: Vec<SupplementExtractionProblem>,
}

/// Extracts the text of one supplement file into located blocks.
///
/// Failures are reported as problems instead of aborting, so one unread
/// member never hides another member's text.
pub fn extract_supplement_text(
    format: SupplementFormat,
    bytes: &[u8],
    budget: &SupplementTextBudget,
) -> SupplementTextExtraction {
    extract_with_budget(format, bytes, budget, &mut ArchiveBudget::default())
}

fn extract_with_budget(
    format: SupplementFormat,
    bytes: &[u8],
    budget: &SupplementTextBudget,
    spend: &mut ArchiveBudget,
) -> SupplementTextExtraction {
    let mut extraction = SupplementTextExtraction::default();
    match format {
        SupplementFormat::Xlsx | SupplementFormat::Xls => {
            extract_workbook(format, bytes, budget, spend, &mut extraction)
        }
        SupplementFormat::Csv => extract_csv(bytes, &mut extraction),
        SupplementFormat::Docx => extract_docx(bytes, budget, spend, &mut extraction),
        SupplementFormat::Pdf => extract_pdf(bytes, budget, &mut extraction),
        SupplementFormat::Zip => {
            extract_archive(bytes, budget, budget.archive_depth, spend, &mut extraction);
        }
    }
    extraction
}

fn truncate_reason(reason: String) -> String {
    const LIMIT: usize = 200;
    if reason.chars().count() <= LIMIT {
        return reason;
    }
    let cut: String = reason.chars().take(LIMIT).collect();
    format!("{cut}…")
}

fn extract_workbook(
    format: SupplementFormat,
    bytes: &[u8],
    budget: &SupplementTextBudget,
    spend: &mut ArchiveBudget,
    extraction: &mut SupplementTextExtraction,
) {
    if format == SupplementFormat::Xlsx {
        if let Err(reason) = admit_package(bytes, budget, spend) {
            extraction
                .problems
                .push(SupplementExtractionProblem::unread(
                    None,
                    format.as_str(),
                    if reason.starts_with("package could") {
                        format!("workbook could not be opened: {reason}")
                    } else {
                        reason
                    },
                ));
            return;
        }
    } else if let Err(reason) = spend.admit(bytes.len() as u64, budget) {
        extraction
            .problems
            .push(SupplementExtractionProblem::unread(
                None,
                format.as_str(),
                reason.into(),
            ));
        return;
    }
    let cursor = Cursor::new(bytes);
    let mut workbook = match open_workbook_auto_from_rs(cursor) {
        Ok(workbook) => workbook,
        Err(error) => {
            extraction
                .problems
                .push(SupplementExtractionProblem::unread(
                    None,
                    format.as_str(),
                    truncate_reason(format!("workbook could not be opened: {error:?}")),
                ));
            return;
        }
    };
    for worksheet in workbook.sheet_names().to_vec() {
        let range = match workbook.worksheet_range(&worksheet) {
            Ok(range) => range,
            Err(error) => {
                extraction
                    .problems
                    .push(SupplementExtractionProblem::unread(
                        None,
                        format.as_str(),
                        truncate_reason(format!("worksheet could not be read: {error}")),
                    ));
                return;
            }
        };
        let Some((first_row, _)) = range.start() else {
            continue;
        };
        for (relative, row) in range.rows().enumerate() {
            let text = row_text(row);
            if text.is_empty() {
                continue;
            }
            if let Err(reason) = spend.admit(text.len() as u64, budget) {
                extraction
                    .problems
                    .push(SupplementExtractionProblem::unread(
                        None,
                        format.as_str(),
                        reason.into(),
                    ));
                return;
            }
            extraction.blocks.push(SupplementTextBlock {
                member: None,
                worksheet: Some(worksheet.clone()),
                row: Some(first_row + relative as u32 + 1),
                page: None,
                paragraph: None,
                text,
            });
        }
    }
}

fn row_text(row: &[Data]) -> String {
    let mut text = String::new();
    for cell in row {
        if matches!(cell, Data::Empty) {
            continue;
        }
        if !text.is_empty() {
            text.push('\t');
        }
        text.push_str(&cell.to_string());
    }
    text
}

fn extract_csv(bytes: &[u8], extraction: &mut SupplementTextExtraction) {
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .has_headers(false)
        .from_reader(Cursor::new(bytes));
    let mut record = csv::ByteRecord::new();
    let mut row = 0u32;
    loop {
        row = row.saturating_add(1);
        match reader.read_byte_record(&mut record) {
            Ok(true) => {}
            Ok(false) => break,
            Err(error) => {
                extraction
                    .problems
                    .push(SupplementExtractionProblem::unread(
                        None,
                        "csv",
                        truncate_reason(format!("csv record {row} could not be parsed: {error}")),
                    ));
                return;
            }
        }
        let mut text = String::new();
        for field in &record {
            if !text.is_empty() {
                text.push('\t');
            }
            text.push_str(&String::from_utf8_lossy(field));
        }
        if text.is_empty() {
            continue;
        }
        extraction.blocks.push(SupplementTextBlock {
            member: None,
            worksheet: None,
            row: Some(row),
            page: None,
            paragraph: None,
            text,
        });
    }
}

fn extract_docx(
    bytes: &[u8],
    budget: &SupplementTextBudget,
    spend: &mut ArchiveBudget,
    extraction: &mut SupplementTextExtraction,
) {
    if let Err(reason) = admit_package(bytes, budget, spend) {
        extraction
            .problems
            .push(SupplementExtractionProblem::unread(None, "docx", reason));
        return;
    }
    let mut archive = match zip::ZipArchive::new(Cursor::new(bytes)) {
        Ok(archive) => archive,
        Err(error) => {
            extraction
                .problems
                .push(SupplementExtractionProblem::unread(
                    None,
                    "docx",
                    truncate_reason(format!("docx package could not be opened: {error}")),
                ));
            return;
        }
    };
    let mut document = Vec::new();
    match archive.by_name("word/document.xml") {
        Ok(mut file) => {
            if let Err(error) = file
                .by_ref()
                .take(budget.max_member_bytes.saturating_add(1))
                .read_to_end(&mut document)
            {
                extraction
                    .problems
                    .push(SupplementExtractionProblem::unread(
                        None,
                        "docx",
                        truncate_reason(format!(
                            "docx member word/document.xml could not be read: {error}"
                        )),
                    ));
                return;
            }
        }
        Err(error) => {
            extraction
                .problems
                .push(SupplementExtractionProblem::unread(
                    None,
                    "docx",
                    truncate_reason(format!("docx member word/document.xml is missing: {error}")),
                ));
            return;
        }
    }
    let Ok(xml) = std::str::from_utf8(&document) else {
        extraction
            .problems
            .push(SupplementExtractionProblem::unread(
                None,
                "docx",
                "docx member word/document.xml is not UTF-8".to_string(),
            ));
        return;
    };
    let doc = match crate::xml::parse_external_xml(xml, crate::xml::ARTICLE_XML_NODE_LIMIT) {
        Ok(doc) => doc,
        Err(_) => {
            extraction
                .problems
                .push(SupplementExtractionProblem::unread(
                    None,
                    "docx",
                    "docx member word/document.xml is not parseable XML".to_string(),
                ));
            return;
        }
    };
    let mut paragraph = 0u32;
    for node in doc.descendants().filter(|node| node.is_element()) {
        if node.tag_name().name() != "p" {
            continue;
        }
        paragraph = paragraph.saturating_add(1);
        let mut text = String::new();
        for run in node.descendants().filter(|run| run.is_element()) {
            match run.tag_name().name() {
                "t" => {
                    if let Some(run_text) = run.text() {
                        text.push_str(run_text);
                    }
                }
                "tab" if !text.is_empty() => text.push('\t'),
                _ => {}
            }
        }
        if text.is_empty() {
            continue;
        }
        extraction.blocks.push(SupplementTextBlock {
            member: None,
            worksheet: None,
            row: None,
            page: None,
            paragraph: Some(paragraph),
            text,
        });
    }
}

fn extract_pdf(
    bytes: &[u8],
    budget: &SupplementTextBudget,
    extraction: &mut SupplementTextExtraction,
) {
    let document = match unpdf::parse_bytes(bytes) {
        Ok(document) => document,
        Err(error) => {
            extraction
                .problems
                .push(SupplementExtractionProblem::unread(
                    None,
                    "pdf",
                    truncate_reason(format!("pdf could not be parsed: {error}")),
                ));
            return;
        }
    };
    let page_count = document.page_count();
    let last_page = page_count.min(budget.max_pdf_pages);
    if page_count > budget.max_pdf_pages {
        extraction.truncated = true;
    }
    for (index, page) in document.pages.iter().take(last_page as usize).enumerate() {
        // unpdf numbers parsed pages from zero; the locator reports the
        // one-based page a reader opens.
        let text = page.plain_text().trim().to_string();
        if text.is_empty() {
            continue;
        }
        extraction.blocks.push(SupplementTextBlock {
            member: None,
            worksheet: None,
            row: None,
            page: Some(index as u32 + 1),
            paragraph: None,
            text,
        });
    }
}

/// Decompressed bytes read so far from one source file.
#[derive(Default)]
struct ArchiveBudget {
    aggregate: u64,
}

impl ArchiveBudget {
    fn admit(&mut self, size: u64, budget: &SupplementTextBudget) -> Result<(), &'static str> {
        if size > budget.max_member_bytes {
            return Err("archive member exceeds the extraction size limit");
        }
        if self.aggregate.saturating_add(size) > budget.max_aggregate_bytes {
            return Err("extraction size limit exceeded");
        }
        self.aggregate += size;
        Ok(())
    }
}

// Inspect every package member before parsers allocate decompressed XML. The
// same ledger travels through nested ZIPs, workbooks and document packages.
fn admit_package(
    bytes: &[u8],
    budget: &SupplementTextBudget,
    spend: &mut ArchiveBudget,
) -> Result<(), String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|error| format!("package could not be opened: {error}"))?;
    for index in 0..archive.len() {
        let file = archive
            .by_index(index)
            .map_err(|error| format!("package member could not be opened: {error}"))?;
        spend.admit(file.size(), budget).map_err(str::to_owned)?;
    }
    Ok(())
}

fn extract_archive(
    bytes: &[u8],
    budget: &SupplementTextBudget,
    depth: usize,
    spend: &mut ArchiveBudget,
    extraction: &mut SupplementTextExtraction,
) {
    let mut archive = match zip::ZipArchive::new(Cursor::new(bytes)) {
        Ok(archive) => archive,
        Err(error) => {
            extraction
                .problems
                .push(SupplementExtractionProblem::unread(
                    None,
                    "zip",
                    truncate_reason(format!("archive could not be opened: {error}")),
                ));
            return;
        }
    };
    for index in 0..archive.len() {
        let mut file = match archive.by_index(index) {
            Ok(file) => file,
            Err(error) => {
                extraction
                    .problems
                    .push(SupplementExtractionProblem::unread(
                        None,
                        "zip",
                        truncate_reason(format!(
                            "archive member {index} could not be opened: {error}"
                        )),
                    ));
                continue;
            }
        };
        let path = match file.enclosed_name() {
            Some(path) => path.to_string_lossy().into_owned(),
            None => {
                extraction
                    .problems
                    .push(SupplementExtractionProblem::unread(
                        Some(
                            file.name()
                                .ok()
                                .map(|name| name.into_owned())
                                .unwrap_or_else(|| {
                                    String::from_utf8_lossy(file.name_raw()).into_owned()
                                }),
                        ),
                        "zip",
                        "archive member name is not a safe path".to_string(),
                    ));
                continue;
            }
        };
        if file.is_dir() {
            continue;
        }
        if let Err(reason) = spend.admit(file.size(), budget) {
            let member_format = member_format_name(&path);
            extraction
                .problems
                .push(SupplementExtractionProblem::unread(
                    Some(path),
                    &member_format,
                    reason.to_string(),
                ));
            extraction.truncated = true;
            continue;
        }
        let mut member = Vec::with_capacity(file.size().min(budget.max_member_bytes) as usize);
        if let Err(error) = file
            .by_ref()
            .take(budget.max_member_bytes.saturating_add(1))
            .read_to_end(&mut member)
        {
            let member_format = member_format_name(&path);
            extraction
                .problems
                .push(SupplementExtractionProblem::unread(
                    Some(path),
                    &member_format,
                    truncate_reason(format!("archive member could not be read: {error}")),
                ));
            continue;
        }
        if member.is_empty() {
            let member_format = member_format_name(&path);
            extraction
                .problems
                .push(SupplementExtractionProblem::unread(
                    Some(path),
                    &member_format,
                    "empty archive member".to_string(),
                ));
            continue;
        }
        let Some(format) = supplement_format_for_filename(&path) else {
            extraction
                .problems
                .push(SupplementExtractionProblem::unread(
                    Some(path),
                    "zip",
                    "unsupported member format".to_string(),
                ));
            continue;
        };
        if matches!(format, SupplementFormat::Zip) {
            if depth == 0 {
                extraction
                    .problems
                    .push(SupplementExtractionProblem::unread(
                        Some(path),
                        "zip",
                        "nested archive exceeds the extraction depth".to_string(),
                    ));
                continue;
            }
            extract_member_archive(&member, &path, budget, depth - 1, spend, extraction);
            continue;
        }
        extract_member(format, &member, &path, budget, spend, extraction);
    }
}

fn extract_member_archive(
    bytes: &[u8],
    member: &str,
    budget: &SupplementTextBudget,
    depth: usize,
    spend: &mut ArchiveBudget,
    extraction: &mut SupplementTextExtraction,
) {
    let mut nested = SupplementTextExtraction::default();
    extract_archive(bytes, budget, depth, spend, &mut nested);
    for mut block in nested.blocks {
        block.member = Some(match block.member.take() {
            Some(inner) => format!("{member}/{inner}"),
            None => member.to_string(),
        });
        extraction.blocks.push(block);
    }
    for mut problem in nested.problems {
        match problem.member.take() {
            Some(inner) => problem.member = Some(format!("{member}/{inner}")),
            None => problem.member = Some(member.to_string()),
        }
        extraction.problems.push(problem);
    }
    extraction.truncated |= nested.truncated;
}

fn extract_member(
    format: SupplementFormat,
    bytes: &[u8],
    member: &str,
    budget: &SupplementTextBudget,
    spend: &mut ArchiveBudget,
    extraction: &mut SupplementTextExtraction,
) {
    let nested = extract_with_budget(format, bytes, budget, spend);
    let member_textless = nested.blocks.is_empty() && nested.problems.is_empty();
    for mut block in nested.blocks {
        block.member = Some(member.to_string());
        extraction.blocks.push(block);
    }
    for mut problem in nested.problems {
        problem.member = Some(member.to_string());
        extraction.problems.push(problem);
    }
    if member_textless {
        // The member read fine but holds no text: report it so a member that
        // searched empty stays distinguishable from one never attempted.
        extraction.problems.push(SupplementExtractionProblem {
            member: Some(member.to_string()),
            format: format.as_str().to_string(),
            reason: "no text extracted".to_string(),
            empty_text: true,
        });
    }
    extraction.truncated |= nested.truncated;
}

fn member_format_name(member: &str) -> String {
    supplement_format_for_filename(member)
        .map(|format| format.as_str().to_string())
        .unwrap_or_else(|| "zip".to_string())
}

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct CleanReport {
    pub(crate) dry_run: bool,
    pub(crate) orphans_removed: usize,
    pub(crate) entries_removed: usize,
    pub(crate) bytes_freed: u64,
    pub(crate) provider_capture_bytes_freed: u64,
    pub(crate) errors: Vec<String>,
}

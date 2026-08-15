#[cfg(any(test, not(unix)))]
pub(super) fn retained_bytes(root: &std::path::Path) -> Result<u64, super::ProviderCaptureError> {
    match std::fs::symlink_metadata(root.join("captures")) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(0),
        Err(_) | Ok(_) => Err(super::ProviderCaptureError::Corrupt),
    }
}

#[cfg(test)]
mod tests {
    use super::{super::ProviderCaptureError, retained_bytes};
    use crate::test_support::TempDirGuard;

    #[test]
    fn absent_capture_store_is_the_only_supported_empty_state() {
        let root = TempDirGuard::new("provider-capture-unsupported-stats");
        assert_eq!(retained_bytes(root.path()), Ok(0));

        std::fs::create_dir(root.path().join("captures")).expect("capture directory");
        assert_eq!(
            retained_bytes(root.path()),
            Err(ProviderCaptureError::Corrupt)
        );
    }
}

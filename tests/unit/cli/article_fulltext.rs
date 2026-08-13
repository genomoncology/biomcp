use super::*;

#[test]
fn line_ranges_are_one_based_ordered_and_bounded() {
    for value in ["0:1", "2:1", "1:501", "x:2", "1"] {
        assert!(parse_range(value).is_err(), "{value} should be rejected");
    }
    assert_eq!(parse_range("2:501").unwrap(), (2, 501));
}

#[test]
fn line_output_stops_only_at_complete_utf8_lines() {
    let first = format!("{}\n", "é".repeat(32_000));
    let text = format!("{first}{}\n", "x".repeat(2_000));
    let result = build_lines(&text, "1:2").unwrap();
    assert_eq!(result.text, first);
    assert!(result.truncated);
    assert_eq!(result.next_line, Some(2));
    assert!(result.returned_bytes <= MAX_RANGE_BYTES);
}

#[test]
fn a_single_oversized_line_is_rejected() {
    let text = "x".repeat(MAX_RANGE_BYTES + 1);
    assert!(matches!(
        build_lines(&text, "1:1"),
        Err(BioMcpError::InputTooLarge {
            limit_bytes: MAX_RANGE_BYTES
        })
    ));
}

#[test]
fn outline_preserves_duplicate_headings_and_reports_the_cap() {
    let text = (0..201).map(|_| "## repeated\nbody\n").collect::<String>();
    let outline = build_outline(&text);
    assert_eq!(outline.total, 201);
    assert_eq!(outline.returned, 200);
    assert!(outline.has_more);
    assert_eq!(outline.headings[0].title, outline.headings[1].title);
    assert_eq!(outline.headings[0].end_line, 2);
}

#[test]
fn heading_titles_are_truncated_on_utf8_boundaries() {
    let outline = build_outline(&format!("# {}\n", "é".repeat(300)));
    assert!(outline.headings[0].title_truncated);
    assert!(outline.headings[0].title.len() <= 512);
    assert!(
        outline.headings[0]
            .title
            .is_char_boundary(outline.headings[0].title.len())
    );
}

#[cfg(unix)]
#[test]
fn fulltext_views_reject_a_symlink_instead_of_reading_its_target() {
    use std::os::unix::fs::symlink;

    let root = crate::test_support::TempDirGuard::new("linked-fulltext-view");
    let outside = root.path().join("outside.txt");
    let linked = root.path().join("article.txt");
    std::fs::write(&outside, "# private\nsecret\n").unwrap();
    symlink(&outside, &linked).unwrap();

    assert!(summary(&linked).is_err());
    assert!(render(&linked, true, None, true).is_err());
    assert!(render(&linked, false, Some("1:1"), true).is_err());
}

#[cfg(unix)]
#[test]
fn an_open_managed_handle_is_not_redirected_by_path_replacement() {
    use std::io::Read as _;
    use std::os::unix::fs::symlink;

    let root = crate::test_support::TempDirGuard::new("fulltext-handle-swap");
    let path = root.path().join("article.txt");
    let outside = root.path().join("outside.txt");
    std::fs::write(&path, "original").unwrap();
    std::fs::write(&outside, "outside").unwrap();
    let mut opened = crate::cache::open_managed_read(&path).expect("open original file");
    std::fs::remove_file(&path).unwrap();
    symlink(&outside, &path).unwrap();

    let mut text = String::new();
    opened.read_to_string(&mut text).unwrap();
    assert_eq!(text, "original");
    assert!(crate::cache::open_managed_read(&path).is_err());
}

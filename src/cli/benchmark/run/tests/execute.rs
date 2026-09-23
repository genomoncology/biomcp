//! Execution-side benchmark tests.

use super::trim_excerpt;

#[test]
fn excerpt_cut_moves_back_from_a_multibyte_boundary() {
    let text = format!("{}β", "a".repeat(239));
    assert_eq!(trim_excerpt(&text), format!("{}...", "a".repeat(239)));
}

//! Internal utility helpers for date parsing, downloads, query escaping, and serde helpers.

pub(crate) mod date;
pub(crate) mod download;
pub(crate) mod query;
pub(crate) mod serde;
pub(crate) mod sync;

/// The message a panic payload carries, for error text that must
/// survive a caught panic. Shared by the CLI runner and the MCP tool
/// wrapper so both render the same payload the same way.
pub(crate) fn panic_payload_message(payload: &(dyn std::any::Any + Send)) -> &str {
    if let Some(message) = payload.downcast_ref::<String>() {
        message
    } else if let Some(message) = payload.downcast_ref::<&str>() {
        message
    } else {
        "unknown panic payload"
    }
}

#[cfg(test)]
mod panic_payload_tests {
    use super::panic_payload_message;

    #[test]
    fn string_and_str_payloads_keep_their_message() {
        let owned = String::from("panic: owned message");
        assert_eq!(panic_payload_message(&owned), "panic: owned message");
        let borrowed: &str = "panic: borrowed message";
        assert_eq!(panic_payload_message(&borrowed), "panic: borrowed message");
    }

    #[test]
    fn any_other_payload_falls_back_to_the_generic_line() {
        let number = 7_u32;
        assert_eq!(panic_payload_message(&number), "unknown panic payload");
    }
}

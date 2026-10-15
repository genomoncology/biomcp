//! Panic backtraces must still name the file and line in dev and test builds:
//! the Cargo profiles carry line tables only, and line tables are exactly the
//! part of debug info a backtrace needs. This test panics, captures the panic
//! backtrace, and refuses any build whose backtrace loses line info.

use std::backtrace::Backtrace;
use std::panic;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[test]
fn panic_backtrace_names_file_and_line() {
    let captured: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let sink = Arc::clone(&captured);
    let previous = panic::take_hook();
    panic::set_hook(Box::new(move |_info| {
        let mut text = sink.lock().expect("panic hook sink");
        if text.is_empty() {
            // Only the backtrace proves line info: the panic location string
            // is embedded in the binary even without any debug info.
            text.push_str(&Backtrace::force_capture().to_string());
        }
    }));
    let caught = panic::catch_unwind(|| panic!("backtrace line-info proof"));
    panic::set_hook(previous);
    assert!(caught.is_err(), "the proof panic did not run");
    let backtrace = captured.lock().expect("panic hook sink").clone();
    println!("PROOF-BACKTRACE:\n{backtrace}");
    let file = Path::new(file!())
        .file_name()
        .and_then(|name| name.to_str())
        .expect("test file name");
    let names_line = backtrace.lines().any(|line| {
        line.rfind(file).is_some_and(|at| {
            let numbers = line[at + file.len()..].strip_prefix(':').unwrap_or("");
            let (row, col) = numbers.split_once(':').unwrap_or(("", ""));
            row.parse::<u32>().is_ok() && col.starts_with(|c: char| c.is_ascii_digit())
        })
    });
    assert!(
        names_line,
        "backtrace names no {file}:LINE:COL frame:\n{backtrace}"
    );
}

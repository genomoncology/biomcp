#![deny(clippy::dbg_macro)]
#![deny(clippy::print_stderr)]
#![deny(clippy::print_stdout)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]

pub mod cli;
pub mod error;
pub mod mcp;

mod entities;
mod render;
mod sources;
#[cfg(test)]
pub(crate) mod test_support;
mod transform;
mod utils;
